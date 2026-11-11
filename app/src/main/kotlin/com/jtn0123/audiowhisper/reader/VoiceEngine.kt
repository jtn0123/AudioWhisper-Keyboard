package com.jtn0123.audiowhisper.reader

import android.content.Context
import android.os.SystemClock
import android.util.Log
import com.jtn0123.audiowhisper.dictation.ModelFiles
import com.k2fsa.sherpa.onnx.GenerationConfig
import com.k2fsa.sherpa.onnx.OfflineTts
import com.k2fsa.sherpa.onnx.OfflineTtsConfig
import com.k2fsa.sherpa.onnx.OfflineTtsModelConfig
import com.k2fsa.sherpa.onnx.OfflineTtsPocketModelConfig
import com.k2fsa.sherpa.onnx.WaveReader
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.io.File

/**
 * The built-in neural voice: Kyutai's Pocket TTS (int8, through sherpa-onnx), entirely on the
 * phone, speaking in the voice of a few seconds of reference recording ("Loona"). It is copied in
 * like the dictation model ([ModelFiles]); without it, reading uses the phone's own voice.
 *
 * One sentence is generated at a time; [speak] may be called from any thread, one at a time.
 */
internal class VoiceEngine private constructor(
    tts: OfflineTts,
    private val config: GenerationConfig,
) : AutoCloseable {
    val sampleRate: Int = tts.sampleRate()
    private val tts = ReleasedOnce(tts) { it.release() }

    /**
     * Speaks [text], handing its audio to [onAudio] as it is made; generation stops early when
     * [onAudio] returns false. Does nothing once the voice is [close]d.
     */
    fun speak(text: String, onAudio: (FloatArray) -> Boolean) {
        tts.use { it.generateWithConfigAndCallback(text, config, AudioCallback(onAudio)) }
    }

    /**
     * What sherpa-onnx's native code calls with each piece of audio: it looks the method up by its
     * exact signature, invoke([F)Ljava/lang/Integer;, which a Kotlin lambda doesn't have (it
     * compiles to the erased invoke(Object)). Kept by name in proguard-rules.pro.
     */
    class AudioCallback(private val onAudio: (FloatArray) -> Boolean) : (FloatArray) -> Int {
        // 1 asks for more audio, 0 stops generating.
        override fun invoke(samples: FloatArray): Int = if (onAudio(samples)) 1 else 0
    }

    /** Frees the voice once the sentence being made, if any, is done. */
    override fun close() = tts.close()

    companion object {
        const val POCKET = "sherpa-onnx-pocket-tts-int8-2026-01-26"
        const val REFERENCE = "loona.wav"
        private const val LM_FLOW = "lm_flow.int8.onnx"
        private const val LM_MAIN = "lm_main.int8.onnx"
        private const val ENCODER = "encoder.onnx"
        private const val DECODER = "decoder.int8.onnx"
        private const val TEXT_CONDITIONER = "text_conditioner.onnx"
        private const val VOCAB = "vocab.json"
        private const val TOKEN_SCORES = "token_scores.json"
        private val REQUIRED = listOf(LM_FLOW, LM_MAIN, ENCODER, DECODER, TEXT_CONDITIONER, VOCAB, TOKEN_SCORES, REFERENCE)

        // As for dictation: 4 threads keep to the S24 Ultra's performance cores.
        private const val THREADS = 4
        private const val TAG = "VoiceEngine"

        fun dir(context: Context): File = ModelFiles.dir(context, POCKET)

        fun isInstalled(context: Context): Boolean = REQUIRED.all { File(dir(context), it).isFile }

        /** Loads the voice; null if it isn't installed or fails to load. Takes a second or two. */
        fun load(context: Context): VoiceEngine? {
            if (!isInstalled(context)) return null
            val started = SystemClock.elapsedRealtime()
            return runCatching { create(dir(context)) }
                .onSuccess { Log.i(TAG, "Loaded in ${SystemClock.elapsedRealtime() - started} ms") }
                .onFailure { Log.w(TAG, "Could not load the voice: ${it.javaClass.simpleName}") }
                .getOrNull()
        }

        private fun create(dir: File): VoiceEngine {
            fun path(name: String) = File(dir, name).path
            val pocket = OfflineTtsPocketModelConfig(
                lmFlow = path(LM_FLOW),
                lmMain = path(LM_MAIN),
                encoder = path(ENCODER),
                decoder = path(DECODER),
                textConditioner = path(TEXT_CONDITIONER),
                vocabJson = path(VOCAB),
                tokenScoresJson = path(TOKEN_SCORES),
            )
            val tts = OfflineTts(config = OfflineTtsConfig(model = OfflineTtsModelConfig(pocket = pocket, numThreads = THREADS)))
            val reference = WaveReader.readWave(path(REFERENCE))
            val config = GenerationConfig(referenceAudio = reference.samples, referenceSampleRate = reference.sampleRate)
            return VoiceEngine(tts, config)
        }
    }
}

/**
 * Keeps the neural voice loaded between readings, so only the first waits for it, and frees it
 * (a few hundred MB) once [IDLE_MS] pass without one.
 */
internal class LoadedVoice(private val context: Context, private val scope: CoroutineScope) {
    private val mutex = Mutex()
    private var engine: VoiceEngine? = null
    private var release: Job? = null
    private var inUse = false

    /** The voice, loading it if need be; null if it isn't installed or won't load. */
    suspend fun take(): VoiceEngine? = mutex.withLock {
        inUse = true
        engine ?: withContext(Dispatchers.IO) { VoiceEngine.load(context) }.also { engine = it }
    }

    /** A reading is over: free the voice unless another starts soon. */
    suspend fun idle() {
        mutex.withLock { inUse = false }
        release?.cancel()
        release = scope.launch {
            delay(IDLE_MS)
            mutex.withLock { if (!inUse) free() }
        }
    }

    suspend fun close() = mutex.withLock { free() }

    private fun free() {
        engine?.close()
        engine = null
    }

    private companion object {
        const val IDLE_MS = 3 * 60_000L
    }
}
