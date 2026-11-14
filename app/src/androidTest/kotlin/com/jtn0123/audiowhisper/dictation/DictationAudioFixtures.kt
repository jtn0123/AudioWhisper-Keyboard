package com.jtn0123.audiowhisper.dictation

import android.content.Context
import com.k2fsa.sherpa.onnx.GeneratedAudio
import com.k2fsa.sherpa.onnx.GenerationConfig
import com.k2fsa.sherpa.onnx.OfflineTts
import com.k2fsa.sherpa.onnx.OfflineTtsConfig
import com.k2fsa.sherpa.onnx.OfflineTtsModelConfig
import com.k2fsa.sherpa.onnx.OfflineTtsPocketModelConfig
import com.k2fsa.sherpa.onnx.WaveReader
import java.io.File
import kotlin.math.min
import kotlin.random.Random

/** Never opens AudioRecord or AudioTrack. Pocket output goes straight to private scratch WAVs. */
internal object DictationAudioFixtures {
    fun create(context: Context, directory: File): List<File> {
        directory.mkdirs()
        val voice = ModelFiles.dir(context, "sherpa-onnx-pocket-tts-int8-2026-01-26")
        fun path(name: String) = File(voice, name).path
        val pocket = OfflineTtsPocketModelConfig(lmFlow = path("lm_flow.int8.onnx"),
            lmMain = path("lm_main.int8.onnx"), encoder = path("encoder.onnx"), decoder = path("decoder.int8.onnx"),
            textConditioner = path("text_conditioner.onnx"), vocabJson = path("vocab.json"), tokenScoresJson = path("token_scores.json"))
        val tts = OfflineTts(config = OfflineTtsConfig(model = OfflineTtsModelConfig(pocket = pocket, numThreads = 4)))
        val generated = try {
            val reference = WaveReader.readWave(path("loona.wav"))
            val config = GenerationConfig(referenceAudio = reference.samples, referenceSampleRate = reference.sampleRate)
            tts.generateWithConfig("A quiet river winds past the garden. The blue boat waits beside the bridge.", config)
        } finally { tts.release() }
        // Fixed linear resampling is fixture preparation; both ASR paths read the same saved WAV.
        val ratio = generated.sampleRate / 16_000.0
        val speech = FloatArray((generated.samples.size / ratio).toInt()) { i ->
            val at = i * ratio; val first = at.toInt(); val fraction = (at - first).toFloat()
            generated.samples[first] * (1f - fraction) + generated.samples[min(first + 1, generated.samples.lastIndex)] * fraction
        }
        val rng = Random(901)
        val silence = FloatArray(16_000)
        val half = speech.size / 2
        val fixtures = listOf(
            silence + speech + silence,
            FloatArray(48_000),
            FloatArray(48_000) { (rng.nextFloat() - .5f) * .02f },
            silence + speech.copyOfRange(0, half) + FloatArray(48_000) + speech.copyOfRange(half, speech.size) + silence,
            speech.copyOfRange(0, min(1600, speech.size)),
            silence + speech.copyOfRange(0, speech.size * 3 / 4),
            silence + FloatArray(speech.size) { (speech[it] + (rng.nextFloat() - .5f) * .01f).coerceIn(-1f, 1f) } + silence,
        )
        return fixtures.mapIndexed { i, audio -> File(directory, "$i.wav").also {
            check(GeneratedAudio(audio, 16_000).save(it.path)) { "Fixture WAV write failed" }
        } }
    }
}
