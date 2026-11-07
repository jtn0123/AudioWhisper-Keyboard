package com.jtn0123.audiowhisper.dictation

import kotlin.concurrent.thread
import kotlin.math.sqrt

/**
 * Records 16 kHz mono speech on its own thread until [stop], handing each
 * 100 ms of audio to [onAudio] and its 0..1 loudness to [onLevel], both on
 * that thread. Keeps nothing itself, so a dictation can run as long as the
 * user likes. Single use.
 *
 * If the microphone fails while recording (another app takes it, or the
 * permission is revoked), the thread ends and [onError] gets the
 * `AudioRecord` error code, on that thread.
 */
class MicRecorder(
    private val onLevel: (Float) -> Unit,
    private val onError: (Int) -> Unit,
    private val onAudio: (FloatArray) -> Unit,
) {
    @Volatile private var running = false
    private var worker: Thread? = null

    /** Starts recording from the audio [open] opens, which throws IllegalStateException if it can't. */
    fun start(open: () -> MicAudio) {
        val audio = open()
        running = true
        worker = thread(name = "mic") { pump(audio) }
    }

    /** Stops recording; every [onAudio] call has returned when this does. */
    fun stop() {
        running = false
        worker?.join()
        worker = null
    }

    private fun pump(audio: MicAudio) {
        val buffer = FloatArray(Transcriber.SAMPLE_RATE / 10)
        try {
            while (running) {
                val n = audio.read(buffer)
                if (n < 0) {
                    if (running) onError(n)
                    break
                }
                if (n == 0) continue
                onAudio(buffer.copyOf(n))
                onLevel(level(buffer, n))
            }
        } finally {
            audio.close()
        }
    }

    /** RMS scaled so ordinary speech fills most of the meter. */
    private fun level(buffer: FloatArray, n: Int): Float {
        var sum = 0.0
        for (i in 0 until n) sum += buffer[i] * buffer[i]
        return (sqrt(sum / n) * 8).toFloat().coerceIn(0f, 1f)
    }
}
