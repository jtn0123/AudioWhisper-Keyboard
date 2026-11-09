package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.rust.DictationRules.Kind
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Dictation's rules through JNI, as the keyboard calls them: results unpacked whole, text passed
 * through unit for unit, and the name callback. The rules themselves are tested in Rust and, against
 * the Kotlin they replace, in the app.
 */
class DictationRulesTest {
    private fun parts(phrase: String) = DictationRules.parse(phrase).map { it.kind to it.text }

    @Test
    fun parseGivesEveryKindOfPart() {
        assertEquals(
            listOf(
                Kind.WORDS to "Hello there.",
                Kind.BREAK to "\n\n",
                Kind.MARK to "?",
                Kind.SCRATCH_THAT to "",
                Kind.WORDS to "Bye.",
            ),
            parts("Hello there. New paragraph. Question mark. Scratch that. Bye."),
        )
        assertEquals(emptyList(), parts("  "))
    }

    @Test
    fun oddUtf16PassesThroughUnchanged() {
        // A lone surrogate, a pair, a NUL and a byte-order mark: Kotlin Strings may hold any of them.
        val odd = "a\uD800b 😀 \u0000﻿"
        assertEquals(listOf(Kind.WORDS to odd), parts(odd))
        assertEquals(" $odd", DictationRules.join("x", null, odd) { false }.text)
    }

    @Test
    fun joinAsksForNamesAndUnpacksTheTyping() {
        val asked = mutableListOf<String>()
        val typing = DictationRules.join("I need to talk to.", "talk to.", "John about it.") {
            asked += it
            it == "John"
        }
        assertEquals(listOf("John"), asked)
        assertEquals(1, typing.replacing)
        assertEquals(" John about it.", typing.text)
    }

    @Test
    fun anExceptionFromIsNameIsThrown() {
        val thrown = assertFailsWith<IllegalStateException> {
            DictationRules.join("so", null, "Paris is") { error("no dictionary") }
        }
        assertEquals("no dictionary", thrown.message)
    }

    @Test
    fun punctuateUnpacksTheTyping() {
        val typing = DictationRules.punctuate("See you soon.", "See you soon.", '?')
        assertEquals(1, typing.replacing)
        assertEquals("?", typing.text)
        assertEquals(0, DictationRules.punctuate("", null, ',').replacing)
    }

    @Test
    fun revertsOnlyRightAfterThePhrase() {
        assertTrue(DictationRules.revertsAt(" to John.", 8, 17, "I talked to John."))
        assertFalse(DictationRules.revertsAt(" to John.", 8, 16, "I talked to John."))
        assertFalse(DictationRules.revertsAt("", 0, 0, ""))
    }

    @Test
    fun micKeyTellsTapsFromHolds() {
        assertEquals(300L, MicKeyState.holdMs)
        val key = MicKeyState()
        assertEquals(MicKeyState.Action.TOGGLE, key.release(0))
        assertEquals(MicKeyState.Action.START, key.press(MicKeyState.Status.IDLE, 1_000))
        assertEquals(MicKeyState.Action.NONE, key.release(1_299))
        key.press(MicKeyState.Status.IDLE, 1_000)
        assertEquals(MicKeyState.Action.END_HOLD, key.release(1_300))
        assertEquals(MicKeyState.Action.NONE, key.press(MicKeyState.Status.LISTENING, 0))
        assertEquals(MicKeyState.Action.FINISH, key.release(1))
        assertEquals(MicKeyState.Action.NONE, key.press(MicKeyState.Status.FINISHING, 0))
        assertEquals(MicKeyState.Action.NONE, key.release(Long.MIN_VALUE))
    }
}
