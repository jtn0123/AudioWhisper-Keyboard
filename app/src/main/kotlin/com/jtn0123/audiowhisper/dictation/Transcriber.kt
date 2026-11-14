package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.SherpaAsr
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

/** Android executor adapter. Native Rust owns the exact Parakeet config and C stream lifetime. */
internal class Transcriber(modelDir: File) : DictationSession.SpeechModel {
    private val recognizer = SherpaAsr(modelDir.path)
    private var pcm = ByteBuffer.allocateDirect(0).order(ByteOrder.nativeOrder())
    override fun transcribe(samples: FloatArray): String {
        if (pcm.capacity() < samples.size * Float.SIZE_BYTES) {
            pcm = ByteBuffer.allocateDirect(samples.size * Float.SIZE_BYTES).order(ByteOrder.nativeOrder())
        }
        pcm.asFloatBuffer().put(samples)
        return recognizer.transcribe(pcm, samples.size)
    }
    override fun cancel() = recognizer.cancel()
    override fun close() = recognizer.close()
    companion object { val SAMPLE_RATE: Int get() = com.jtn0123.audiowhisper.rust.DictationAudio.sampleRate }
}
