package com.jtn0123.audiowhisper.dictation

import com.k2fsa.sherpa.onnx.FeatureConfig
import com.k2fsa.sherpa.onnx.OfflineModelConfig
import com.k2fsa.sherpa.onnx.OfflineRecognizer
import com.k2fsa.sherpa.onnx.OfflineRecognizerConfig
import com.k2fsa.sherpa.onnx.OfflineTransducerModelConfig
import java.io.File

/**
 * Transcribes a whole recording once the user stops, with Parakeet TDT 0.6B
 * v2 through sherpa-onnx. On the S24 Ultra this runs at about 20x real time:
 * a 30-second dictation comes back in about 1.5 seconds.
 *
 * Not thread-safe: create, use and close it on one worker thread.
 */
class Transcriber(modelDir: File) : AutoCloseable {
    private val recognizer = OfflineRecognizer(config = config(modelDir))

    fun transcribe(samples: FloatArray): String {
        val stream = recognizer.createStream()
        try {
            stream.acceptWaveform(samples, SAMPLE_RATE)
            recognizer.decode(stream)
            return recognizer.getResult(stream).text.trim()
        } finally {
            stream.release()
        }
    }

    override fun close() = recognizer.release()

    companion object {
        const val SAMPLE_RATE = 16_000

        // Measured on the S24 Ultra: 2 to 4 threads is fastest; 6 or 8 spill
        // onto the efficiency cores and run up to 1.8x slower.
        private const val THREADS = 4

        private fun config(dir: File) = OfflineRecognizerConfig(
            featConfig = FeatureConfig(sampleRate = SAMPLE_RATE, featureDim = 80),
            modelConfig = OfflineModelConfig(
                transducer = OfflineTransducerModelConfig(
                    encoder = File(dir, ModelFiles.ENCODER).path,
                    decoder = File(dir, ModelFiles.DECODER).path,
                    joiner = File(dir, ModelFiles.JOINER).path,
                ),
                tokens = File(dir, ModelFiles.TOKENS).path,
                numThreads = THREADS,
                modelType = "nemo_transducer",
            ),
            decodingMethod = "greedy_search",
        )
    }
}
