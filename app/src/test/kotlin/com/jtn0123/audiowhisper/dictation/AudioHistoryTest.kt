package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.MicRules
import java.nio.ByteBuffer
import java.nio.ByteOrder
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith

/**
 * The debug replay's WAV reading. The audio history the listener keeps is Rust's, and tested there
 * (lib/rust/crates/dictation/src/segmentation.rs).
 */
class AudioHistoryTest {
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
