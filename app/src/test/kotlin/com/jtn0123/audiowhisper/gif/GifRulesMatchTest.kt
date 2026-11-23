package com.jtn0123.audiowhisper.gif

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The GIF panel's rules in Rust give exactly what the Kotlin they replaced gave
 * ([ReferenceGif]), through [GifComparison]: random queries and texts, typed words for the chips,
 * every key code near FlorisBoard's, swipes and failed calls. The result cache and recents are
 * held to main's inside the session, by GifSearchTraceTest. The same comparison runs on the
 * emulator, whose Java judges text differently.
 */
class GifRulesMatchTest {
    private fun assertNone(reached: List<String>, body: GifComparison.() -> Unit) {
        val comparison = GifComparison().apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
        for (outcome in reached) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    @Test
    fun textMatches() = assertNone(TEXT_OUTCOMES) { text(Random(SEED), 100_000) }

    @Test
    fun chipsMatch() = assertNone(CHIP_OUTCOMES) { chips(Random(SEED), 40_000) }

    @Test
    fun keysMatch() = assertNone(KEY_OUTCOMES) { keys(Random(SEED), swipes = 40_000) }

    @Test
    fun problemsMatch() = assertNone(emptyList()) { problems() }

    @Test
    fun constantsMatch() = assertNone(emptyList()) { constants() }

    private companion object {
        const val SEED = 20261108L

        /** Each outcome must come up this often, or the run wasn't testing it. */
        const val MIN_SEEN = 100

        val TEXT_OUTCOMES = listOf(
            "cut at the limit", "an emoji deleted whole", "half an emoji deleted", "words deleted", "a word offered",
            "no word offered", "a word with an apostrophe",
        )
        val CHIP_OUTCOMES = listOf("no word", "a chip moved first", "a word put first")
        val KEY_OUTCOMES = listOf(
            "route SEARCH", "route KEYBOARD", "route IGNORE", "no swipe delete", "a swipe deletes", "a swipe deletes nothing",
        )
    }
}
