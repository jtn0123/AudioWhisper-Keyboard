package com.jtn0123.audiowhisper.suggest

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The suggest package's rules in Rust give exactly what the Kotlin they replaced gave
 * ([ReferenceSuggest]), through [SuggestComparison]: every code point and every UTF-16 unit
 * through each rule that judges one, then random words, contexts, corrections, taps and tap
 * trails. The same comparison runs on the emulator, whose Java judges some characters differently.
 */
class SuggestRulesMatchTest {
    private fun assertNone(body: SuggestComparison.() -> Unit) {
        val comparison = SuggestComparison(sigmaExact = false).apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
    }

    @Test
    fun theLayoutAndNumbersAreTheSame() = assertNone { constants() }

    @Test
    fun everyCodePointIsJudgedAsKotlinJudgedIt() = assertNone { everyCodePoint() }

    @Test
    fun everyCharacterIsJudgedAsKotlinJudgedIt() = assertNone { everyUnit() }

    @Test
    fun randomWordsContextsAndCorrectionsGiveTheSame() = assertNone { randomStrings(Random(SEED), 200_000) }

    @Test
    fun randomTapsLandTheSame() = assertNone { taps(Random(SEED), 200_000) }

    @Test
    fun randomTapTrailsRememberTheSame() = assertNone { trails(Random(SEED), 20_000) }

    private companion object {
        const val SEED = 20261108L
    }
}
