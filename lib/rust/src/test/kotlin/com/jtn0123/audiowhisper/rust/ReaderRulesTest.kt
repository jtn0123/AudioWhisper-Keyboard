package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * Reading aloud's rules through JNI, as the keyboard calls them: blocks' edges as one array, a
 * block's place and text back together, none as -1 or null, and bad input thrown. The rules
 * themselves are tested in Rust and, against the Kotlin they replace, in the app.
 */
class ReaderRulesTest {
    @Test
    fun theNumbersAreKotlins() {
        assertEquals(3_900, ReaderRules.maxUtterance)
        assertEquals(50, ReaderRules.recent)
        assertEquals(400L, SpeakerKeyState.holdMs)
    }

    @Test
    fun blocksOnScreenAreFound() {
        val edges = intArrayOf(-150, -50, -50, 50, 50, 150)
        assertEquals(1, ReaderRules.firstOnScreen(edges, 0, 1000))
        assertEquals(-1, ReaderRules.firstOnScreen(edges, 2000, 3000))
        assertEquals(-1, ReaderRules.firstOnScreen(intArrayOf(), 0, 1000))
        assertFalse(ReaderRules.allShown(edges, 0, 1000))
        assertTrue(ReaderRules.allShown(edges.copyOfRange(2, 6), 0, 1000))
        assertFailsWith<IllegalArgumentException> { ReaderRules.firstOnScreen(intArrayOf(1, 2, 3), 0, 1000) }
    }

    @Test
    fun joinedBlocksComeBackWithTheirFirstPieceAndPlace() {
        val texts = listOf("• ", "Has a ", "link", "|", "Next", ", more.")
        val edges = intArrayOf(0, 50, 0, 50, 0, 100, 110, 120, 130, 130, 130, 180)
        val joined = ReaderRules.join(texts, edges, BooleanArray(texts.size))
        assertEquals(listOf("Has a link", "Next, more."), joined.map { it.text })
        assertEquals(listOf(0, 4), joined.map { it.first })
        assertEquals(listOf(0 to 100, 130 to 180), joined.map { it.top to it.bottom })
        assertEquals(emptyList(), ReaderRules.join(emptyList(), intArrayOf(), BooleanArray(0)))
        assertFailsWith<IllegalArgumentException> { ReaderRules.join(texts, edges, BooleanArray(1)) }
    }

    @Test
    fun aSelectionIsItsBlockAndOffset() {
        val texts = listOf("Hello there.", "The quick brown fox.")
        val edges = intArrayOf(0, 100, 100, 200)
        assertContentEquals(intArrayOf(1, 10), ReaderRules.find(texts, edges, "BROWN\nfox", 0, 1000))
        assertNull(ReaderRules.find(texts, edges, "zebra", 0, 1000))
        assertFailsWith<IllegalArgumentException> { ReaderRules.find(texts, intArrayOf(0, 100), "a", 0, 1000) }
    }

    @Test
    fun readingCarriesOnAfterTheLastRead() {
        assertEquals(3, ReaderRules.resumeAfter(listOf("b", "c", "d", "e"), listOf("a", "b", "c", "d")))
        assertEquals(0, ReaderRules.resumeAfter(listOf("x"), emptyList()))
    }

    @Test
    fun textIsCutAtTheSentencesGiven() {
        val text = "First one. Second one! Third?"
        assertEquals(listOf(text), ReaderRules.utterances(text, intArrayOf(11, 12, 6), 3_900))
        assertEquals(listOf("First one.", "Second one!", "Third?"), ReaderRules.utterances(text, intArrayOf(11, 12, 6), 12))
        assertEquals(emptyList(), ReaderRules.utterances("", intArrayOf(), 1))
        for (max in listOf(0, -1, Int.MIN_VALUE)) {
            assertFailsWith<IllegalArgumentException> { ReaderRules.utterances(text, intArrayOf(29), max) }
        }
        assertFailsWith<IllegalArgumentException> { ReaderRules.utterances(text, intArrayOf(11, 12), 12) }
        assertFailsWith<IllegalArgumentException> { ReaderRules.utterances(text, intArrayOf(11, 12, 7), 12) }
    }

    @Test
    fun aPieceFitsOnceItStartsInTheTopHalf() {
        assertTrue(ReaderRules.fits(400, 600, 200, 1_400))
        assertTrue(ReaderRules.fits(500, 2_000, 200, 1_400))
        assertFalse(ReaderRules.fits(1_000, 2_500, 200, 1_400))
    }

    @Test
    fun theSpeakerKeyTellsATapFromAHold() {
        val key = SpeakerKeyState()
        assertNull(key.down(reading = false, now = 1_000))
        assertEquals(SpeakerKeyState.Action.FROM_SCREEN, key.up(1_399))
        key.down(reading = false, now = 1_000)
        assertEquals(SpeakerKeyState.Action.FROM_TOP, key.up(1_400))
        assertNull(key.up(2_000))
        assertEquals(SpeakerKeyState.Action.STOP, key.down(reading = true, now = 0))
        assertNull(key.up(1_000))
        key.down(reading = false, now = 0)
        key.cancel()
        assertNull(key.up(1_000))
    }
}
