package com.jtn0123.audiowhisper.dictation

import androidx.annotation.StringRes
import dev.patrickgold.florisboard.R
import java.util.concurrent.atomic.AtomicBoolean
import kotlinx.coroutines.flow.MutableStateFlow

/**
 * What the mic button and the smartbar show of dictation. One per process in the app
 * ([Dictation.status] and the rest); a test makes its own.
 */
internal class ReferenceDictationFlows {
    val status = MutableStateFlow(Dictation.Status.IDLE)
    val level = MutableStateFlow(0f)
    val preview = MutableStateFlow("")
    val warming = MutableStateFlow(false)
}

/**
 * Dictation's logic, from a press of the mic to the text typed, with everything Android behind
 * [Platform] so it can be unit tested. [Dictation] describes the behaviour and supplies the
 * Android side.
 *
 * Main thread only, apart from the work handed to [Platform.work], which owns the model: it
 * loads, runs and releases it, and keeps phrases in the order they were spoken.
 */
internal class ReferenceDictationSession(
    private val editor: Dictation.Editor,
    private val platform: DictationSession.Platform,
    private val flows: ReferenceDictationFlows,
) {
    private val models = ReferenceModelHolder(::loadModel) // worker only
    private var listener: DictationSession.Listening? = null
    private val micKey = MicKey(platform::uptimeMs)

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

    /** When the last dictation ended, in uptime; 0 before the first, when no model was loaded. */
    private var idleSince = 0L
    private var keyboardShown = true

    @Volatile private var released = false

    private val status get() = flows.status.value

    fun pressMic() {
        if (released) return
        if (micKey.press(status) == MicKey.Action.START) start()
    }

    fun releaseMic() {
        if (released) return
        when (micKey.release()) {
            MicKey.Action.FINISH -> if (status == Dictation.Status.LISTENING) finish()
            MicKey.Action.END_HOLD -> if (status == Dictation.Status.LISTENING) {
                editor.tick()
                finish()
            }
            MicKey.Action.TOGGLE -> toggle()
            MicKey.Action.START, MicKey.Action.NONE -> Unit
        }
    }

    private fun toggle() {
        when (status) {
            Dictation.Status.IDLE -> start()
            Dictation.Status.LISTENING -> finish()
            Dictation.Status.FINISHING -> Unit
        }
    }

    fun stopListening() {
        if (status != Dictation.Status.LISTENING) return
        explained = true
        finish()
    }

    fun abandon() {
        if (status == Dictation.Status.IDLE) return
        session++
        listener?.stop(keepLastPhrase = false)
        listener = null
        idle()
    }

    fun keyboardVisible(visible: Boolean) {
        if (released) return
        keyboardShown = visible
        if (status == Dictation.Status.IDLE && idleSince != 0L) scheduleRelease()
    }

    fun release() {
        if (released) return
        released = true
        abandon()
        platform.removeCallbacks(unload)
        platform.work(::closeModel)
        platform.shutDownWorker()
    }

    /** Frees the model now if no dictation is using it. Any thread. */
    fun freeIfIdle() {
        if (released) return
        post {
            if (status != Dictation.Status.IDLE) return@post
            platform.removeCallbacks(unload)
            unload.run()
        }
    }

    /** Drops late main-thread results and requests after the owner has released this session. */
    private fun post(action: Runnable) {
        platform.post { if (!released) action.run() }
    }

    private fun start() {
        if (!mayStart()) return
        val id = ++session
        val next = try {
            platform.listen(callbacks(id))
        } catch (e: IllegalStateException) {
            platform.log(DictationSession.LogLevel.ERROR, "Could not start recording", e)
            editor.problem(R.string.dictation__mic_busy)
            return
        }
        platform.removeCallbacks(unload)
        preload(id)
        listener = next
        typedAny = false
        failures = 0
        lastPhrase = null
        explained = false
        flows.status.value = Dictation.Status.LISTENING
    }

    private fun mayStart(): Boolean {
        // The mic is greyed out in these fields; this also covers a tap that raced the field change.
        if (!editor.allowsDictation()) {
            editor.problem(R.string.dictation__field_off)
            return false
        }
        if (!platform.hasMicPermission()) {
            editor.problem(R.string.dictation__allow_mic)
            platform.openAppSettings()
            return false
        }
        return modelInstalled()
    }

    private fun modelInstalled(): Boolean = when (val state = platform.modelState()) {
        ModelState.Ready -> true
        is ModelState.Missing -> {
            platform.log(DictationSession.LogLevel.WARN, "Missing model files: ${state.files}")
            modelHelp(R.string.dictation__not_installed)
        }
        is ModelState.Damaged -> {
            platform.log(DictationSession.LogLevel.WARN, "Model files of the wrong size: ${state.files}")
            modelHelp(R.string.dictation__damaged)
        }
    }

    /** Says what's wrong, and opens Settings › Dictation, which says how to fix it. False. */
    private fun modelHelp(@StringRes message: Int): Boolean {
        editor.problem(message)
        platform.openModelSettings()
        return false
    }

    /**
     * Loads the model, if it was released, while the user starts talking; stops at once if it
     * can't. While it loads the mic shows [Dictation.warming], since the first phrase will be a
     * little late.
     */
    private fun preload(id: Int) = platform.work {
        models.retry()
        val loading = !models.isLoaded
        if (loading) post { if (current(id)) flows.warming.value = true }
        try {
            models.get()
        } catch (e: ModelLoadException) {
            platform.log(DictationSession.LogLevel.ERROR, "Could not load the model", e)
            post { if (current(id)) modelFailed() }
        } finally {
            if (loading) post { flows.warming.value = false }
        }
    }

    private fun current(id: Int) = id == session && status != Dictation.Status.IDLE

    private fun listening(id: Int) = id == session && status == Dictation.Status.LISTENING

    private fun modelFailed() {
        editor.problem(R.string.dictation__model_failed)
        abandon()
    }

    /** The mic failed mid-dictation. The words heard until then are still typed. */
    private fun micStopped() {
        editor.problem(R.string.dictation__mic_stopped)
        explained = true
        finish()
    }

    private fun callbacks(id: Int) = DictationSession.Callbacks(
        onLevel = { level -> post { if (listening(id)) flows.level.value = level } },
        onMicError = { code ->
            platform.log(DictationSession.LogLevel.ERROR, "Microphone failed while recording: $code")
            post { if (listening(id)) micStopped() }
        },
        onPhrase = { phrase ->
            platform.work {
                val result = runCatching { timed(phrase) }
                post { type(id, result) }
            }
        },
        onPreview = { phrase -> preview(id, phrase) },
    )

    /** Microphone thread. Skipped while the last preview is unfinished, so they never pile up. */
    private fun preview(id: Int, phrase: FloatArray) {
        if (!previewing.compareAndSet(false, true)) return
        platform.work {
            val started = platform.uptimeMs()
            val text = try {
                runCatching { models.get().transcribe(phrase) }.getOrNull()
            } finally {
                previewing.set(false)
            }
            val seconds = phrase.size / Transcriber.SAMPLE_RATE.toFloat()
            platform.log(DictationSession.LogLevel.DEBUG, "Previewed %.1f s in %d ms".format(seconds, platform.uptimeMs() - started))
            if (text != null) post { showPreview(id, text) }
        }
    }

    /** Shown until the phrase itself is typed; one that comes after the mic stopped is dropped. */
    private fun showPreview(id: Int, text: String) {
        if (listening(id)) flows.preview.value = text.trim()
    }

    private fun finish() {
        val id = session
        listener?.stop(keepLastPhrase = true) ?: return
        listener = null
        flows.status.value = Dictation.Status.FINISHING
        flows.level.value = 0f
        // Queued behind every phrase handed over so far, so it runs last.
        platform.work { post { if (id == session) finished() } }
    }

    private fun finished() {
        if (!explained) referenceEndingMessage(typedAny, failures)?.let(editor::problem)
        idle()
    }

    private fun idle() {
        flows.status.value = Dictation.Status.IDLE
        flows.level.value = 0f
        flows.warming.value = false
        flows.preview.value = ""
        idleSince = platform.uptimeMs()
        scheduleRelease()
    }

    private fun scheduleRelease() {
        platform.removeCallbacks(unload)
        platform.postAtTime(unload, referenceReleaseAt(idleSince, platform.uptimeMs(), keyboardShown))
    }

    private fun type(id: Int, result: Result<String>) {
        if (id != session) return
        flows.preview.value = ""
        result.onSuccess { text ->
            if (text.isNotBlank()) {
                Spoken.parse(editor.tidy(text)).forEach(::act)
                typedAny = true
            }
        }.onFailure { error ->
            failures++
            platform.log(DictationSession.LogLevel.ERROR, "Transcription failed", error)
        }
    }

    private fun act(spoken: Spoken) {
        when (spoken) {
            is Spoken.Words -> insert(Phrases.join(editor.textBeforeCursor(), lastPhrase, spoken.text, editor::isName))
            is Spoken.Mark -> insert(Phrases.punctuate(editor.textBeforeCursor(), lastPhrase, spoken.mark))
            is Spoken.Break -> insert(Phrases.Typing(0, spoken.text))
            Spoken.ScratchThat -> {
                if (!editor.undoLastPhrase()) platform.log(DictationSession.LogLevel.INFO, "Nothing to scratch")
                lastPhrase = null
            }
        }
    }

    private fun insert(typing: Phrases.Typing) {
        editor.type(typing.text, typing.replacing)
        lastPhrase = typing.text
    }

    private fun timed(audio: FloatArray): String {
        val started = platform.uptimeMs()
        val text = models.get().transcribe(audio)
        val seconds = audio.size / Transcriber.SAMPLE_RATE.toFloat()
        platform.log(DictationSession.LogLevel.INFO, "Transcribed %.1f s phrase in %d ms".format(seconds, platform.uptimeMs() - started))
        return text
    }

    /** Worker only, through [models]. */
    private fun loadModel(): DictationSession.SpeechModel {
        val started = platform.uptimeMs()
        return platform.loadModel().also {
            platform.log(DictationSession.LogLevel.INFO, "Model loaded in ${platform.uptimeMs() - started} ms")
        }
    }

    /** Worker only. */
    private fun closeModel() {
        if (models.close()) platform.log(DictationSession.LogLevel.INFO, "Model released")
    }

    private val unload = Runnable { if (!released) platform.work(::closeModel) }
}

/**
 * When to release the model (uptime): [Dictation.KEEP_LOADED_MS] after the last dictation ended
 * at [idleSince], or, with the keyboard out of sight, [Dictation.KEEP_LOADED_HIDDEN_MS] from [now]
 * if that comes first.
 */
internal fun referenceReleaseAt(idleSince: Long, now: Long, keyboardShown: Boolean): Long {
    val kept = idleSince + 5 * 60 * 1000L
    return if (keyboardShown) kept else minOf(kept, now + 60 * 1000L)
}

/** What to say when a dictation ends without a message already shown, or null when it typed something. */
@StringRes
internal fun referenceEndingMessage(typedAny: Boolean, failures: Int): Int? = when {
    typedAny -> null
    failures > 0 -> R.string.dictation__failed
    else -> R.string.dictation__not_caught
}
