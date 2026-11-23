package com.jtn0123.audiowhisper.gif

import java.io.ByteArrayInputStream
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class LimitedReadTest {
    @Test
    fun `reads everything up to the cap`() {
        val bytes = ByteArray(10_000) { it.toByte() }
        assertContentEquals(bytes, readLimited(ByteArrayInputStream(bytes), 10_000))
    }

    @Test
    fun `refuses one byte past the cap`() {
        val e = assertFailsWith<TooLargeException> { readLimited(ByteArrayInputStream(ByteArray(10_001)), 10_000) }
        assertEquals(10_000, e.maxBytes)
    }
}
