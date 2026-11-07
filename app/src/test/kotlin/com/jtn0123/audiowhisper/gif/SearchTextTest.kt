package com.jtn0123.audiowhisper.gif

import kotlin.test.Test
import kotlin.test.assertEquals

class SearchTextTest {
    @Test
    fun `typed text is appended`() {
        assertEquals("cats", SearchText.type("cat", "s"))
    }

    @Test
    fun `glided words are spaced from the one before`() {
        assertEquals("happy", SearchText.typeWord("", "happy"))
        assertEquals("happy dance", SearchText.typeWord("happy", "dance"))
        assertEquals("happy dance", SearchText.typeWord("happy ", "dance"))
    }

    @Test
    fun `delete removes one character, or a whole emoji`() {
        assertEquals("ca", SearchText.deleteLast("cat"))
        assertEquals("cat ", SearchText.deleteLast("cat 😀"))
        assertEquals("", SearchText.deleteLast(""))
    }

    @Test
    fun `the query stops at GIPHY's limit`() {
        val long = "a".repeat(SearchText.MAX_LENGTH)
        assertEquals(long, SearchText.type(long, "b"))
    }
}
