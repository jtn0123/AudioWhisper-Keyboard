package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The screen walk's rules in Rust give exactly what the Kotlin they replaced gave
 * ([ReferenceScreen]), through [ScreenComparison]: random scrollable views, boxes, trees of
 * scrollers and pieces to bring into view. The same comparison runs on the emulator.
 */
class ScreenRulesMatchTest {
    private fun assertNone(name: String, reached: List<String>, body: ScreenComparison.() -> Unit) {
        val tape = GoldenTape.ofReference("screen-rules-$name")
        val comparison = ScreenComparison(tape).apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
        tape.finish()
        for (outcome in reached) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    @Test
    fun sidewaysMatches() = assertNone("sideways", SIDEWAYS_OUTCOMES) { sideways(Random(SEED), 40_000) }

    @Test
    fun boxesMatch() = assertNone("boxes", BOX_OUTCOMES) { boxes(Random(SEED), 100_000) }

    @Test
    fun listsMatch() = assertNone("lists", LIST_OUTCOMES) { lists(Random(SEED), 40_000) }

    @Test
    fun movesMatch() = assertNone("moves", MOVE_OUTCOMES) { moves(Random(SEED), 100_000) }

    @Test
    fun constantsMatch() = assertNone("constants", emptyList()) { constants() }

    private companion object {
        const val SEED = 20261108L

        /** Each outcome must come up this often, or the run wasn't testing it. */
        const val MIN_SEEN = 100

        val SIDEWAYS_OUTCOMES =
            listOf("sideways by its collection", "sideways by its scrolling", "sideways by its name", "not sideways")
        val BOX_OUTCOMES = listOf("overlapping", "overlapping with no area", "apart", "an area that wraps", "a width that wraps")
        val LIST_OUTCOMES =
            listOf("no list", "not the biggest", "a piece in the list", "a piece in a list inside it", "a piece outside it")
        val MOVE_OUTCOMES = listOf(
            "move STAY", "move SHOW_ON_SCREEN", "move TURN_PAGES", "after a turn SHOW_ON_SCREEN", "after a turn DONE",
            "after a turn TURN_AGAIN", "well inside", "not well inside",
        )
    }
}
