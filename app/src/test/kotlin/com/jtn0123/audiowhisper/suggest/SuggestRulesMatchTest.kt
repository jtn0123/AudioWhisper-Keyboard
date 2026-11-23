package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test

/**
 * The suggest package's rules in Rust give exactly what the Kotlin they replaced gave (its answers
 * are golden files, docs/phase6-kotlin-references.md), through [SuggestComparison]: every code
 * point and every UTF-16 unit through each rule that judges one, then random words, contexts,
 * corrections, taps and tap trails.
 */
class SuggestRulesMatchTest {
    private fun assertGolden(name: String, body: SuggestComparison.() -> Unit) {
        GoldenTape.check("suggest-rules-$name") { SuggestComparison(sigmaExact = false, it).apply(body) }
    }

    @Test
    fun theLayoutAndNumbersAreTheSame() = assertGolden("constants") { constants() }

    @Test
    fun everyCodePointIsJudgedAsKotlinJudgedIt() = assertGolden("code-points") { everyCodePoint() }

    @Test
    fun everyCharacterIsJudgedAsKotlinJudgedIt() = assertGolden("units") { everyUnit() }

    @Test
    fun randomWordsContextsAndCorrectionsGiveTheSame() = assertGolden("strings") { randomStrings(Random(SEED), 200_000) }

    @Test
    fun randomTapsLandTheSame() = assertGolden("taps") { taps(Random(SEED), 200_000) }

    @Test
    fun randomTapTrailsRememberTheSame() = assertGolden("trails") { trails(Random(SEED), 20_000) }

    private companion object {
        const val SEED = 20261108L
    }
}
