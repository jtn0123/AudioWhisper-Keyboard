package com.jtn0123.audiowhisper.dictation

import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.concurrent.thread
import kotlin.math.sqrt

/**
 * The microphone's audio rules as main had them at b1cf45f18, before they moved to Rust
 * (dictation::audio): MicRecorder.kt whole, and MicAudio.Replay.pcm16 and Mic.bufferBytes, copied
 * with only these mechanical changes, so MicTraceTest can hold the new code to them: the class is
 * renamed, the chunk size and sample rate are constants here, and bufferBytes takes Android's
 * minimum, which it read from AudioRecord. Don't change it.
 */
internal object ReferenceMic {
    const val SAMPLE_RATE = 16_000
    const val CHUNK_SAMPLES = SAMPLE_RATE / 10

    fun pcm16(wav: ByteArray): FloatArray {
        val bytes = ByteBuffer.wrap(wav).order(ByteOrder.LITTLE_ENDIAN)
        require(wav.size > 12 && String(wav, 0, 4) == "RIFF" && String(wav, 8, 4) == "WAVE") { "Not a WAV file" }
        var offset = 12
        while (offset + 8 <= wav.size) {
            val size = bytes.getInt(offset + 4)
            if (String(wav, offset, 4) == "data") {
                val count = minOf(size, wav.size - offset - 8) / 2
                return FloatArray(count) { bytes.getShort(offset + 8 + it * 2) / 32768f }
            }
            offset += 8 + size + (size and 1)
        }
        throw IllegalArgumentException("No audio in the WAV file")
    }

    fun bufferBytes(minimum: Int): Int = maxOf(
        minimum,
        SAMPLE_RATE * Float.SIZE_BYTES / 2,
    )
}

internal class ReferenceMicRecorder(
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
        val buffer = FloatArray(ReferenceMic.CHUNK_SAMPLES)
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
            synchronized(audioLock) {
                this.audio = null
                audio.close()
            }
        }
    }

    /** RMS scaled so ordinary speech fills most of the meter. */
    private fun level(buffer: FloatArray, n: Int): Float {
        var sum = 0.0
        for (i in 0 until n) sum += buffer[i] * buffer[i]
        return (sqrt(sum / n) * 8).toFloat().coerceIn(0f, 1f)
    }
}
