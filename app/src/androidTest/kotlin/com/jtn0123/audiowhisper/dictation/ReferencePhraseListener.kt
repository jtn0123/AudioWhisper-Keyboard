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
 * the caller's thread from [stop]. [onPreview] gets the phrase still being
 * spoken, about once a second, on the microphone thread, for a preview of
 * its words; it always comes after the [onPhrase] of the phrase before. [onMicError] is called on the
 * microphone thread if the mic fails; listening has stopped, and [stop]
 * still hands over the words heard until then. Single use.
 *
 * Pauses are found by Silero VAD (MIT licence), bundled in assets.
 */
internal class ReferencePhraseListener(
    private val assets: AssetManager,
    onLevel: (Float) -> Unit,
    onMicError: (Int) -> Unit,
    private val onPhrase: (FloatArray) -> Unit,
    private val onPreview: (FloatArray) -> Unit = {},
) : DictationSession.Listening {
    // Used by the microphone thread, then by stop() after that thread has
    // ended, never by both at once. Made by that thread on its first audio, so
    // the tap that starts dictation doesn't wait the 10-30 ms it takes; the
    // mic's half-second buffer holds what is said meanwhile.
    private var vad: Vad? = null
    private val history = ReferenceAudioHistory(HISTORY_SAMPLES)
    fun push(audio: FloatArray): Boolean {
        val vad = this.vad ?: newVad().also { this.vad = it }
        history.add(audio)
        vad.acceptWaveform(audio)
        drain(vad)
        preview(vad)
        return vad.isSpeechDetected()
    }
    val boundaries = mutableListOf<Long>()

    /** Where the phrase being spoken starts, roughly, or null between phrases. */
    private var speechFrom: Long? = null
    private var nextPreviewAt = 0L
    private var lastPhraseEnd = 0L

    /** Stops listening. With [keepLastPhrase], the words since the last pause are handed over too. */
    override fun stop(keepLastPhrase: Boolean) {
        val vad = this.vad ?: return
        if (keepLastPhrase) {
            vad.flush()
            drain(vad)
        }
        vad.release()
    }

    private fun newVad() = Vad(assets, VadModelConfig(sileroVadModelConfig = SILERO, sampleRate = ReferenceTranscriber.SAMPLE_RATE))

    private fun drain(vad: Vad) {
        while (!vad.empty()) {
            val segment = vad.front()
            boundaries.add(segment.start)
            boundaries.add(segment.samples.size.toLong())
            onPhrase(history.slice(segment.start - LEAD_IN_SAMPLES, segment.start) + segment.samples)
            lastPhraseEnd = segment.start + segment.samples.size
            speechFrom = null
            vad.pop()
        }
    }

    /** Hands over the phrase in progress, at most its last [PREVIEW_MAX_SAMPLES], every [PREVIEW_EVERY_SAMPLES]. */
    private fun preview(vad: Vad) {
        if (!vad.isSpeechDetected()) {
            speechFrom = null
            return
        }
        val now = history.end
        val from = speechFrom ?: maxOf(lastPhraseEnd, now - DETECTION_DELAY_SAMPLES - LEAD_IN_SAMPLES).also {
            speechFrom = it
            nextPreviewAt = now + PREVIEW_EVERY_SAMPLES / 2
        }
        if (now < nextPreviewAt) return
        nextPreviewAt = now + PREVIEW_EVERY_SAMPLES
        onPreview(history.slice(maxOf(from, now - PREVIEW_MAX_SAMPLES), now))
    }

    private companion object {
        val SILERO = SileroVadModelConfig(
            model = "silero_vad.onnx",
            // Long enough that a breath mid-sentence doesn't split it.
            minSilenceDuration = MIN_SILENCE_SECONDS,
            minSpeechDuration = MIN_SPEECH_SECONDS,
            // Past this, the VAD splits at the next brief dip instead of
            // waiting for a real pause, so no phrase grows without bound.
            maxSpeechDuration = MAX_SPEECH_SECONDS,
        )
        const val MIN_SILENCE_SECONDS = 0.6f
        const val MIN_SPEECH_SECONDS = 0.25f
        const val MAX_SPEECH_SECONDS = 20f

        // The VAD starts a phrase late enough to cut a short first word ("I", the "J" of
        // "John"); 0.3 s brought them back in testing. It stays inside the pause before, which
        // is at least MIN_SILENCE_SECONDS, so it never repeats the end of the last phrase.
        const val LEAD_IN_SAMPLES = ReferenceTranscriber.SAMPLE_RATE * 3 / 10

        // The VAD reports speech once it has lasted MIN_SPEECH_SECONDS, judged a 512-sample
        // window at a time, so the phrase began about this long before.
        const val DETECTION_DELAY_SAMPLES = (ReferenceTranscriber.SAMPLE_RATE * MIN_SPEECH_SECONDS).toInt() + 512

        // Previews about once a second; each covers at most the last 10 s, which bounds the
        // work (the strip shows only the latest words anyway).
        const val PREVIEW_EVERY_SAMPLES = ReferenceTranscriber.SAMPLE_RATE
        const val PREVIEW_MAX_SAMPLES = ReferenceTranscriber.SAMPLE_RATE * 10

        // Back as far as the start of the longest phrase, with room to spare.
        const val HISTORY_SAMPLES = ReferenceTranscriber.SAMPLE_RATE * (MAX_SPEECH_SECONDS.toInt() + 5)
    }
}
