package com.jtn0123.audiowhisper.dictation

import android.Manifest
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import androidx.annotation.RequiresPermission
import kotlin.concurrent.thread
import kotlin.math.sqrt

/**
 * Records 16 kHz mono speech on its own thread until [stop], handing each
 * 100 ms of audio to [onAudio] and its 0..1 loudness to [onLevel], both on
 * that thread. Keeps nothing itself, so a dictation can run as long as the
 * user likes. Single use.
 */
class MicRecorder(
    private val onLevel: (Float) -> Unit,
    private val onAudio: (FloatArray) -> Unit,
) {
    @Volatile private var running = false
    private var worker: Thread? = null

    @RequiresPermission(Manifest.permission.RECORD_AUDIO)
    fun start() {
        val record = AudioRecord(
            MediaRecorder.AudioSource.VOICE_RECOGNITION,
            Transcriber.SAMPLE_RATE,
            AudioFormat.CHANNEL_IN_MONO,
            AudioFormat.ENCODING_PCM_FLOAT,
            bufferBytes(),
        )
        if (record.state != AudioRecord.STATE_INITIALIZED) {
            record.release()
            throw IllegalStateException("The microphone could not be opened")
        }
        running = true
        record.startRecording()
        worker = thread(name = "mic") { pump(record) }
    }

    /** Stops recording; every [onAudio] call has returned when this does. */
    fun stop() {
        running = false
        worker?.join()
        worker = null
    }

    private fun pump(record: AudioRecord) {
        val buffer = FloatArray(Transcriber.SAMPLE_RATE / 10)
        try {
            while (running) {
                val n = record.read(buffer, 0, buffer.size, AudioRecord.READ_BLOCKING)
                if (n < 0) break
                if (n == 0) continue
                onAudio(buffer.copyOf(n))
                onLevel(level(buffer, n))
            }
        } finally {
            record.stop()
            record.release()
        }
    }

    private fun bufferBytes(): Int = maxOf(
        AudioRecord.getMinBufferSize(
            Transcriber.SAMPLE_RATE,
            AudioFormat.CHANNEL_IN_MONO,
            AudioFormat.ENCODING_PCM_FLOAT,
        ),
        Transcriber.SAMPLE_RATE * Float.SIZE_BYTES / 2,
    )

    /** RMS scaled so ordinary speech fills most of the meter. */
    private fun level(buffer: FloatArray, n: Int): Float {
        var sum = 0.0
        for (i in 0 until n) sum += buffer[i] * buffer[i]
        return (sqrt(sum / n) * 8).toFloat().coerceIn(0f, 1f)
    }
}
