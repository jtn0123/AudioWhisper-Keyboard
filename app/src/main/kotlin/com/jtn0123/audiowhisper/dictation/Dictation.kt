package com.jtn0123.audiowhisper.dictation

import android.Manifest
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.provider.Settings
import android.util.Log
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Dictation inside the keyboard. A tap on the mic key toggles it and a hold talks until
 * it is let go (see [MicKey]); while it listens,
 * each phrase is typed as the user pauses and every key keeps working, so
 * the keyboard never goes away. Hiding the keyboard stops it, and leaving
 * the field drops whatever it hasn't typed yet: a dictation only ever types
 * into the field it started in.
 *
 * Main thread only, apart from [worker], which owns the model: it loads,
 * runs and releases it, and keeps phrases in the order they were spoken.
 * The model takes about 800 MB, so it stays loaded for [KEEP_LOADED_MS]
 * after a dictation (the next one starts at once) and is then released.
 *
 * Every dictation that ends without typing anything says why: nothing was
 * heard, the mic stopped, the model is broken, or transcription failed.
 *
 * While a phrase is being spoken its words so far are shown in [preview]. Previews share
 * [worker] with the phrases, in order, so the text typed never waits for more than one.
 */
class Dictation(private val context: Context, private val editor: Editor) {
    /** Where dictation types. Called on the main thread. */
    interface Editor {
        /** The field is one dictation may type into (see [DictationFields]). */
        fun allowsDictation(): Boolean
        fun textBeforeCursor(): CharSequence?

        /** Deletes [replacing] characters before the cursor, then types [text]. */
        fun type(text: String, replacing: Int)

        /** A capitalised word is usually written that way ("John"), so it keeps its capital mid-sentence. */
        fun isName(word: String): Boolean

        /** Takes back the phrase dictation typed last, if nothing has changed since. */
        fun undoLastPhrase(): Boolean
        fun problem(message: String)

        /** A key-press haptic, for letting go of the mic after a hold: the press had one, the release none. */
        fun tick()
    }

    enum class Status { IDLE, LISTENING, FINISHING }

    private val main = Handler(Looper.getMainLooper())
    private val worker: ExecutorService = Executors.newSingleThreadExecutor()
    private val models = ModelHolder(::loadModel) // worker thread only
    private var listener: PhraseListener? = null
    private val micKey = MicKey(SystemClock::uptimeMillis)

    /** A preview is queued or running; another isn't queued behind it. */
    private val previewing = AtomicBoolean(false)

    /** Bumped for each dictation and to abandon one; late phrases from an old one are dropped. */
    private var session = 0
    private var typedAny = false
    private var failures = 0

    /** What this dictation typed last, so a full stop it put at a pause can be taken back. */
    private var lastPhrase: String? = null

    /** A message about this dictation has been shown, so its end needn't add one. */
    private var explained = false

    /** The mic key went down. */
    fun pressMic() {
        if (micKey.press(status.value) == MicKey.Action.START) start()
    }

    /** The mic key came up, or the finger slid off it. */
    fun releaseMic() {
        when (micKey.release()) {
            MicKey.Action.FINISH -> if (status.value == Status.LISTENING) finish()
            MicKey.Action.END_HOLD -> if (status.value == Status.LISTENING) {
                editor.tick()
                finish()
            }
            MicKey.Action.TOGGLE -> toggle()
            MicKey.Action.START, MicKey.Action.NONE -> Unit
        }
    }

    private fun toggle() {
        when (status.value) {
            Status.IDLE -> start()
            Status.LISTENING -> finish()
            Status.FINISHING -> Unit
        }
    }

    /**
     * The keyboard was hidden (back, the hide key) but its field is still there: stop
     * listening and type what was said, as a tap on the mic would. No message, since the
     * keyboard is gone; a dictation already finishing carries on.
     */
    fun stopListening() {
        if (status.value != Status.LISTENING) return
        explained = true
        finish()
    }

    /** The field went away. Typed phrases stay; the rest, including the one being spoken, is dropped. */
    fun abandon() {
        if (status.value == Status.IDLE) return
        session++
        listener?.stop(keepLastPhrase = false)
        listener = null
        idle()
    }

    fun release() {
        abandon()
        main.removeCallbacks(unload)
        worker.execute(::closeModel)
        worker.shutdown()
    }

    private fun start() {
        if (!mayStart()) return
        val id = ++session
        val next = newListener(id)
        try {
            next.start(::openAudio)
        } catch (e: IllegalStateException) {
            Log.e(TAG, "Could not start recording", e)
            editor.problem("Microphone unavailable. Another app may be using it.")
            return
        }
        main.removeCallbacks(unload)
        preload(id)
        listener = next
        typedAny = false
        failures = 0
        lastPhrase = null
        explained = false
        mutableStatus.value = Status.LISTENING
    }

    private fun mayStart(): Boolean {
        // The mic is greyed out in these fields; this also covers a tap that raced the field change.
        if (!editor.allowsDictation()) {
            editor.problem("Dictation is off in password and number fields")
            return false
        }
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED) {
            editor.problem("Allow microphone access to dictate")
            context.startActivity(appSettings())
            return false
        }
        return modelInstalled()
    }

    /**
     * Loads the model, if it was released, while the user starts talking; stops at once if it
     * can't. While it loads the mic shows [warming], since the first phrase will be a little late.
     */
    private fun preload(id: Int) = worker.execute {
        models.retry()
        val loading = !models.isLoaded
        if (loading) main.post { if (id == session && status.value != Status.IDLE) mutableWarming.value = true }
        try {
            models.get()
        } catch (e: ModelLoadException) {
            Log.e(TAG, "Could not load the model", e)
            main.post { if (id == session && status.value != Status.IDLE) modelFailed() }
        } finally {
            if (loading) main.post { mutableWarming.value = false }
        }
    }

    private fun modelFailed() {
        editor.problem("Speech model missing or damaged")
        abandon()
    }

    /** The mic failed mid-dictation. The words heard until then are still typed. */
    private fun micStopped() {
        editor.problem("The mic stopped. Another app may be using it.")
        explained = true
        finish()
    }

    private fun modelInstalled(): Boolean {
        val missing = ModelFiles.missing(context)
        if (missing.isNotEmpty()) {
            Log.w(TAG, "Missing model files: $missing in ${ModelFiles.dir(context)}")
            editor.problem("Speech model not installed")
            return false
        }
        return true
    }

    /**
     * The mic, or in a debug build a recording copied in for testing (see [MicAudio.Replay]).
     * [mayStart] asked for the permission already; checked again here, where the mic opens, in
     * case it was revoked since, which [start] then explains like any mic that won't open.
     */
    private fun openAudio(): MicAudio {
        val replay = MicAudio.Replay.file(context)
        if (replay == null) {
            val granted = context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED
            check(granted) { "No microphone permission" }
            return MicAudio.Mic.open()
        }
        Log.i(TAG, "Listening to ${replay.name} instead of the mic")
        return try {
            MicAudio.Replay.open(replay)
        } catch (e: IllegalArgumentException) {
            throw IllegalStateException("Could not read ${replay.name}", e)
        }
    }

    private fun appSettings() = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS)
        .setData(Uri.fromParts("package", context.packageName, null))
        .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

    private fun newListener(id: Int) = PhraseListener(
        context.assets,
        onLevel = { level -> main.post { if (id == session && status.value == Status.LISTENING) mutableLevel.value = level } },
        onMicError = { code ->
            Log.e(TAG, "Microphone failed while recording: $code")
            main.post { if (id == session && status.value == Status.LISTENING) micStopped() }
        },
        onPhrase = { phrase ->
            worker.execute {
                val result = runCatching { timed(phrase) }
                main.post { type(id, result) }
            }
        },
        onPreview = { phrase -> preview(id, phrase) },
    )

    /** Microphone thread. Skipped while the last preview is unfinished, so they never pile up. */
    private fun preview(id: Int, phrase: FloatArray) {
        if (!previewing.compareAndSet(false, true)) return
        worker.execute {
            val started = SystemClock.elapsedRealtime()
            val text = try {
                runCatching { models.get().transcribe(phrase) }.getOrNull()
            } finally {
                previewing.set(false)
            }
            val seconds = phrase.size / Transcriber.SAMPLE_RATE.toFloat()
            Log.d(TAG, "Previewed %.1f s in %d ms".format(seconds, SystemClock.elapsedRealtime() - started))
            if (text != null) main.post { showPreview(id, text) }
        }
    }

    /** Shown until the phrase itself is typed; one that comes after the mic stopped is dropped. */
    private fun showPreview(id: Int, text: String) {
        if (id == session && status.value == Status.LISTENING) mutablePreview.value = text.trim()
    }

    private fun finish() {
        val id = session
        listener?.stop(keepLastPhrase = true) ?: return
        listener = null
        mutableStatus.value = Status.FINISHING
        mutableLevel.value = 0f
        // Queued behind every phrase handed over so far, so it runs last.
        worker.execute { main.post { if (id == session) finished() } }
    }

    private fun finished() {
        if (!explained) endingMessage(typedAny, failures)?.let(editor::problem)
        idle()
    }

    private fun idle() {
        mutableStatus.value = Status.IDLE
        mutableLevel.value = 0f
        mutableWarming.value = false
        mutablePreview.value = ""
        main.removeCallbacks(unload)
        main.postDelayed(unload, KEEP_LOADED_MS)
    }

    private fun type(id: Int, result: Result<String>) {
        if (id != session) return
        mutablePreview.value = ""
        result.onSuccess { text ->
            if (text.isNotBlank()) {
                Spoken.parse(text).forEach(::act)
                typedAny = true
            }
        }.onFailure { error ->
            failures++
            Log.e(TAG, "Transcription failed", error)
        }
    }

    private fun act(spoken: Spoken) {
        when (spoken) {
            is Spoken.Words -> insert(Phrases.join(editor.textBeforeCursor(), lastPhrase, spoken.text, editor::isName))
            is Spoken.Mark -> insert(Phrases.punctuate(editor.textBeforeCursor(), lastPhrase, spoken.mark))
            is Spoken.Break -> insert(Phrases.Typing(0, spoken.text))
            Spoken.ScratchThat -> {
                if (!editor.undoLastPhrase()) Log.i(TAG, "Nothing to scratch")
                lastPhrase = null
            }
        }
    }

    private fun insert(typing: Phrases.Typing) {
        editor.type(typing.text, typing.replacing)
        lastPhrase = typing.text
    }

    private fun timed(audio: FloatArray): String {
        val started = SystemClock.elapsedRealtime()
        val text = models.get().transcribe(audio)
        val seconds = audio.size / Transcriber.SAMPLE_RATE.toFloat()
        Log.i(TAG, "Transcribed %.1f s phrase in %d ms".format(seconds, SystemClock.elapsedRealtime() - started))
        return text
    }

    /** Worker thread only, through [models]. */
    private fun loadModel(): Transcriber {
        val started = SystemClock.elapsedRealtime()
        return Transcriber(ModelFiles.dir(context)).also {
            Log.i(TAG, "Model loaded in ${SystemClock.elapsedRealtime() - started} ms")
        }
    }

    /** Worker thread only. */
    private fun closeModel() {
        if (models.close()) Log.i(TAG, "Model released")
    }

    private val unload = Runnable { worker.execute(::closeModel) }

    companion object {
        private const val TAG = "AWDictation"
        private const val KEEP_LOADED_MS = 5 * 60 * 1000L

        // One keyboard per process, so the mic button can watch these
        // without holding on to the service.
        private val mutableStatus = MutableStateFlow(Status.IDLE)
        private val mutableLevel = MutableStateFlow(0f)
        private val mutablePreview = MutableStateFlow("")
        private val mutableWarming = MutableStateFlow(false)

        /** What the mic button shows. */
        val status: StateFlow<Status> = mutableStatus.asStateFlow()

        /** Loudness while listening, 0..1, for the mic's glow. */
        val level: StateFlow<Float> = mutableLevel.asStateFlow()

        /** The words of the phrase being spoken, as heard so far; empty when there is none. Never logged. */
        val preview: StateFlow<String> = mutablePreview.asStateFlow()

        /** The model is loading for this dictation (about 2 s on a phone), so its first phrase comes late. */
        val warming: StateFlow<Boolean> = mutableWarming.asStateFlow()
    }
}

/** What to say when a dictation ends without a message already shown, or null when it typed something. */
internal fun endingMessage(typedAny: Boolean, failures: Int): String? = when {
    typedAny -> null
    failures > 0 -> "Transcription failed"
    else -> "Didn't catch that"
}
