package com.jtn0123.audiowhisper.dictation

import android.Manifest
import android.content.res.AssetManager
import androidx.annotation.RequiresPermission
import com.k2fsa.sherpa.onnx.SileroVadModelConfig
import com.k2fsa.sherpa.onnx.Vad
import com.k2fsa.sherpa.onnx.VadModelConfig

/**
 * Listens to the microphone and cuts what it hears into phrases at each
 * pause, so every phrase can be transcribed and typed while the user is
 * still talking. That keeps the wait after the last word short, and memory
 * flat however long the dictation runs.
 *
 * [onPhrase] is called on the microphone thread while listening, and on
 * the caller's thread from [stop]. Single use.
 *
 * Pauses are found by Silero VAD (MIT licence), bundled in assets.
 */
class PhraseListener(
    assets: AssetManager,
    onLevel: (Float) -> Unit,
    private val onPhrase: (FloatArray) -> Unit,
) {
    // Used by the microphone thread, then by stop() after that thread has
    // ended, never by both at once.
    private val vad = Vad(assets, VadModelConfig(sileroVadModelConfig = SILERO, sampleRate = Transcriber.SAMPLE_RATE))
    private val mic = MicRecorder(onLevel) { audio ->
        vad.acceptWaveform(audio)
        drain()
    }

    @RequiresPermission(Manifest.permission.RECORD_AUDIO)
    fun start() {
        try {
            mic.start()
        } catch (e: IllegalStateException) {
            vad.release()
            throw e
        }
    }

    /** Stops listening. With [keepLastPhrase], the words since the last pause are handed over too. */
    fun stop(keepLastPhrase: Boolean) {
        mic.stop()
        if (keepLastPhrase) {
            vad.flush()
            drain()
        }
        vad.release()
    }

    private fun drain() {
        while (!vad.empty()) {
            onPhrase(vad.front().samples)
            vad.pop()
        }
    }

    private companion object {
        val SILERO = SileroVadModelConfig(
            model = "silero_vad.onnx",
            // Long enough that a breath mid-sentence doesn't split it.
            minSilenceDuration = 0.6f,
            minSpeechDuration = 0.25f,
            // Past this, the VAD splits at the next brief dip instead of
            // waiting for a real pause, so no phrase grows without bound.
            maxSpeechDuration = 20f,
        )
    }
}
