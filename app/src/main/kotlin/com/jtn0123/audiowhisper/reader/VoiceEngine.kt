package com.jtn0123.audiowhisper.reader

import android.content.Context
import android.os.SystemClock
import android.util.Log
import com.jtn0123.audiowhisper.dictation.ExpectedFile
import com.jtn0123.audiowhisper.dictation.LargeModels
import com.jtn0123.audiowhisper.dictation.ModelCheck
import com.jtn0123.audiowhisper.dictation.ModelFiles
import com.jtn0123.audiowhisper.dictation.ModelState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import java.io.File
import java.nio.ByteBuffer
import com.jtn0123.audiowhisper.rust.SherpaTts

/**
 * The built-in neural voice: Kyutai's Pocket TTS (int8, through sherpa-onnx), entirely on the
 * phone, speaking in the voice of a few seconds of reference recording ("Loona"). It is copied in
 * like the dictation model ([ModelFiles]); without it, reading uses the phone's own voice.
 *
 * One sentence is generated at a time; [speak] may be called from any thread, one at a time.
 */
internal class VoiceEngine private constructor(private val tts: SherpaTts) : AutoCloseable {
    val sampleRate: Int = tts.sampleRate

    /** Borrowed PCM is consumed synchronously, before Rust releases the generation lease. */
    fun speak(text: String, onAudio: (ByteBuffer) -> Boolean) = tts.speak(text, onAudio)
    fun cancel() = tts.cancel()
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

        private fun create(dir: File): VoiceEngine = VoiceEngine(SherpaTts(dir.path))

    }
}

/** The neural voice's files, load and timer, as [LoadedVoice] holds it. */
internal class NeuralVoiceSource(private val context: Context) : VoiceSource<VoiceEngine> {
    override fun missing() = VoiceEngine.quickState(context) is ModelState.Missing

    override fun loading() = LargeModels.loading(LargeModels.Kind.VOICE)

    override suspend fun load() = withContext(Dispatchers.IO) { VoiceEngine.load(context) }

    override fun register(evict: () -> Unit) = LargeModels.register(LargeModels.Kind.VOICE, evict)

    override fun unregister() = LargeModels.unregister(LargeModels.Kind.VOICE)

    override suspend fun wait(ms: Long) = delay(ms)
}
