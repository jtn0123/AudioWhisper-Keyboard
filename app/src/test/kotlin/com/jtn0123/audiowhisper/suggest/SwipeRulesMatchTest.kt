package com.jtn0123.audiowhisper.suggest

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * Swipe typing's ranking and word list in Rust give exactly what the Kotlin they replaced gave
 * ([ReferenceSwipe]), through [SwipeComparison]. The same comparison runs on the emulator, whose
 * Java judges some characters differently, and where frequencies must match to the bit; this
 * computer's JVM has an `exp` of its own (see [SwipeComparison]).
 */
class SwipeRulesMatchTest {
    private fun assertNone(body: SwipeComparison.() -> Unit) {
        val comparison = SwipeComparison(exactMath = false).apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
    }

    @Test
    fun everyBonusIsTheSame() = assertNone { bonuses(Random(SEED), maxCount = 1_000_000, span = 260, randomCount = 100_000) }

    @Test
    fun guessesRankTheSame() = assertNone { rankings(Random(SEED), 100_000) }

    @Test
    fun everyFrequencyIsTheSame() = assertNone { frequencies() }

    @Test
    fun theSameWordsCanBeSwiped() = assertNone { swipeable(Random(SEED), 100_000) }

    @Test
    fun wordListsAreTheSame() = assertNone { wordLists(Random(SEED), 20_000) }

    private companion object {
        const val SEED = 20261108L
    }
}
