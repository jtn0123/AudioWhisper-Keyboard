package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * The GIF panel's rules through JNI, as the keyboard calls them: text in and out, recents' order
 * as indexes, the cache behind a handle, and bad input thrown. The rules themselves are tested in
 * Rust and, against the Kotlin they replace, in the app.
 */
class GifRulesTest {
    @Test
    fun theNumbersAreKotlins() {
        assertEquals(50, GifRules.maxLength)
        assertEquals("Trending", GifRules.trending)
        assertEquals("Recents", GifRules.recents)
        assertEquals(14, GifRules.categories.size)
        assertEquals(30, GifRules.recentsCapacity)
        assertEquals(600_000L, GifResultKeys.TTL_MS)
        assertEquals(20, GifResultKeys.CAPACITY)
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

    @Test
    fun recentsAreOrdered() {
        val order = GifRules.recentsAdd(listOf("a", "b", "c"), "b", 30)
        assertContentEquals(intArrayOf(-1, 0, 2), order)
        assertTrue(GifRules.isSent(order[0]))
        assertEquals(2, GifRules.index(order[2]))
        assertContentEquals(intArrayOf(-1, -2, 0, 2), GifRules.recentsLoad(listOf("x", "a", "y"), listOf("a", "b"), 30))
        assertEquals(1, GifRules.index(-2))
        assertFailsWith<IllegalArgumentException> { GifRules.recentsAdd(listOf("a"), "b", -1) }
    }

    @Test
    fun theCacheKeepsRecentSearches() {
        val keys = GifResultKeys(ttlMs = 100, capacity = 1)
        assertEquals("cats", GifResultKeys.key(" Cats "))
        assertEquals(emptyList(), keys.put("cats", 0))
        assertTrue(keys.get("cats", 99))
        assertFalse(keys.get("cats", 100))
        keys.put("dogs", 0)
        assertEquals(listOf("dogs"), keys.put("owls", 0))
        assertFailsWith<IllegalArgumentException> { GifResultKeys(capacity = -1) }
    }
}
