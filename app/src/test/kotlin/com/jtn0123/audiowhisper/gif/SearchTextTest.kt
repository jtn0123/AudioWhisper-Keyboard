package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRules
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

/** Editing the GIF search field, in Rust (lib/rust/crates/gif), through its binding [GifRules]. */
class SearchTextTest {
    @Test
    fun `typed text is appended`() {
        assertEquals("cats", GifRules.type("cat", "s"))
    }

    @Test
    fun `glided words are spaced from the one before`() {
        assertEquals("happy", GifRules.typeWord("", "happy"))
        assertEquals("happy dance", GifRules.typeWord("happy", "dance"))
        assertEquals("happy dance", GifRules.typeWord("happy ", "dance"))
    }

    @Test
    fun `delete removes one character, or a whole emoji`() {
        assertEquals("ca", GifRules.deleteLast("cat", 1))
        assertEquals("cat ", GifRules.deleteLast("cat 😀", 1))
        assertEquals("", GifRules.deleteLast("", 1))
    }

    @Test
    fun `a swipe deletes several characters, emoji whole`() {
        assertEquals("ca", GifRules.deleteLast("cat 😀", 3))
        assertEquals("cat", GifRules.deleteLast("cat", 0))
        assertEquals("", GifRules.deleteLast("cat", 9))
    }

    @Test
    fun `deleting words keeps the space before them`() {
        assertEquals("happy ", GifRules.deleteWords("happy dance", 1))
        assertEquals("happy ", GifRules.deleteWords("happy dance  ", 1))
        assertEquals("", GifRules.deleteWords("happy dance", 2))
        assertEquals("", GifRules.deleteWords("", 1))
    }

    @Test
    fun `the query stops at GIPHY's limit`() {
        val long = "a".repeat(GifRules.maxLength)
        assertEquals(long, GifRules.type(long, "b"))
    }

    @Test
    fun `the word just typed is offered as a search`() {
        assertEquals("congrats", GifRules.lastWord("Big congrats! "))
        assertEquals("Congrats", GifRules.lastWord("Congrats"))
        assertEquals("don't", GifRules.lastWord("I don't"))
        assertEquals("café", GifRules.lastWord("at the café."))
    }

    @Test
    fun `nothing is offered without a word at the end`() {
        assertNull(GifRules.lastWord(null))
        assertNull(GifRules.lastWord(""))
        assertNull(GifRules.lastWord("   "))
        assertNull(GifRules.lastWord("a"), "too short")
        assertNull(GifRules.lastWord("call 555"))
        assertNull(GifRules.lastWord("see 😀"))
        assertNull(GifRules.lastWord("x".repeat(31)))
    }
}
