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

/**
 * Dictation inside the keyboard. The mic key toggles it; while it listens,
 * each phrase is typed as the user pauses and every key keeps working, so
 * the keyboard never goes away. Hiding the keyboard stops it.
 *
 * Main thread only, apart from [worker], which owns the model: it loads,
 * runs and releases it, and keeps phrases in the order they were spoken.
 * The model takes about 800 MB, so it stays loaded for [KEEP_LOADED_MS]
 * after a dictation (the next one starts at once) and is then released.
 */
class Dictation(private val context: Context, private val editor: Editor) {
    /** Where dictation types. Called on the main thread. */
    interface Editor {
        fun textBeforeCursor(): CharSequence?
        fun type(text: String)
        fun problem(message: String)
    }

    enum class Status { IDLE, LISTENING, FINISHING }

    private val main = Handler(Looper.getMainLooper())
    private val worker: ExecutorService = Executors.newSingleThreadExecutor()
    private var transcriber: Transcriber? = null // worker thread only
    private var listener: PhraseListener? = null

    /** Bumped to abandon a dictation; late phrases from it are dropped. */
    private var session = 0
    private var typedAny = false
    private var failures = 0

    /** The mic key. */
    fun toggle() {
        when (status.value) {
            Status.IDLE -> start()
            Status.LISTENING -> finish()
            Status.FINISHING -> Unit
        }
    }

    /** The keyboard went away. Typed phrases stay; the unfinished one is dropped. */
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
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED) {
            editor.problem("Allow microphone access to dictate")
            context.startActivity(appSettings())
            return
        }
        if (!modelInstalled()) return
        main.removeCallbacks(unload)
        // Load (if it was released) while the user starts talking.
        worker.execute { runCatching { model() }.onFailure { Log.e(TAG, "Could not load the model", it) } }
        val next = newListener(session)
        try {
            next.start()
        } catch (e: IllegalStateException) {
            Log.e(TAG, "Could not start recording", e)
            editor.problem("Microphone unavailable. Another app may be using it.")
            return
        }
        listener = next
        typedAny = false
        failures = 0
        mutableStatus.value = Status.LISTENING
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

    private fun appSettings() = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS)
        .setData(Uri.fromParts("package", context.packageName, null))
        .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

    private fun newListener(id: Int) = PhraseListener(
        context.assets,
        onLevel = { level -> main.post { if (id == session) mutableLevel.value = level } },
        onPhrase = { phrase ->
            worker.execute {
                val result = runCatching { timed(phrase) }
                main.post { type(id, result) }
            }
        },
    )

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
        if (failures > 0 && !typedAny) editor.problem("Transcription failed")
        idle()
    }

    private fun idle() {
        mutableStatus.value = Status.IDLE
        mutableLevel.value = 0f
        main.removeCallbacks(unload)
        main.postDelayed(unload, KEEP_LOADED_MS)
    }

    private fun type(id: Int, result: Result<String>) {
        if (id != session) return
        result.onSuccess { text ->
            if (text.isNotBlank()) {
                commit(text.trim(), continuing = typedAny)
                typedAny = true
            }
        }.onFailure { error ->
            failures++
            Log.e(TAG, "Transcription failed", error)
        }
    }

    /**
     * Types [text], adding a space if it would otherwise run into a word.
     * [continuing] means an earlier phrase of this dictation was just typed.
     */
    private fun commit(text: String, continuing: Boolean) {
        val before = editor.textBeforeCursor()
        val lead = if (before.isNullOrEmpty() || before.last().isWhitespace()) "" else " "
        val phrase = if (continuing) Phrases.continueSentence(before, text) else text
        editor.type(lead + phrase)
    }

    private fun timed(audio: FloatArray): String {
        val started = SystemClock.elapsedRealtime()
        val text = model().transcribe(audio)
        val seconds = audio.size / Transcriber.SAMPLE_RATE.toFloat()
        Log.i(TAG, "Transcribed %.1f s phrase in %d ms".format(seconds, SystemClock.elapsedRealtime() - started))
        return text
    }

    /** The loaded model, loading it on first use. Worker thread only. */
    private fun model(): Transcriber = transcriber ?: run {
        val started = SystemClock.elapsedRealtime()
        Transcriber(ModelFiles.dir(context)).also {
            transcriber = it
            Log.i(TAG, "Model loaded in ${SystemClock.elapsedRealtime() - started} ms")
        }
    }

    /** Worker thread only. */
    private fun closeModel() {
        transcriber?.close() ?: return
        transcriber = null
        Log.i(TAG, "Model released")
    }

    private val unload = Runnable { worker.execute(::closeModel) }

    companion object {
        private const val TAG = "AWDictation"
        private const val KEEP_LOADED_MS = 5 * 60 * 1000L

        // One keyboard per process, so the mic button can watch these
        // without holding on to the service.
        private val mutableStatus = MutableStateFlow(Status.IDLE)
        private val mutableLevel = MutableStateFlow(0f)

        /** What the mic button shows. */
        val status: StateFlow<Status> = mutableStatus.asStateFlow()

        /** Loudness while listening, 0..1, for the mic's glow. */
        val level: StateFlow<Float> = mutableLevel.asStateFlow()
    }
}
