package com.jtn0123.audiowhisper.reader

import android.content.Context
import android.os.SystemClock
import android.util.Log
import com.jtn0123.audiowhisper.dictation.ExpectedFile
import com.jtn0123.audiowhisper.dictation.ModelCheck
import com.jtn0123.audiowhisper.dictation.ModelFiles
import com.jtn0123.audiowhisper.dictation.ModelState
import com.k2fsa.sherpa.onnx.GenerationConfig
import com.k2fsa.sherpa.onnx.OfflineTts
import com.k2fsa.sherpa.onnx.OfflineTtsConfig
import com.k2fsa.sherpa.onnx.OfflineTtsModelConfig
import com.k2fsa.sherpa.onnx.OfflineTtsPocketModelConfig
import com.k2fsa.sherpa.onnx.WaveReader
import java.io.File

/**
 * The built-in neural voice: Kyutai's Pocket TTS (int8, through sherpa-onnx), entirely on the
 * phone, speaking in the voice of a few seconds of reference recording ("Loona"). It is copied in
 * like the dictation model ([ModelFiles]); without it, reading uses the phone's own voice.
 *
 * One sentence is generated at a time; [speak] may be called from any thread, one at a time.
 */
internal class ReferenceVoiceEngine private constructor(
    tts: OfflineTts,
    private val config: GenerationConfig,
) : AutoCloseable {
    val sampleRate: Int = tts.sampleRate()
    private val tts = ReferenceReleasedOnce(tts) { it.release() }

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
        private const val TAG = "ReferenceVoiceEngine"

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
        fun load(context: Context): ReferenceVoiceEngine? {
            val started = SystemClock.elapsedRealtime()
            return runCatching {
                if (quickState(context) is ModelState.Missing) return@runCatching null
                val state = fullState(context)
                if (state != ModelState.Ready) {
                    Log.w(TAG, "Voice files not as expected: $state")
                    return@runCatching null
                }
                create(dir(context))
            }
                .onSuccess { if (it != null) Log.i(TAG, "Loaded in ${SystemClock.elapsedRealtime() - started} ms") }
                .onFailure { Log.w(TAG, "Could not load the voice: ${it.javaClass.simpleName}") }
                .getOrNull()
        }

        private fun create(dir: File): ReferenceVoiceEngine {
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
            return ReferenceVoiceEngine(tts, config)
        }
    }
}

