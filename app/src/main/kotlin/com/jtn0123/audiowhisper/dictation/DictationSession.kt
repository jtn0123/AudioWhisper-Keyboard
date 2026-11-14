package com.jtn0123.audiowhisper.dictation

import androidx.annotation.StringRes
import com.jtn0123.audiowhisper.rust.DictationSessionPlan
import dev.patrickgold.florisboard.R
import java.util.concurrent.atomic.AtomicReference
import kotlinx.coroutines.flow.MutableStateFlow

/**
 * What the mic button and the smartbar show of dictation. One per process in the app
 * ([Dictation.status] and the rest); a test makes its own.
 */
internal class DictationFlows {
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
internal class DictationSession(
    private val editor: Dictation.Editor,
    private val platform: Platform,
    private val flows: DictationFlows,
) {
    /** The Android side: the main thread's queue, the worker, the clock, the mic and the model. */
    interface Platform {
        /** Runs [action] on the main thread, after what is queued there. */
        fun post(action: Runnable)
        fun postAtTime(action: Runnable, uptimeMs: Long)
        fun removeCallbacks(action: Runnable)
        fun uptimeMs(): Long

        /** Runs [action] on the worker thread, after what was handed to it before. */
        fun work(action: Runnable)
        fun shutDownWorker()

        fun hasMicPermission(): Boolean

        /** Opens this app's page in Android's settings, where the mic can be allowed. */
        fun openAppSettings()

        /** Whether the model's files are all there, by size; see [ModelCheck.quick]. */
        fun modelState(): ModelState

        /** Opens Settings › Dictation, which says how to copy the model in. */
        fun openModelSettings()

        /** Starts listening; throws IllegalStateException if the mic won't open. */
        fun listen(callbacks: Callbacks): Listening

        /** Loads the speech model, on the worker. Throws if it can't. */
        fun loadModel(): SpeechModel
        fun log(level: LogLevel, message: String, error: Throwable? = null)
    }

    /** What [PhraseListener] reports, on the microphone thread (see there). */
    class Callbacks(
        val onLevel: (Float) -> Unit,
        val onMicError: (Int) -> Unit,
        val onPhrase: (FloatArray) -> Unit,
        val onPreview: (FloatArray) -> Unit,
    ) {
        var onSpeech: (Boolean) -> Unit = {}
        var onModelError: (Throwable) -> Unit = {}
    }

    interface Listening {
        /** Stops listening. With [keepLastPhrase], the words since the last pause are handed over too. */
        fun stop(keepLastPhrase: Boolean)
    }

    interface SpeechModel : AutoCloseable {
        fun transcribe(samples: FloatArray): String
        fun cancel() = Unit
    }

    enum class LogLevel { DEBUG, INFO, WARN, ERROR }

    private val plan = DictationSessionPlan()
    private val models = ModelHolder(policy = plan, load = ::loadModel) // worker executor only
    private var listener: Listening? = null
    private val activeModel = AtomicReference<SpeechModel?>()
    @Volatile private var released = false
    private val unload = Runnable { event("unload") }

    fun pressMic() = event("press")
    fun releaseMic() = event("releaseMic")
    fun stopListening() = event("stop")
    fun abandon() = event("cancel")
    fun keyboardVisible(visible: Boolean) = event("visible", fields = arrayOf(visible.toString()))
    fun release() { event("release"); released = true }
    fun freeIfIdle() { post { event("free") } }

    private fun post(action: Runnable) { platform.post { if (!released) action.run() } }

    /** Executes effects only. State, generation acceptance and scheduling are native decisions. */
    private fun event(operation: String, id: Int = 0, fields: Array<String> = emptyArray()) {
        val result = plan.call(operation, platform.uptimeMs(), id, *fields)
        if (result.isEmpty()) return
        // A flush can synchronously enqueue phrase work; publish UI after its ordered effects.
        var i = 6
        while (i < result.size) {
            when (val effect = result[i++]) {
                "check" -> {
                    val allowed = editor.allowsDictation()
                    val permission = platform.hasMicPermission()
                    val model = when (platform.modelState()) {
                        ModelState.Ready -> "ready"
                        is ModelState.Missing -> "missing"
                        is ModelState.Damaged -> "damaged"
                    }
                    event("ready", fields = arrayOf(allowed.toString(), permission.toString(), model))
                    return
                }
                "problem" -> editor.problem(problem(result[i++]))
                "appSettings" -> platform.openAppSettings()
                "modelSettings" -> platform.openModelSettings()
                "listen" -> {
                    val generation = result[i++].toInt()
                    try { listener = platform.listen(callbacks(generation)) }
                    catch (error: IllegalStateException) {
                        platform.log(LogLevel.ERROR, "Could not start recording", error)
                        event("openError"); return
                    }
                    event("opened"); return
                }
                "preload" -> {
                    platform.removeCallbacks(unload)
                    preload(result[i++].toInt())
                }
                "discard" -> { activeModel.get()?.cancel(); listener?.stop(false); listener = null }
                "flush" -> { listener?.stop(true); listener = null }
                "barrier" -> {
                    val generation = result[i++].toInt()
                    platform.work { post { event("finalizing", generation); event("finished", generation) } }
                }
                "schedule" -> {
                    platform.removeCallbacks(unload)
                    platform.postAtTime(unload, result[i++].toLong())
                }
                "removeUnload" -> platform.removeCallbacks(unload)
                "unload" -> platform.work(::closeModel)
                "closeWorker" -> { platform.work(::closeModel); platform.shutDownWorker() }
                "tick" -> editor.tick()
                "tidy" -> { event("tidied", fields = arrayOf(editor.tidy(result[i++]))); return }
                "names" -> {
                    val count = result[i++].toInt()
                    val names = Array(count) { editor.isName(result[i++]).toString() }
                    event("compose", fields = arrayOf(editor.textBeforeCursor()?.toString() ?: "", *names)); return
                }
                "insert" -> { val replacing = result[i++].toInt(); editor.type(result[i++], replacing) }
                "undo" -> editor.undoLastPhrase()
                "resume" -> { event("compose", fields = arrayOf(editor.textBeforeCursor()?.toString() ?: "")); return }
                else -> error("Unknown dictation effect: $effect")
            }
        }
        flows.status.value = Dictation.Status.entries[result[0].toInt()]
        flows.warming.value = result[1].toBooleanStrict()
        flows.level.value = result[2].toFloat()
        flows.preview.value = result[3]
    }

    private fun preload(id: Int) = platform.work {
        models.retry()
        val loading = !models.isLoaded
        if (loading) post { event("warming", id) }
        try { models.get() }
        catch (error: ModelLoadException) {
            platform.log(LogLevel.ERROR, "Could not load the model", error)
            post { event("modelError", id) }
        } finally { if (loading) post { event("warmed", id) } }
    }

    private fun callbacks(id: Int) = Callbacks(
        onLevel = { level -> post { event("level", id, arrayOf(level.toString())) } },
        onMicError = { code ->
            platform.log(LogLevel.ERROR, "Microphone failed while recording: $code")
            post { event("micError", id) }
        },
        onPhrase = { phrase -> if (!released) platform.work {
            val result = runCatching { timed(phrase) }
            post {
                result.exceptionOrNull()?.let { platform.log(LogLevel.ERROR, "Transcription failed", it) }
                event("final", id, arrayOf(result.isSuccess.toString(), result.getOrDefault("")))
            }
        } },
        onPreview = { phrase -> preview(id, phrase) },
    ).also { callbacks ->
        callbacks.onSpeech = { speech -> plan.speech(id, speech) }
        callbacks.onModelError = { error ->
            platform.log(LogLevel.ERROR, "Speech model failed", error)
            post { event("modelError", id) }
        }
    }

    private fun preview(id: Int, phrase: FloatArray) {
        if (!plan.claimPreview()) return
        platform.work {
            val started = platform.uptimeMs()
            val text = try { runCatching { models.get().transcribe(phrase) }.getOrNull() }
            finally { plan.completePreview() }
            platform.log(LogLevel.DEBUG, "Previewed ${phrase.size} samples in ${platform.uptimeMs() - started} ms")
            if (text != null) post { event("preview", id, arrayOf(text)) }
        }
    }

    private fun timed(audio: FloatArray): String {
        val started = platform.uptimeMs()
        return models.get().transcribe(audio).also {
            platform.log(LogLevel.INFO, "Transcribed ${audio.size} samples in ${platform.uptimeMs() - started} ms")
        }
    }
    private fun loadModel(): SpeechModel {
        val started = platform.uptimeMs()
        return platform.loadModel().also { activeModel.set(it); platform.log(LogLevel.INFO, "Model loaded in ${platform.uptimeMs() - started} ms") }
    }
    private fun closeModel() { activeModel.set(null); if (models.close()) platform.log(LogLevel.INFO, "Model released") }
}

internal fun releaseAt(idleSince: Long, now: Long, keyboardShown: Boolean): Long =
    DictationSessionPlan.releaseAt(idleSince, now, keyboardShown)

@StringRes internal fun endingMessage(typedAny: Boolean, failures: Int): Int? =
    DictationSessionPlan.ending(typedAny, failures).takeIf { it.isNotEmpty() }?.let(::problem)

@StringRes private fun problem(code: String): Int = when (code) {
    "field_off" -> R.string.dictation__field_off
    "allow_mic" -> R.string.dictation__allow_mic
    "not_installed" -> R.string.dictation__not_installed
    "damaged" -> R.string.dictation__damaged
    "mic_busy" -> R.string.dictation__mic_busy
    "model_failed" -> R.string.dictation__model_failed
    "mic_stopped" -> R.string.dictation__mic_stopped
    "failed" -> R.string.dictation__failed
    "not_caught" -> R.string.dictation__not_caught
    else -> error("Unknown dictation problem code")
}
