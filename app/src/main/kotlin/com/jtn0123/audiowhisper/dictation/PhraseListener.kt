package com.jtn0123.audiowhisper.dictation

import android.content.Context
import com.jtn0123.audiowhisper.rust.SherpaVad
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

/** Microphone executor only. Rust owns VAD, history, lead-in, preview schedule and stop/free. */
internal class PhraseListener(
    private val context: Context,
    onLevel: (Float) -> Unit,
    onMicError: (Int) -> Unit,
    private val onPhrase: (FloatArray) -> Unit,
    private val onPreview: (FloatArray) -> Unit = {},
    private val onSpeech: (Boolean) -> Unit = {},
    private val onModelError: (Throwable) -> Unit,
) : DictationSession.Listening {
    private var vad: SherpaVad? = null
    private val pcm = ByteBuffer.allocateDirect(com.jtn0123.audiowhisper.rust.DictationAudio.chunkSamples * Float.SIZE_BYTES).order(ByteOrder.nativeOrder())
    private val mic = MicRecorder(onLevel, onMicError) { audio ->
        try {
            val owner = vad ?: newVad().also { vad = it }
            pcm.asFloatBuffer().put(audio)
            deliver(owner.push(pcm, audio.size))
        } catch (error: Throwable) { onModelError(error) }
    }
    fun start(open: () -> MicAudio) = mic.start(open)
    override fun stop(keepLastPhrase: Boolean) {
        mic.stop() // join the producer before native flush/free, even after an AudioRecord error
        val owner = vad ?: return
        try { deliver(owner.stop(keepLastPhrase)) }
        catch (error: Throwable) { onModelError(error) }
        finally { owner.close(); vad = null }
    }
    private fun deliver(batch: SherpaVad.Batch) {
        when (batch.transition) { 1 -> onSpeech(true); -1 -> onSpeech(false) }
        batch.phrases.forEach(onPhrase)
        batch.preview?.let(onPreview)
    }
    private fun newVad(): SherpaVad {
        // App startup and memory-pressure cleanup can delete cacheDir concurrently. A private
        // temporary file survives that cleanup until synchronous native model creation finishes.
        val model = File.createTempFile("dictation-vad-", ".onnx", context.noBackupFilesDir)
        try {
            context.assets.open("silero_vad.onnx").use { input -> model.outputStream().use(input::copyTo) }
            return SherpaVad(model.path)
        } finally { model.delete() }
    }
}
