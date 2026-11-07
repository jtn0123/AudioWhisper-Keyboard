package com.jtn0123.audiowhisper.gif

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class GifResultCacheTest {
    private var now = 0L
    private val cache = GifResultCache(clock = { now }, ttlMs = 1_000, capacity = 2)

    private fun gifs(vararg ids: String) = ids.map { Gif(it, it, "p/$it", 1f, "s/$it", "u/$it") }

    @Test
    fun aSearchIsAnsweredFromTheCacheUntilItExpires() {
        cache.put("Haha", gifs("a", "b"))
        now = 999
        assertEquals(gifs("a", "b"), cache.get("Haha"))
        now = 1_000
        assertNull(cache.get("Haha"))
        assertNull(cache.get("Haha"), "an expired entry stays gone")
    }

    @Test
    fun theSameSearchInAnotherCaseOrWithSpacesIsTheSameEntry() {
        cache.put("Thank you", gifs("a"))
        assertEquals(gifs("a"), cache.get("thank you "))
        assertNull(cache.get("thank"))
    }

    @Test
    fun aNewerAnswerReplacesTheOld() {
        cache.put("cats", gifs("a"))
        now = 600
        cache.put("cats", gifs("b"))
        now = 1_500
        assertEquals(gifs("b"), cache.get("cats"))
    }

    @Test
    fun theLeastRecentlyUsedSearchGoesWhenFull() {
        cache.put("one", gifs("1"))
        cache.put("two", gifs("2"))
        cache.get("one")
        cache.put("three", gifs("3"))
        assertEquals(gifs("1"), cache.get("one"))
        assertNull(cache.get("two"))
        assertEquals(gifs("3"), cache.get("three"))
    }
}
