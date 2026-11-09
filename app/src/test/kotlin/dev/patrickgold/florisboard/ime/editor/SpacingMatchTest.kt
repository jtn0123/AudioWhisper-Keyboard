package dev.patrickgold.florisboard.ime.editor

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The keyboard's own spaces in Rust ([Spacing], which calls lib/rust/crates/spacing, and the
 * flags' wrappers in [EditorInstance]) do exactly what the Kotlin they replaced did
 * ([ReferenceSpacing]): every UTF-16 unit in each place the rules look, and random runs of
 * characters, words, flags and selection updates in random editors, settings and rules.
 */
class SpacingMatchTest {
    @Test
    fun spacingMatches() {
        val comparison = SpacingComparison()
        comparison.constants()
        comparison.everyUnit()
        comparison.runs(Random(SEED), runs = 4_000, steps = 50)
        assertEquals(emptyList(), comparison.differences)
        for (outcome in OUTCOMES) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        val OUTCOMES = listOf(
            "char: none",
            "char: delete previous space",
            "char: space before",
            "char: space after",
            "looked up: nothing",
            "looked up: [1]",
            "looked up: [1, 3]",
            "determine: true",
            "determine: false",
            "determine: true forced",
            "determine: false forced",
            "determine: unspaced language",
            "update: candidate kept",
            "update: candidate dropped",
            "update: still active",
        )
    }
}
