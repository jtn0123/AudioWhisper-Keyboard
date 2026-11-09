package com.jtn0123.audiowhisper.gif

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * GIPHY's answers read, and its URLs built, in Rust give exactly what the Kotlin they replaced
 * gave ([ReferenceGiphy]), through [GiphyComparison]: random answers whole and broken, random
 * sizes, and URLs for odd searches. The same comparison runs on the emulator, whose Java parses
 * numbers and encodes URLs itself.
 */
class GiphyRulesMatchTest {
    private fun assertNone(reached: List<String>, body: GiphyComparison.() -> Unit) {
        val comparison = GiphyComparison().apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
        for (outcome in reached) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times: ${comparison.seen}")
        }
    }

    @Test
    fun answersMatch() = assertNone(BODY_OUTCOMES) { bodies(Random(SEED), 60_000) }

    @Test
    fun sizesMatch() = assertNone(SIZE_OUTCOMES) { sizes(Random(SEED), 100_000) }

    @Test
    fun urlsMatch() = assertNone(URL_OUTCOMES) { urls(Random(SEED), 30_000) }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        val BODY_OUTCOMES = listOf(
            "readable", "unreadable", "broken: unreadable", "broken but readable", "no gifs", "one gif", "several gifs",
        )
        val SIZE_OUTCOMES = listOf(
            "not a number", "NaN", "infinite", "zero", "subnormal", "a number", "hexadecimal", "a width read",
        )
        val URL_OUTCOMES = listOf("a search cut", "half an emoji", "a setting in the params")
    }
}
