package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
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
    private fun assertNone(name: String, body: SwipeComparison.() -> Unit) {
        val tape = GoldenTape.ofReference("swipe-rules-$name")
        val comparison = SwipeComparison(exactMath = false, tape).apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
        tape.finish()
    }

    @Test
    fun everyBonusIsTheSame() = assertNone("bonuses") { bonuses(Random(SEED), maxCount = 1_000_000, span = 260, randomCount = 100_000) }

    @Test
    fun guessesRankTheSame() = assertNone("rankings") { rankings(Random(SEED), 100_000) }

    @Test
    fun everyFrequencyIsTheSame() = assertNone("frequencies") { frequencies() }

    @Test
    fun theSameWordsCanBeSwiped() = assertNone("swipeable") { swipeable(Random(SEED), 100_000) }

    @Test
    fun wordListsAreTheSame() = assertNone("word-lists") { wordLists(Random(SEED), 20_000) }

    private companion object {
        const val SEED = 20261108L
    }
}
