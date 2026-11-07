package com.jtn0123.audiowhisper.gif

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class GiphyTest {
    private fun item(id: String, images: String) =
        """{"id":"$id","title":"Cat $id","url":"https://giphy.com/gifs/$id","rating":"g","images":{$images}}"""

    private val fixedWidth =
        """"fixed_width":{"url":"https://media.giphy.com/a/200w.gif","webp":"https://media.giphy.com/a/200w.webp","width":"200","height":"100","size":"1"}"""
    private val downsized = """"downsized":{"url":"https://media.giphy.com/a/giphy-downsized.gif","width":"480","height":"240"}"""

    @Test
    fun `parses the preview, its shape and the GIF to send`() {
        val gifs = Giphy.parse("""{"data":[${item("abc", "$fixedWidth,$downsized")}],"pagination":{},"meta":{}}""")
        assertEquals(
            listOf(
                Gif(
                    id = "abc",
                    title = "Cat abc",
                    previewUrl = "https://media.giphy.com/a/200w.webp",
                    aspectRatio = 2f,
                    sendUrl = "https://media.giphy.com/a/giphy-downsized.gif",
                    pageUrl = "https://giphy.com/gifs/abc",
                ),
            ),
            gifs,
        )
    }

    @Test
    fun `falls back to the fixed-width GIF when there is no smaller rendition`() {
        val gif = Giphy.parse("""{"data":[${item("abc", fixedWidth)}]}""").single()
        assertEquals("https://media.giphy.com/a/200w.gif", gif.sendUrl)
    }

    @Test
    fun `skips GIFs without a usable preview, and repeats`() {
        val noSize = """"fixed_width":{"url":"https://media.giphy.com/b.gif","width":"","height":"0"}"""
        val body = """{"data":[${item("a", fixedWidth)},${item("b", noSize)},${item("c", downsized)},${item("a", fixedWidth)}]}"""
        assertEquals(listOf("a"), Giphy.parse(body).map { it.id })
    }

    @Test
    fun `an empty result is no GIFs`() {
        assertTrue(Giphy.parse("""{"data":[],"meta":{"status":200}}""").isEmpty())
    }

    @Test
    fun `search URLs carry the key, a rating and an encoded query`() {
        val url = Giphy("KEY123").url("search", mapOf("q" to "thumbs up & ok"))
        assertTrue(url.startsWith("https://api.giphy.com/v1/gifs/search?q=thumbs+up+%26+ok&"), url)
        assertTrue(url.contains("api_key=KEY123") && url.contains("rating=pg-13") && url.contains("limit=30"), url)
    }
}
