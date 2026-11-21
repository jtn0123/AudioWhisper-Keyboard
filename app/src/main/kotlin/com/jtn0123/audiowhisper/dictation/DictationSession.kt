package com.jtn0123.audiowhisper.dictation

import androidx.annotation.StringRes
import com.jtn0123.audiowhisper.rust.*
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
    private val unload = Runnable { event(DictationEvent.UNLOAD) }

    fun pressMic() = event(DictationEvent.PRESS)
    fun releaseMic() = event(DictationEvent.RELEASE_MIC)
    fun stopListening() = event(DictationEvent.STOP)
    fun abandon() = event(DictationEvent.CANCEL)
    fun keyboardVisible(visible: Boolean) = event(DictationEvent.VISIBLE, fields = arrayOf(WireFlag.encode(visible)))
    fun release() { event(DictationEvent.RELEASE); released = true }
    fun freeIfIdle() { post { event(DictationEvent.FREE) } }

    private fun post(action: Runnable) { platform.post { if (!released) action.run() } }

    /** Executes effects only. State, generation acceptance and scheduling are native decisions. */
    private fun event(operation: DictationEvent, id: Int = 0, fields: Array<String> = emptyArray()) {
        val result = plan.call(operation, platform.uptimeMs(), id, *fields)
        if (result.isEmpty()) return
        // A flush can synchronously enqueue phrase work; publish UI after its ordered effects.
        var i = DictationPlan.SIZE
        while (i < result.size) {
            when (val effect = result[i++]) {
                DictationEffect.CHECK -> {
                    val model = when (platform.modelState()) {
                        ModelState.Ready -> DictationModelAvailability.READY
                        is ModelState.Missing -> DictationModelAvailability.MISSING
                        is ModelState.Damaged -> DictationModelAvailability.DAMAGED
                    }
                    val ready = with(DictationReady) { row(ALLOWED to WireFlag.encode(editor.allowsDictation()),
                        PERMISSION to WireFlag.encode(platform.hasMicPermission()), MODEL to model) }
                    event(DictationEvent.READY, fields = ready)
                    return
                }
                DictationEffect.PROBLEM -> editor.problem(problem(result[i++]))
                DictationEffect.APP_SETTINGS -> platform.openAppSettings()
                DictationEffect.MODEL_SETTINGS -> platform.openModelSettings()
                DictationEffect.LISTEN -> {
                    val generation = result[i++].toInt()
                    try { listener = platform.listen(callbacks(generation)) }
                    catch (error: IllegalStateException) {
                        platform.log(LogLevel.ERROR, "Could not start recording", error)
                        event(DictationEvent.OPEN_ERROR); return
                    }
                    event(DictationEvent.OPENED); return
                }
                DictationEffect.PRELOAD -> {
                    platform.removeCallbacks(unload)
                    preload(result[i++].toInt())
                }
                DictationEffect.DISCARD -> { activeModel.get()?.cancel(); listener?.stop(false); listener = null }
                DictationEffect.FLUSH -> { listener?.stop(true); listener = null }
                DictationEffect.BARRIER -> {
                    val generation = result[i++].toInt()
                    platform.work { post { event(DictationEvent.FINALIZING, generation); event(DictationEvent.FINISHED, generation) } }
                }
                DictationEffect.SCHEDULE -> {
                    platform.removeCallbacks(unload)
                    platform.postAtTime(unload, result[i++].toLong())
                }
                DictationEffect.REMOVE_UNLOAD -> platform.removeCallbacks(unload)
                DictationEffect.UNLOAD -> platform.work(::closeModel)
                DictationEffect.CLOSE_WORKER -> { platform.work(::closeModel); platform.shutDownWorker() }
                DictationEffect.TICK -> editor.tick()
                DictationEffect.TIDY -> { event(DictationEvent.TIDIED, fields = arrayOf(editor.tidy(result[i++]))); return }
                DictationEffect.NAMES -> {
                    val count = result[i++].toInt()
                    val names = Array(count) { WireFlag.encode(editor.isName(result[i++])) }
                    event(DictationEvent.COMPOSE, fields = arrayOf(editor.textBeforeCursor()?.toString() ?: "", *names)); return
                }
                DictationEffect.INSERT -> { val replacing = result[i++].toInt(); editor.type(result[i++], replacing) }
                DictationEffect.UNDO -> editor.undoLastPhrase()
                DictationEffect.RESUME -> { event(DictationEvent.COMPOSE, fields = arrayOf(editor.textBeforeCursor()?.toString() ?: "")); return }
                else -> error("Unknown dictation effect: $effect")
            }
        }
        flows.status.value = Dictation.Status.entries[result[DictationPlan.STATUS].toInt()]
        flows.warming.value = WireFlag.decode(result[DictationPlan.WARMING])
        flows.level.value = result[DictationPlan.LEVEL].toFloat()
        flows.preview.value = result[DictationPlan.PREVIEW]
    }

    private fun preload(id: Int) = platform.work {
        models.retry()
        val loading = !models.isLoaded
        if (loading) post { event(DictationEvent.WARMING, id) }
        try { models.get() }
        catch (error: ModelLoadException) {
            platform.log(LogLevel.ERROR, "Could not load the model", error)
            post { event(DictationEvent.MODEL_ERROR, id) }
        } finally { if (loading) post { event(DictationEvent.WARMED, id) } }
    }

    private fun callbacks(id: Int) = Callbacks(
        onLevel = { level -> post { event(DictationEvent.LEVEL, id, arrayOf(level.toString())) } },
        onMicError = { code ->
            platform.log(LogLevel.ERROR, "Microphone failed while recording: $code")
            post { event(DictationEvent.MIC_ERROR, id) }
        },
        onPhrase = { phrase -> if (!released) platform.work {
            val result = runCatching { timed(phrase) }
            post {
                result.exceptionOrNull()?.let { platform.log(LogLevel.ERROR, "Transcription failed", it) }
                event(DictationEvent.FINAL, id, arrayOf(WireFlag.encode(result.isSuccess), result.getOrDefault("")))
            }
        } },
        onPreview = { phrase -> preview(id, phrase) },
    ).also { callbacks ->
        callbacks.onSpeech = { speech -> plan.speech(id, speech) }
        callbacks.onModelError = { error ->
            platform.log(LogLevel.ERROR, "Speech model failed", error)
            post { event(DictationEvent.MODEL_ERROR, id) }
        }
    }

    private fun preview(id: Int, phrase: FloatArray) {
        if (!plan.claimPreview()) return
        platform.work {
            val started = platform.uptimeMs()
            val text = try { runCatching { models.get().transcribe(phrase) }.getOrNull() }
            finally { plan.completePreview() }
            platform.log(LogLevel.DEBUG, "Previewed ${phrase.size} samples in ${platform.uptimeMs() - started} ms")
            if (text != null) post { event(DictationEvent.PREVIEW, id, arrayOf(text)) }
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
    DictationProblem.FIELD_OFF -> R.string.dictation__field_off
    DictationProblem.ALLOW_MIC -> R.string.dictation__allow_mic
    DictationProblem.NOT_INSTALLED -> R.string.dictation__not_installed
    DictationProblem.DAMAGED -> R.string.dictation__damaged
    DictationProblem.MIC_BUSY -> R.string.dictation__mic_busy
    DictationProblem.MODEL_FAILED -> R.string.dictation__model_failed
    DictationProblem.MIC_STOPPED -> R.string.dictation__mic_stopped
    DictationProblem.FAILED -> R.string.dictation__failed
    DictationProblem.NOT_CAUGHT -> R.string.dictation__not_caught
    else -> error("Unknown dictation problem code")
}
