package com.jtn0123.audiowhisper.rust

import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class LatinSessionBoundaryTest {
    private val bytes get() = File("../../app/src/main/assets/dicts/main_en-US.dict").readBytes()

    @Test fun bytesAndClosedHandleHaveExplicitLifetimes() {
        assertNull(LatinSession.open(byteArrayOf()))
        val session = requireNotNull(LatinSession.open(bytes))
        assertTrue(session.probability("hello") >= 0)
        session.close()
        session.close()
        val error = assertFailsWith<IllegalArgumentException> { session.probability("hello") }
        assertEquals("the session is closed", error.message)
        assertFailsWith<IllegalArgumentException> { session.words() }
        assertFailsWith<IllegalArgumentException> { session.decode("a", emptyList(), false, true, FloatArray(0)) }
    }

    @Test fun malformedTapTransferThrowsBeforeSearching() {
        val handle = LatinSessionNative.open(bytes)
        assertTrue(handle != 0L)
        try {
            assertFailsWith<IllegalArgumentException> {
                LatinSessionNative.decode(handle, "hello", emptyArray(), false, true, floatArrayOf(1f))
            }
            val error = assertFailsWith<IllegalArgumentException> {
                LatinSessionNative.decode(handle, "a", emptyArray(), false, true, floatArrayOf(1f, Float.NaN, 0f))
            }
            assertEquals("Cannot round NaN value.", error.message)
            assertTrue(LatinSessionNative.probability(handle, "hello") >= 0)
        } finally { LatinSessionNative.free(handle) }
    }

    @Test fun midWordUsesKotlinUtf16CharacterRules() {
        for (unit in Char.MIN_VALUE..Char.MAX_VALUE) {
            assertEquals(unit.isLetterOrDigit(), SuggestRules.isCursorMidWord(unit.toString()))
        }
        assertEquals(false, SuggestRules.isCursorMidWord(""))
    }
}
