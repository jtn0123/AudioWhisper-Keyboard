package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * The GIF panel's rules in Rust give exactly what the Kotlin they replaced gave, as recorded in
 * the golden files `gif-rules-*` (docs/phase6-kotlin-references.md), through [GifComparison]:
 * random queries and texts, typed words for the chips, every key code near FlorisBoard's, swipes
 * and failed calls. The result cache and recents are held to main's inside the session, by
 * GifSearchTraceTest.
 */
class GifRulesMatchTest {
    private fun assertNone(name: String, reached: List<String>, body: GifComparison.() -> Unit) {
        val comparison = GoldenTape.check("gif-rules-$name") { tape -> GifComparison(tape).apply(body) }
        for (outcome in reached) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    @Test
    fun textMatches() = assertNone("text", TEXT_OUTCOMES) { text(Random(SEED), 100_000) }

    @Test
    fun chipsMatch() = assertNone("chips", CHIP_OUTCOMES) { chips(Random(SEED), 40_000) }

    @Test
    fun keysMatch() = assertNone("keys", KEY_OUTCOMES) { keys(Random(SEED), swipes = 40_000) }

    @Test
    fun problemsMatch() = assertNone("problems", emptyList()) { problems() }

    @Test
    fun constantsMatch() = assertNone("constants", emptyList()) { constants() }

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
