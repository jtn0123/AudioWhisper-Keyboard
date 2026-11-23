package dev.patrickgold.florisboard.ime.text.keyboard

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The text keyboard's geometry in Rust ([TextKeyboard] and [TextKey.applyFlex], which call
 * lib/rust/crates/keys) gives exactly what the Kotlin it replaced did ([ReferenceTextKeyboard]):
 * every key's touch and visible bounds to the bit, over random keyboards, flex factors, sizes and
 * desired keys from realistic to NaN, the key each touch on and around their edges lands on, the
 * flex table for every mode, and the desired key.
 */
class TextKeyboardMatchTest {
    @Test
    fun geometryMatches() {
        val tape = GoldenTape.ofReference("text-keyboard")
        val comparison = TextKeyboardComparison(tape)
        val random = Random(SEED)
        comparison.flex(random, runs = 40_000)
        comparison.desiredKeys(random, runs = 20_000)
        comparison.keyboards(random, runs = 10_000, touches = 30)
        tape.finish()
        assertEquals(emptyList(), comparison.differences)
        for (outcome in OUTCOMES) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        val OUTCOMES = listOf(
            "flex 1.0 0.0 2.68",
            "flex 1.0 0.0 1.56",
            "flex 1.0 1.0 1.56",
            "flex 1.0 0.0 1.26",
            "flex 1.0 1.0 1.26",
            "flex 1.0 0.0 1.0",
            "flex 1.0 1.0 1.0",
            "flex 1.5 0.0 1.56",
            "flex 0.0 0.0 1.56",
            "desired: base row",
            "desired: capped",
            "desired: a row of the height",
            "layout: skipped",
            "layout: laid out",
            "row: empty",
            "row: fits exactly",
            "row: grows at the ends",
            "row: grows",
            "row: shrinks",
            "touch: a key",
            "touch: a key past a row without keys",
            "touch: no key",
        )
    }
}
