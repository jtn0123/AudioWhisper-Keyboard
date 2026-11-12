package com.jtn0123.audiowhisper.reader

import android.content.Context
import android.os.SystemClock
import android.util.Log
import com.jtn0123.audiowhisper.dictation.ExpectedFile
import com.jtn0123.audiowhisper.dictation.LargeModels
import com.jtn0123.audiowhisper.dictation.ModelCheck
import com.jtn0123.audiowhisper.dictation.ModelFiles
import com.jtn0123.audiowhisper.dictation.ModelState
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

        /** sherpa-onnx's tts-models release, and loona.wav from its test_wavs. */
        private val EXPECTED = listOf(
            ExpectedFile(LM_FLOW, 9_962_530, "8d627d235c44a597da908e1085ebe241cbbe358964c502c5a5063d18851a5529"),
            ExpectedFile(LM_MAIN, 76_341_079, "bfc0c7e7e3d72864fa3bb2ee499f62f21ddc1474b885f5f3ca570f8be73e787e"),
            ExpectedFile(ENCODER, 72_713_165, "e8f2f6d301ffb96e398b138a7dc6d3038622d236044636b73d920bab85890260"),
            ExpectedFile(DECODER, 22_693_618, "12b0857402d31aead94df19d6783b4350d1f740e811f3a3202c70ad89ae11eea"),
            ExpectedFile(TEXT_CONDITIONER, 16_388_343, "0b84e837d7bfaf2c896627b03e3f080320309f37f4fc7df7698c644f7ba5e6b1"),
            ExpectedFile(VOCAB, 69_478, "6fb646346cf931016f70c4921aab0900ce7a304b893cb02135c74e294abfea01"),
            ExpectedFile(TOKEN_SCORES, 123_616, "5be2f278caf9b9800741f0fd82bff677f4943ec764c356f907213434b622d958"),
            ExpectedFile(REFERENCE, 50_478, "82bde825fe9ad43b3d7fa2b429a7e7acc9bcda2d3a208546e2db4863043ec64c"),
        )

        // As for dictation: 4 threads keep to the S24 Ultra's performance cores.
        private const val THREADS = 4
        private const val TAG = "VoiceEngine"

        fun dir(context: Context): File = ModelFiles.dir(context, POCKET)

        /** Names and sizes only, so cheap enough for the main thread. */
        fun quickState(context: Context): ModelState = ModelCheck.quick(dir(context), EXPECTED)

        /** Also checks each file's SHA-256, once per file. Blocking: off the main thread only. */
        fun fullState(context: Context): ModelState =
            ModelCheck.full(dir(context), EXPECTED, ModelCheck.verifiedFiles(context))

        /**
         * Loads the voice; null if it isn't installed or fails to load. Takes a second or two, and
         * the first time, a second more to check the files' SHA-256.
         */
        fun load(context: Context): VoiceEngine? {
            if (quickState(context) is ModelState.Missing) return null
            val state = fullState(context)
            if (state != ModelState.Ready) {
                Log.w(TAG, "Voice files not as expected: $state")
                return null
            }
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

    /**
     * The voice is installed but wouldn't load (damaged, or not enough memory). Remembered until
     * the service restarts, so each reading doesn't retry a load of hundreds of MB first.
     */
    private var failed = false
    private var justFailed = false

    init {
        LargeModels.register(LargeModels.Kind.VOICE) { scope.launch { mutex.withLock { if (!inUse) free() } } }
    }

    /** The voice, loading it if need be; null if it isn't installed or won't load. */
    suspend fun take(): VoiceEngine? = mutex.withLock {
        inUse = true
        engine?.let { return@withLock it }
        if (failed || VoiceEngine.quickState(context) is ModelState.Missing) return@withLock null
        LargeModels.loading(LargeModels.Kind.VOICE)
        withContext(Dispatchers.IO) { VoiceEngine.load(context) }
            .also { engine = it; failed = it == null; justFailed = failed }
    }

    /** Whether the last [take] found the voice won't load; true once per failure, to say so once. */
    suspend fun failedJustNow(): Boolean = mutex.withLock { justFailed.also { justFailed = false } }

    /** A reading is over: free the voice unless another starts soon. */
    suspend fun idle() {
        mutex.withLock { inUse = false }
        release?.cancel()
        release = scope.launch {
            delay(IDLE_MS)
            mutex.withLock { if (!inUse) free() }
        }
    }

    suspend fun close() {
        LargeModels.unregister(LargeModels.Kind.VOICE)
        mutex.withLock { free() }
    }

    private fun free() {
        engine?.close()
        engine = null
    }

    private companion object {
        const val IDLE_MS = 3 * 60_000L
    }
}
