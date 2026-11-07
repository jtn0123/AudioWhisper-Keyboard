package com.jtn0123.audiowhisper.dictation

import android.Manifest
import android.content.Context
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import androidx.annotation.RequiresPermission
import dev.patrickgold.florisboard.BuildConfig
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

/** Where [MicRecorder] gets its 16 kHz mono audio. */
interface MicAudio : AutoCloseable {
    /** Fills [buffer] (blocking for about its length); the count read, or a negative error code. */
    fun read(buffer: FloatArray): Int

    /** The phone's microphone. */
    class Mic private constructor(private val record: AudioRecord) : MicAudio {
        override fun read(buffer: FloatArray): Int = record.read(buffer, 0, buffer.size, AudioRecord.READ_BLOCKING)

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

            private fun bufferBytes(): Int = maxOf(
                AudioRecord.getMinBufferSize(Transcriber.SAMPLE_RATE, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_FLOAT),
                Transcriber.SAMPLE_RATE * Float.SIZE_BYTES / 2,
            )
        }
    }

    /**
     * Debug builds only: a recording played in place of the mic, in real time, then silence.
     * It lets dictation be tested on an emulator, which has no voice to hear.
     */
    class Replay(private val samples: FloatArray) : MicAudio {
        private var position = 0

        override fun read(buffer: FloatArray): Int {
            Thread.sleep(buffer.size * 1000L / Transcriber.SAMPLE_RATE)
            val n = (samples.size - position).coerceIn(0, buffer.size)
            samples.copyInto(buffer, 0, position, position + n)
            buffer.fill(0f, n)
            position += n
            return buffer.size
        }

        override fun close() = Unit

        companion object {
            private const val FILE_NAME = "debug-mic.wav"

            /** The recording to play, if this is a debug build and one was copied in with run-as. */
            fun file(context: Context): File? =
                File(context.filesDir, FILE_NAME).takeIf { BuildConfig.DEBUG && it.isFile }

            /** Reads a 16 kHz mono 16-bit PCM WAV file. */
            fun open(file: File): Replay = Replay(pcm16(file.readBytes()))

            internal fun pcm16(wav: ByteArray): FloatArray {
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
        }
    }
}
