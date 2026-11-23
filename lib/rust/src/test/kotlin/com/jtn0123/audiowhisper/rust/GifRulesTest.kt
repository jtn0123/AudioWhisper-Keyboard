package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

/**
 * The GIF panel's rules through JNI, as the keyboard calls them: text in and out, and bad input
 * thrown. The rules themselves are tested in Rust and, against the Kotlin they replace, in the
 * app; the session that runs them is GifSessionPlanTest's.
 */
class GifRulesTest {
    @Test
    fun theNumbersAreKotlins() {
        assertEquals(50, GifRules.maxLength)
        assertEquals("Trending", GifRules.trending)
        assertEquals("Recents", GifRules.recents)
        assertEquals(14, GifRules.categories.size)
    }

    @Test
    fun theSearchIsEdited() {
        assertEquals("happy dance", GifRules.typeWord("happy", "dance"))
        assertEquals("a".repeat(50), GifRules.type("a".repeat(50), "b"))
        assertEquals("ca", GifRules.deleteLast("cat 😀", 3))
        assertEquals("happy ", GifRules.deleteWords("happy dance", 1))
        // Half an emoji survives the trip, as a Kotlin String holds it.
        assertEquals("\uD83D", GifRules.deleteLast("\uD83Da", 1))
        assertEquals("congrats", GifRules.lastWord("Big congrats! "))
        assertNull(GifRules.lastWord(null))
        assertNull(GifRules.lastWord("see 😀"))
    }

    @Test
    fun theTypedWordLeadsTheChips() {
        assertEquals(listOf("Congrats", "Recents", "Trending"), GifRules.chips("congrats", true).take(3))
        assertEquals(GifRules.categories, GifRules.chips(null, false))
    }

    @Test
    fun keysAreRouted() {
        assertEquals(GifRules.Route.SEARCH, GifRules.route('a'.code, typing = true))
        assertEquals(GifRules.Route.KEYBOARD, GifRules.route(-11, typing = false))
        assertEquals(GifRules.Route.IGNORE, GifRules.route(9, typing = true))
        assertEquals(2, GifRules.swipeDelete(GifRules.Swipe.DELETE_CHARACTERS, -3))
        assertEquals(1, GifRules.swipeDelete(GifRules.Swipe.DELETE_WORDS, -5))
        assertEquals(0, GifRules.swipeDelete(GifRules.Swipe.SELECT, -9))
    }

    @Test
    fun failuresAreExplained() {
        assertEquals("GIPHY didn't accept the API key.", GifRules.problem(GifRules.Failure.STATUS, 403))
        assertEquals("Couldn't reach GIPHY. Check your connection.", GifRules.problem(GifRules.Failure.UNREACHABLE))
    }
}
