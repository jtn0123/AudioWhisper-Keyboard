package com.jtn0123.audiowhisper.gif

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

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
    fun `a swipe deletes several characters, emoji whole`() {
        assertEquals("ca", SearchText.deleteLast("cat 😀", 3))
        assertEquals("cat", SearchText.deleteLast("cat", 0))
        assertEquals("", SearchText.deleteLast("cat", 9))
    }

    @Test
    fun `deleting words keeps the space before them`() {
        assertEquals("happy ", SearchText.deleteWords("happy dance", 1))
        assertEquals("happy ", SearchText.deleteWords("happy dance  ", 1))
        assertEquals("", SearchText.deleteWords("happy dance", 2))
        assertEquals("", SearchText.deleteWords("", 1))
    }

    @Test
    fun `the query stops at GIPHY's limit`() {
        val long = "a".repeat(SearchText.MAX_LENGTH)
        assertEquals(long, SearchText.type(long, "b"))
    }

    @Test
    fun `the word just typed is offered as a search`() {
        assertEquals("congrats", SearchText.lastWord("Big congrats! "))
        assertEquals("Congrats", SearchText.lastWord("Congrats"))
        assertEquals("don't", SearchText.lastWord("I don't"))
        assertEquals("café", SearchText.lastWord("at the café."))
    }

    @Test
    fun `nothing is offered without a word at the end`() {
        assertNull(SearchText.lastWord(null))
        assertNull(SearchText.lastWord(""))
        assertNull(SearchText.lastWord("   "))
        assertNull(SearchText.lastWord("a"), "too short")
        assertNull(SearchText.lastWord("call 555"))
        assertNull(SearchText.lastWord("see 😀"))
        assertNull(SearchText.lastWord("x".repeat(31)))
    }
}
