package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.rust.ShiftRules.Behavior
import com.jtn0123.audiowhisper.rust.ShiftRules.CapsMode
import com.jtn0123.audiowhisper.rust.ShiftRules.Event
import com.jtn0123.audiowhisper.rust.ShiftRules.State
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * The shift rules through JNI: every ordinal and fact bit reaches Rust as the right one. The rules
 * themselves are tested in Rust and, against the Kotlin they replace, in the app.
 */
class ShiftRulesTest {
    @Test
    fun everyStateCrossesBothWays() {
        for (state in State.entries) {
            assertEquals(state, ShiftRules.next(state, Event.SHIFT_UP, anyPressed = true))
        }
    }

    @Test
    fun eachFactReachesItsRule() {
        assertEquals(State.SHIFTED_AUTOMATIC, ShiftRules.next(
            State.UNSHIFTED, Event.REEVALUATE, capsMode = CapsMode.WORDS,
            autoCapitalization = true, localeCapitalizes = true,
        ))
        assertEquals(State.UNSHIFTED, ShiftRules.next(
            State.SHIFTED_AUTOMATIC, Event.REEVALUATE, capsMode = CapsMode.NONE,
            autoCapitalization = true, localeCapitalizes = true,
        ))
        assertEquals(State.UNSHIFTED, ShiftRules.next(
            State.SHIFTED_AUTOMATIC, Event.REEVALUATE, capsMode = CapsMode.ALL, autoCapitalization = true,
        ))
        assertEquals(State.SHIFTED_MANUAL, ShiftRules.next(State.SHIFTED_MANUAL, Event.REEVALUATE, shiftPressed = true))
        assertEquals(State.CAPS_LOCK, ShiftRules.next(State.UNSHIFTED, Event.SHIFT_DOWN, consecutiveDown = true))
        assertEquals(State.SHIFTED_MANUAL, ShiftRules.next(State.UNSHIFTED, Event.SHIFT_DOWN))
        assertEquals(State.CAPS_LOCK, ShiftRules.next(State.SHIFTED_MANUAL, Event.SHIFT_DOWN, Behavior.CAPSLOCK_BY_CYCLE))
        assertEquals(State.SHIFTED_MANUAL, ShiftRules.next(State.SHIFTED_MANUAL, Event.SHIFT_UP, uninterrupted = true))
        assertEquals(State.UNSHIFTED, ShiftRules.next(State.SHIFTED_MANUAL, Event.SHIFT_UP))
        assertEquals(State.CAPS_LOCK, ShiftRules.next(State.UNSHIFTED, Event.CAPS_LOCK))
        assertEquals(State.UNSHIFTED, ShiftRules.next(State.CAPS_LOCK, Event.SHIFT_CANCEL))
        assertEquals(State.SHIFTED_MANUAL, ShiftRules.next(State.SHIFTED_MANUAL, Event.KEY_TYPED, shiftPressed = true))
        assertEquals(State.UNSHIFTED, ShiftRules.next(State.SHIFTED_MANUAL, Event.KEY_TYPED))
    }

    @Test
    fun doubleTapsAndDoubleSpaces() {
        assertTrue(ShiftRules.isConsecutive(-11, 1_000, -11, 1_299, 300))
        assertFalse(ShiftRules.isConsecutive(-11, 1_000, -11, 1_300, 300))
        assertTrue(ShiftRules.isConsecutive(32, Long.MAX_VALUE, 32, Long.MIN_VALUE, 300))
        assertTrue(ShiftRules.isDoubleSpacePeriod("a "))
        assertFalse(ShiftRules.isDoubleSpacePeriod(". "))
        assertFalse(ShiftRules.isDoubleSpacePeriod("😀"))
        assertTrue(ShiftRules.isDoubleSpacePeriod("\uD83D "))
        assertFalse(ShiftRules.isDoubleSpacePeriod("ab "))
    }
}
