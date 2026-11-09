package dev.patrickgold.florisboard.ime.keyboard

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The shift key in Rust ([ShiftLogic], which calls lib/rust/crates/shift) does exactly what the
 * Kotlin it replaced did ([ReferenceShift]): every event from every state under every behaviour,
 * caps mode and set of facts, random runs of them carrying the state along, double taps across a
 * wrapping clock, and every UTF-16 unit on either side of a second space.
 */
class ShiftMatchTest {
    @Test
    fun shiftStatesMatch() {
        val comparison = ShiftComparison()
        comparison.everyCase()
        comparison.runs(Random(SEED), runs = 5_000, steps = 40)
        assertEquals(emptyList(), comparison.differences)
        // Every case is each case once, so it need only reach each transition; the runs, by chance, often.
        assertEquals(TRANSITIONS.flatMap { listOf("every $it", "run $it") }.toSet(), comparison.seen.keys)
        for (outcome in TRANSITIONS.map { "run $it" }) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    @Test
    fun doubleTapsAndDoubleSpacesMatch() {
        val comparison = ShiftComparison()
        comparison.doubleTaps(Random(SEED), count = 200_000)
        comparison.doubleSpaces(Random(SEED), texts = 100_000)
        assertEquals(emptyList(), comparison.differences)
        val expected = listOf("consecutive: true", "consecutive: false", "double space 2: true", "double space 2: false") +
            (0..3).filter { it != 2 }.map { "double space $it: false" }
        assertEquals(expected.toSet(), comparison.seen.keys)
        for (outcome in expected) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        const val U = "UNSHIFTED"
        const val M = "SHIFTED_MANUAL"
        const val A = "SHIFTED_AUTOMATIC"
        const val C = "CAPS_LOCK"
        val TRANSITIONS = listOf(
            "REEVALUATE $U -> $U", "REEVALUATE $U -> $A", "REEVALUATE $M -> $M", "REEVALUATE $M -> $U",
            "REEVALUATE $M -> $A", "REEVALUATE $A -> $A", "REEVALUATE $A -> $U", "REEVALUATE $C -> $C",
            "SHIFT_DOWN $U -> $M", "SHIFT_DOWN $U -> $C", "SHIFT_DOWN $M -> $U", "SHIFT_DOWN $M -> $C",
            "SHIFT_DOWN $A -> $U", "SHIFT_DOWN $A -> $C", "SHIFT_DOWN $C -> $U", "SHIFT_DOWN $C -> $C",
            "SHIFT_UP $U -> $U", "SHIFT_UP $M -> $M", "SHIFT_UP $M -> $U", "SHIFT_UP $A -> $A",
            "SHIFT_UP $A -> $U", "SHIFT_UP $C -> $C",
            "CAPS_LOCK $U -> $C", "CAPS_LOCK $M -> $C", "CAPS_LOCK $A -> $C", "CAPS_LOCK $C -> $C",
            "SHIFT_CANCEL $U -> $U", "SHIFT_CANCEL $M -> $U", "SHIFT_CANCEL $A -> $U", "SHIFT_CANCEL $C -> $U",
            "KEY_TYPED $U -> $U", "KEY_TYPED $M -> $M", "KEY_TYPED $M -> $U", "KEY_TYPED $A -> $A",
            "KEY_TYPED $A -> $U", "KEY_TYPED $C -> $C",
        )
    }
}
