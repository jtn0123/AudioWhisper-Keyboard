package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/**
 * GIPHY's URLs and answers through JNI: GIFs back as the caller makes them, and bad input thrown.
 * The reading itself is tested in Rust and, against the Kotlin it replaces, in the app.
 */
class GiphyRulesTest {
    private fun parse(body: String) = GiphyRules.parse(body) { id, title, preview, ratio, send, page ->
        listOf(id, title, preview, ratio, send, page)
    }

    @Test
    fun gifsComeBackInOrderWithEveryField() {
        val gifs = parse(
            """{"data":[
                {"id":"a","title":"Ä 😀","url":"pa","images":{"fixed_width":{"url":"u","webp":"w","width":"3","height":"2"},"downsized":{"url":"d"}}},
                {"id":"b","images":{"fixed_width":{"url":"u2","width":"1","height":"4"}}}
            ]}""",
        )
        assertEquals(listOf(listOf("a", "Ä 😀", "w", 1.5f, "d", "pa"), listOf("b", "", "u2", 0.25f, "u2", "")), gifs)
        assertTrue(parse("{}").isEmpty())
    }

    @Test
    fun anUnreadableAnswerThrows() {
        assertFailsWith<IllegalArgumentException> { parse("not json") }
        assertFailsWith<IllegalArgumentException> { parse("""{"data":[1]}""") }
    }

    @Test
    fun urlsCarryTheKeyAndSettings() {
        assertEquals(
            "https://api.giphy.com/v1/gifs/trending?api_key=k&limit=30&rating=pg-13&lang=en",
            GiphyRules.trendingUrl("k"),
        )
        assertEquals(
            "https://api.giphy.com/v1/gifs/search?q=a+%26+b&api_key=k&limit=30&rating=pg-13&lang=en",
            GiphyRules.searchUrl("k", "a & b"),
        )
        assertEquals(
            "https://api.giphy.com/v1/gifs/e?rating=pg-13&q=x&api_key=k&limit=30&lang=en",
            GiphyRules.url("k", "e", linkedMapOf("rating" to "g", "q" to "x")),
        )
        assertTrue(GiphyRules.searchUrl("k", "x".repeat(60)).contains("q=${"x".repeat(50)}&"))
    }
}
