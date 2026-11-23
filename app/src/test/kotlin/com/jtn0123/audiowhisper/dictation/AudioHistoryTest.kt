package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.MicRules
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class AudioHistoryTest {
    private fun floats(vararg values: Int) = FloatArray(values.size) { values[it].toFloat() }

    @Test
    fun `gives back the audio before a phrase`() {
        val history = ReferenceAudioHistory(capacity = 10)
        history.add(floats(0, 1, 2, 3))
        history.add(floats(4, 5, 6))
        assertEquals(7, history.end)
        assertContentEquals(floats(2, 3, 4), history.slice(2, 5))
    }

    @Test
    fun `a lead-in from before the start is cut to the start`() {
        val history = ReferenceAudioHistory(capacity = 10)
        history.add(floats(0, 1, 2))
        assertContentEquals(floats(0, 1), history.slice(-3, 2))
    }

    @Test
    fun `audio older than the capacity is gone, the rest wraps around`() {
        val history = ReferenceAudioHistory(capacity = 4)
        history.add(floats(0, 1, 2, 3, 4, 5))
        assertContentEquals(floats(2, 3, 4, 5), history.slice(0, 6))
        assertContentEquals(floats(4, 5), history.slice(4, 9))
        assertContentEquals(FloatArray(0), history.slice(0, 2))
    }

    @Test
    fun `reads a 16-bit WAV for the debug replay`() {
        val wav = wav(listOf("fmt " to ByteArray(16), "LIST" to ByteArray(3), "data" to pcm(0, 16384, -32768)))
        assertContentEquals(floatArrayOf(0f, 0.5f, -1f), MicRules.pcm16(wav))
        assertFailsWith<IllegalArgumentException> { MicRules.pcm16("not a wav file".toByteArray()) }
        assertFailsWith<IllegalArgumentException> { MicRules.pcm16(wav(listOf("fmt " to ByteArray(16)))) }
    }

    private fun pcm(vararg samples: Int): ByteArray =
        ByteBuffer.allocate(samples.size * 2).order(ByteOrder.LITTLE_ENDIAN).apply {
            samples.forEach { putShort(it.toShort()) }
        }.array()

    /** A RIFF file of [chunks], each padded to an even length as the format requires. */
    private fun wav(chunks: List<Pair<String, ByteArray>>): ByteArray {
        val body = chunks.sumOf { 8 + it.second.size + (it.second.size and 1) }
        val out = ByteBuffer.allocate(12 + body).order(ByteOrder.LITTLE_ENDIAN)
        out.put("RIFF".toByteArray()).putInt(4 + body).put("WAVE".toByteArray())
        for ((id, data) in chunks) {
            out.put(id.toByteArray()).putInt(data.size).put(data)
            if (data.size and 1 == 1) out.put(0)
        }
        return out.array()
    }
}
