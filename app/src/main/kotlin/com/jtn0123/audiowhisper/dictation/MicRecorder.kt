package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.DictationAudio
import com.jtn0123.audiowhisper.rust.MicRead
import com.jtn0123.audiowhisper.rust.MicRules
import kotlin.concurrent.thread

/**
 * Records 16 kHz mono speech on its own thread until [stop], handing each
 * 100 ms of audio to [onAudio] and its 0..1 loudness to [onLevel], both on
 * that thread. What a read means and how loud it is are Rust's ([MicRules]). Keeps nothing itself, so a dictation can run as long as the
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

    private val audioLock = Any()

    /** The audio being recorded, until the recording thread closes it. Guarded by [audioLock]. */
    private var audio: MicAudio? = null

    /** Starts recording from the audio [open] opens, which throws IllegalStateException if it can't. */
    fun start(open: () -> MicAudio) {
        val audio = open()
        synchronized(audioLock) { this.audio = audio }
        running = true
        worker = thread(name = "mic") { pump(audio) }
    }

    /**
     * Stops recording; every [onAudio] call has returned when this does. The read in progress is
     * cut short, so on the main thread this takes milliseconds rather than up to a 100 ms read.
     */
    fun stop() {
        running = false
        // Locked so the recording thread, ending after a mic error, can't close the audio meanwhile.
        synchronized(audioLock) { audio?.stop() }
        worker?.join()
        worker = null
    }

    private fun pump(audio: MicAudio) {
        val buffer = FloatArray(DictationAudio.chunkSamples)
        try {
            while (running) {
                val n = audio.read(buffer)
                when (MicRules.read(n, running)) {
                    MicRead.REPORT -> return onError(n)
                    MicRead.END -> return
                    MicRead.SKIP -> continue
                }
                onAudio(buffer.copyOf(n))
                onLevel(MicRules.level(buffer, n))
            }
        } finally {
            synchronized(audioLock) {
                this.audio = null
                audio.close()
            }
        }
    }
}
