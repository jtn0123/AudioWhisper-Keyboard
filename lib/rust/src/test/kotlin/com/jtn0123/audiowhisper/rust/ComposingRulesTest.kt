package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

/**
 * The text composers through JNI, as the keyboard calls them: actions unpacked, whatever their
 * count, and odd text crossing intact. The rules themselves are tested in Rust and, against the
 * Kotlin they replace, in the app.
 */
class ComposingRulesTest {
    @Test
    fun rulesKeepTheirOrderAndAnyCount() {
        val rules = ComposingRules(listOf("uow" to "ươ", "aa" to "â", "a" to "1", "" to "e"))
        assertEquals(2 to "ươ", rules.actions("uo", "w"))
        assertEquals(1 to "Â", rules.actions("A", "a"))
        assertEquals(0 to "1", rules.actions("b", "a"))
        assertEquals(-1 to "E", rules.actions("b", "c"))
        val long = "x".repeat(70_000)
        assertEquals(69_999 to "y", ComposingRules(listOf(long to "y")).actions(long.drop(1), "x"))
        assertEquals(0 to "z", ComposingRules(emptyList()).actions("", "z"))
    }

    @Test
    fun oddTextCrossesIntact() {
        val rules = ComposingRules(listOf("\uD800" to "\uDC00\u0000", "😀" to "ß"))
        assertEquals(0 to "\uDC00\u0000", rules.actions("", "\uD800"))
        assertEquals(1 to "SS", rules.actions("\uD83D", "\uDE00"))
    }

    @Test
    fun badTablesThrow() {
        assertFailsWith<IllegalArgumentException> { ComposingNative.newRules("ab", intArrayOf(1), "", IntArray(0)) }
        assertFailsWith<IllegalArgumentException> { ComposingNative.newRules("a", intArrayOf(1), "", IntArray(0)) }
        assertFailsWith<IllegalArgumentException> { ComposingNative.newRules("a", intArrayOf(-1), "a", intArrayOf(1)) }
        assertFailsWith<IllegalArgumentException> { ComposingNative.rules(0, "", "a") }
    }
}
