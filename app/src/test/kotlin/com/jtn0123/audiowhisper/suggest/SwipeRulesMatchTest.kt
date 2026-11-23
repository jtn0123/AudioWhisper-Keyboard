package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test

/**
 * Swipe typing's ranking and word list in Rust give exactly what the Kotlin they replaced gave (its
 * answers are golden files, docs/phase6-kotlin-references.md), through [SwipeComparison]; a
 * frequency as the Float the glide classifier takes (see [SwipeComparison]).
 */
class SwipeRulesMatchTest {
    private fun assertGolden(name: String, body: SwipeComparison.() -> Unit) {
        GoldenTape.check("swipe-rules-$name") { SwipeComparison(it).apply(body) }
    }

    @Test
    fun everyBonusIsTheSame() = assertGolden("bonuses") { bonuses(Random(SEED), maxCount = 1_000_000, span = 260, randomCount = 100_000) }

    @Test
    fun guessesRankTheSame() = assertGolden("rankings") { rankings(Random(SEED), 100_000) }

    @Test
    fun everyFrequencyIsTheSame() = assertGolden("frequencies") { frequencies() }

    @Test
    fun theSameWordsCanBeSwiped() = assertGolden("swipeable") { swipeable(Random(SEED), 100_000) }

    @Test
    fun wordListsAreTheSame() = assertGolden("word-lists") { wordLists(Random(SEED), 20_000) }

    private companion object {
        const val SEED = 20261108L
    }
}
