package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
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
    private fun assertNone(name: String, body: SuggestComparison.() -> Unit) {
        val tape = GoldenTape.ofReference("suggest-rules-$name")
        val comparison = SuggestComparison(sigmaExact = false, tape).apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
        tape.finish()
    }

    @Test
    fun theLayoutAndNumbersAreTheSame() = assertNone("constants") { constants() }

    @Test
    fun everyCodePointIsJudgedAsKotlinJudgedIt() = assertNone("code-points") { everyCodePoint() }

    @Test
    fun everyCharacterIsJudgedAsKotlinJudgedIt() = assertNone("units") { everyUnit() }

    @Test
    fun randomWordsContextsAndCorrectionsGiveTheSame() = assertNone("strings") { randomStrings(Random(SEED), 200_000) }

    @Test
    fun randomTapsLandTheSame() = assertNone("taps") { taps(Random(SEED), 200_000) }

    @Test
    fun randomTapTrailsRememberTheSame() = assertNone("trails") { trails(Random(SEED), 20_000) }

    private companion object {
        const val SEED = 20261108L
    }
}
