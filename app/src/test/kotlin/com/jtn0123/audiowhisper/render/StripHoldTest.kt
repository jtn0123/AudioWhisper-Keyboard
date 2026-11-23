package com.jtn0123.audiowhisper.render

import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.WordSuggestionCandidate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertSame

/**
 * The strip across a refill: a key empties Rust's list while `wordsShown` holds the row, and the
 * strip keeps the words it drew until the new ones come or [STRIP_HOLD_MS] passes, so one key
 * changes it once. Every wake asked for must lead to a rebuild that ends the hold.
 */
class StripHoldTest {
    private val first = listOf<SuggestionCandidate>(WordSuggestionCandidate("a"), WordSuggestionCandidate("b"))
    private val second = listOf<SuggestionCandidate>(WordSuggestionCandidate("c"))
    private val none = emptyList<SuggestionCandidate>()
    private val wakes = mutableListOf<Long>()
    private var cancels = 0
    private val hold = StripHold({ wakes.add(it) }, { cancels++ })

    @Test
    fun wordsAreDrawnAsTheyCome() {
        assertSame(first, hold.words(first, due = true, now = 0))
        assertSame(second, hold.words(second, due = true, now = 10))
        assertEquals(emptyList(), wakes)
        assertEquals(0, cancels)
    }

    @Test
    fun aRefillDueKeepsTheWordsDrawnUntilTheNewOnesCome() {
        hold.words(first, due = true, now = 0)
        assertSame(first, hold.words(none, due = true, now = 100))
        assertSame(first, hold.words(none, due = true, now = 140))
        assertEquals(listOf(STRIP_HOLD_MS, STRIP_HOLD_MS - 40), wakes)
        assertSame(second, hold.words(second, due = true, now = 160))
        assertEquals(1, cancels)
    }

    @Test
    fun theHoldEndsAtItsDeadlineAndAnotherKeyDoesNotExtendIt() {
        hold.words(first, due = true, now = 0)
        assertSame(first, hold.words(none, due = true, now = 100))
        // A second key restarts the refill; the deadline stays the first one's.
        assertSame(first, hold.words(none, due = true, now = 100 + STRIP_HOLD_MS - 1))
        assertEquals(listOf(STRIP_HOLD_MS, 1L), wakes)
        assertEquals(none, hold.words(none, due = true, now = 100 + STRIP_HOLD_MS))
        // Nothing is drawn now, so nothing is kept and no wake is asked for.
        assertEquals(none, hold.words(none, due = true, now = 400))
        assertEquals(2, wakes.size)
    }

    @Test
    fun aRowNotHeldEmptiesAtOnce() {
        hold.words(first, due = true, now = 0)
        assertEquals(none, hold.words(none, due = false, now = 10))
        assertEquals(emptyList(), wakes)
    }

    @Test
    fun aRowThatTurnsOutEmptyEmptiesWhenRustLetsItGo() {
        hold.words(first, due = true, now = 0)
        assertSame(first, hold.words(none, due = true, now = 10))
        // The refill found no words: wordsShown turns false and the strip empties then.
        assertEquals(none, hold.words(none, due = false, now = 30))
        assertEquals(1, cancels)
        // A later refill due with nothing drawn keeps nothing.
        assertEquals(none, hold.words(none, due = true, now = 50))
        assertEquals(1, wakes.size)
    }
}
