package com.jtn0123.audiowhisper.dictation

import android.content.res.AssetManager
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
 * the caller's thread from [stop]. [onMicError] is called on the
 * microphone thread if the mic fails; listening has stopped, and [stop]
 * still hands over the words heard until then. Single use.
 *
 * Pauses are found by Silero VAD (MIT licence), bundled in assets.
 */
class PhraseListener(
    assets: AssetManager,
    onLevel: (Float) -> Unit,
    onMicError: (Int) -> Unit,
    private val onPhrase: (FloatArray) -> Unit,
) {
    // Used by the microphone thread, then by stop() after that thread has
    // ended, never by both at once.
    private val vad = Vad(assets, VadModelConfig(sileroVadModelConfig = SILERO, sampleRate = Transcriber.SAMPLE_RATE))
    private val history = AudioHistory(HISTORY_SAMPLES)
    private val mic = MicRecorder(onLevel, onMicError) { audio ->
        history.add(audio)
        vad.acceptWaveform(audio)
        drain()
    }

    /** Starts listening to the audio [open] opens; throws IllegalStateException if it can't. */
    fun start(open: () -> MicAudio) {
        try {
            mic.start(open)
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
            val segment = vad.front()
            onPhrase(history.slice(segment.start - LEAD_IN_SAMPLES, segment.start) + segment.samples)
            vad.pop()
        }
    }

    private companion object {
        val SILERO = SileroVadModelConfig(
            model = "silero_vad.onnx",
            // Long enough that a breath mid-sentence doesn't split it.
            minSilenceDuration = MIN_SILENCE_SECONDS,
            minSpeechDuration = 0.25f,
            // Past this, the VAD splits at the next brief dip instead of
            // waiting for a real pause, so no phrase grows without bound.
            maxSpeechDuration = MAX_SPEECH_SECONDS,
        )
        const val MIN_SILENCE_SECONDS = 0.6f
        const val MAX_SPEECH_SECONDS = 20f

        // The VAD starts a phrase late enough to cut a short first word ("I", the "J" of
        // "John"); 0.3 s brought them back in testing. It stays inside the pause before, which
        // is at least MIN_SILENCE_SECONDS, so it never repeats the end of the last phrase.
        const val LEAD_IN_SAMPLES = Transcriber.SAMPLE_RATE * 3 / 10

        // Back as far as the start of the longest phrase, with room to spare.
        const val HISTORY_SAMPLES = Transcriber.SAMPLE_RATE * (MAX_SPEECH_SECONDS.toInt() + 5)
    }
}
