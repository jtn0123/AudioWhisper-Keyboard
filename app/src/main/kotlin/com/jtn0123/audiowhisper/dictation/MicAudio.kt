package com.jtn0123.audiowhisper.dictation

import android.Manifest
import android.content.Context
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import androidx.annotation.RequiresPermission
import com.jtn0123.audiowhisper.rust.MicRules
import dev.patrickgold.florisboard.BuildConfig
import java.io.File
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/** Where [MicRecorder] gets its 16 kHz mono audio. */
interface MicAudio : AutoCloseable {
    /** Fills [buffer] (blocking for about its length); the count read, or a negative error code. */
    fun read(buffer: FloatArray): Int

    /**
     * Stops listening from another thread: a [read] blocked there returns at once with what it
     * has (maybe 0), and later ones return 0 or an error. Never called after [close].
     */
    fun stop()

    /** The phone's microphone. */
    class Mic private constructor(private val record: AudioRecord) : MicAudio {
        override fun read(buffer: FloatArray): Int = record.read(buffer, 0, buffer.size, AudioRecord.READ_BLOCKING)

        // AudioRecord.stop interrupts a blocking read in progress.
        override fun stop() = record.stop()

        override fun close() {
            record.stop()
            record.release()
        }

        companion object {
            /** Opens and starts the mic; throws IllegalStateException if it can't. */
            @RequiresPermission(Manifest.permission.RECORD_AUDIO)
            fun open(): Mic {
                val record = AudioRecord(
                    MediaRecorder.AudioSource.VOICE_RECOGNITION,
                    Transcriber.SAMPLE_RATE,
                    AudioFormat.CHANNEL_IN_MONO,
                    AudioFormat.ENCODING_PCM_FLOAT,
                    bufferBytes(),
                )
                try {
                    check(record.state == AudioRecord.STATE_INITIALIZED) { "The microphone could not be opened" }
                    record.startRecording()
                    // Not an exception when another app holds the mic: it just doesn't start.
                    check(record.recordingState == AudioRecord.RECORDSTATE_RECORDING) { "The microphone could not be started" }
                } catch (e: IllegalStateException) {
                    record.release()
                    throw e
                }
                return Mic(record)
            }

            private fun bufferBytes(): Int = MicRules.bufferBytes(
                AudioRecord.getMinBufferSize(Transcriber.SAMPLE_RATE, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_FLOAT),
            )
        }
    }

    /**
     * Debug builds only: a recording played in place of the mic, in real time, then silence.
     * It lets dictation be tested on an emulator, which has no voice to hear.
     */
    class Replay(private val samples: FloatArray) : MicAudio {
        private var position = 0
        private val stopped = CountDownLatch(1)

        override fun read(buffer: FloatArray): Int {
            if (stopped.await(buffer.size * 1000L / Transcriber.SAMPLE_RATE, TimeUnit.MILLISECONDS)) return 0
            val n = (samples.size - position).coerceIn(0, buffer.size)
            samples.copyInto(buffer, 0, position, position + n)
            buffer.fill(0f, n)
            position += n
            return buffer.size
        }

        override fun stop() = stopped.countDown()

        override fun close() = Unit

        companion object {
            private const val FILE_NAME = "debug-mic.wav"

            /** The recording to play, if this is a debug build and one was copied in with run-as. */
            fun file(context: Context): File? =
                File(context.filesDir, FILE_NAME).takeIf { BuildConfig.DEBUG && it.isFile }

            /** Reads a 16 kHz mono 16-bit PCM WAV file; IllegalArgumentException if it holds no audio. */
            fun open(file: File): Replay = Replay(MicRules.pcm16(file.readBytes()))
        }
    }
}
