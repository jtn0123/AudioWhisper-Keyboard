package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/**
 * Reading aloud's rules in Rust give exactly what the Kotlin they replaced gave
 * ([ReferenceReader]), through [ReaderComparison]: random pages, selections, scrolls, text to
 * speak, pieces to keep in view and presses of the speaker key. The same comparison runs on the
 * emulator, whose Java judges spaces, case and sentences differently.
 */
class ReaderRulesMatchTest {
    private fun assertNone(name: String, reached: List<String>, body: ReaderComparison.() -> Unit) {
        val tape = GoldenTape.ofReference("reader-rules-$name")
        val comparison = ReaderComparison(sigmaExact = false, tape).apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
        tape.finish()
        for (outcome in reached) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    @Test
    fun pagesMatch() = assertNone("pages", PAGE_OUTCOMES) { pages(Random(SEED), 40_000) }

    @Test
    fun joinedBlocksMatch() = assertNone("joins", JOIN_OUTCOMES) { joins(Random(SEED), 40_000) }

    @Test
    fun scrollsMatch() = assertNone("scrolls", SCROLL_OUTCOMES) { scrolls(Random(SEED), 40_000) }

    @Test
    fun speechMatches() = assertNone("speech", SPEECH_OUTCOMES) { speech(Random(SEED), 20_000) }

    @Test
    fun viewsMatch() = assertNone("views", VIEW_OUTCOMES) { views(Random(SEED), 100_000) }

    @Test
    fun speakerKeysMatch() = assertNone("keys", KEY_OUTCOMES) { keys(Random(SEED), runs = 2_000, steps = 40) }

    @Test
    fun constantsMatch() = assertNone("constants", emptyList()) { constants() }

    @Test
    fun nothingIsCutIntoPiecesOfNoLength() {
        // The Kotlin cut text forever for 0, and threw StringIndexOutOfBounds for less.
        for (max in listOf(0, -1, Int.MIN_VALUE)) {
            assertFailsWith<IllegalArgumentException> { PageText.utterances("Hi.", max) }
        }
    }

    private companion object {
        const val SEED = 20261108L

        /** Each outcome must come up this often, or the run wasn't testing it. */
        const val MIN_SEEN = 100

        val PAGE_OUTCOMES = listOf(
            "none on screen", "one on screen", "all shown", "not all shown", "selection found on screen",
            "selection found off screen", "selection found mid-block", "selection not found",
        )
        val JOIN_OUTCOMES = listOf("pieces joined", "a bullet dropped", "a piece dropped")
        val SCROLL_OUTCOMES = listOf("resumed at the start", "nothing new", "resumed part way", "read more than the recent")
        val SPEECH_OUTCOMES = listOf(
            "sentences joined", "a sentence cut", "nothing to say", "cut at the app's limit", "whole at the app's limit",
        )
        val VIEW_OUTCOMES = listOf("fits", "doesn't fit", "fits by its top half")
        val KEY_OUTCOMES = listOf("STOP", "FROM_SCREEN", "FROM_TOP", "no action")
    }
}
