package com.jtn0123.audiowhisper.gif

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** GIPHY's answers as the panel's session reads them (crates/gif), through the calls Kotlin makes. */
class GiphyTest {
    private val rig = SessionRig()

    private fun item(id: String, images: String) =
        """{"id":"$id","title":"Cat $id","url":"https://giphy.com/gifs/$id","rating":"g","images":{$images}}"""

    private val fixedWidth =
        """"fixed_width":{"url":"https://media.giphy.com/a/200w.gif","webp":"https://media.giphy.com/a/200w.webp","width":"200","height":"100","size":"1"}"""
    private val downsized = """"downsized":{"url":"https://media.giphy.com/a/giphy-downsized.gif","width":"480","height":"240"}"""

    private fun read(body: String): GifSearch.Results = rig.results("cats", FakeReply.Body(200, body.encodeToByteArray()))

    private fun gifs(body: String): List<Gif> = (read(body) as GifSearch.Results.Loaded).gifs

    private fun problem(reply: FakeReply): String {
        rig.session.load("dogs ${rig.source.api.size}")
        rig.answer(reply)
        return (rig.session.results.value as GifSearch.Results.Failed).message
    }

    @Test
    fun `parses the preview, its shape and the GIF to send`() {
        val gif = Gif(
            id = "abc",
            title = "Cat abc",
            previewUrl = "https://media.giphy.com/a/200w.webp",
            aspectRatio = 2f,
            sendUrl = "https://media.giphy.com/a/giphy-downsized.gif",
            pageUrl = "https://giphy.com/gifs/abc",
        )
        assertEquals(listOf(gif), gifs("""{"data":[${item("abc", "$fixedWidth,$downsized")}],"pagination":{},"meta":{}}"""))
    }

    @Test
    fun `falls back to the fixed-width GIF when there is no smaller rendition`() {
        assertEquals("https://media.giphy.com/a/200w.gif", gifs("""{"data":[${item("abc", fixedWidth)}]}""").single().sendUrl)
    }

    @Test
    fun `skips GIFs without a usable preview, and repeats`() {
        val noSize = """"fixed_width":{"url":"https://media.giphy.com/b.gif","width":"","height":"0"}"""
        val body = """{"data":[${item("a", fixedWidth)},${item("b", noSize)},${item("c", downsized)},${item("a", fixedWidth)}]}"""
        assertEquals(listOf("a"), gifs(body).map { it.id })
    }

    @Test
    fun `an empty result is no GIFs`() {
        assertTrue(gifs("""{"data":[],"meta":{"status":200}}""").isEmpty())
    }

    @Test
    fun `search URLs carry the key, a rating and an encoded query`() {
        rig.session.load("thumbs up & ok")
        rig.threads.drain()
        val url = rig.source.api.single().url
        assertTrue(url.startsWith("https://api.giphy.com/v1/gifs/search?q=thumbs+up+%26+ok&"), "search URL")
        assertTrue(url.contains("api_key=KEY123") && url.contains("rating=pg-13") && url.contains("limit=30"), "search URL")
    }

    @Test
    fun `a body that isn't the expected JSON is a format error, not a crash`() {
        val bodies = listOf("", "<html>502 Bad Gateway</html>", "null", """{"data":{}}""", """{"data":[1,2]}""", """{"data":[""")
        for (body in bodies) assertEquals(UNREADABLE, problem(FakeReply.Body(200, body.encodeToByteArray())))
    }

    @Test
    fun `skips GIFs with no id or no images instead of failing the whole page`() {
        val body = """{"data":[{"title":"no id","images":{$fixedWidth}},{"id":"b"},${item("c", fixedWidth)}]}"""
        assertEquals(listOf("c"), gifs(body).map { it.id })
    }

    @Test
    fun `each kind of failure gets its own message`() {
        assertEquals(UNREADABLE, problem(FakeReply.TooLarge))
        assertEquals("GIPHY didn't accept the API key.", problem(FakeReply.Status(401)))
        assertEquals("GIPHY didn't accept the API key.", problem(FakeReply.Status(403)))
        assertEquals("GIPHY's hourly limit is used up. Try again later.", problem(FakeReply.Status(429)))
        assertEquals("GIPHY isn't answering right now.", problem(FakeReply.Status(500)))
        assertEquals("Couldn't reach GIPHY. Check your connection.", problem(FakeReply.Unreachable))
        assertEquals(List(6) { "Could not load GIFs" }, rig.logs, "each failure is logged, naming nothing")
    }

    @Test
    fun `a reply is read as UTF-8`() {
        val body = """{"data":[${item("abc", fixedWidth).replace("Cat abc", "café")}]}"""
        assertEquals("café", gifs(body).single().title)
    }

    private companion object {
        const val UNREADABLE = "GIPHY sent something unreadable. Try again later."
    }
}
