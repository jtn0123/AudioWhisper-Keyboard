package com.jtn0123.audiowhisper.gif

import kotlinx.coroutines.runBlocking
import java.io.File
import java.io.IOException
import java.nio.file.Files
import kotlin.coroutines.EmptyCoroutineContext
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** Sending a GIF as Rust's `gif::send` steps it: downloaded once, whole, kept a day, else its link. */
class GifSenderFilesTest {
    private val dir: File = Files.createTempDirectory("gifs").toFile()
    private var now = 1_000_000_000_000L
    private val day = 24 * 60 * 60 * 1000L
    private val gif = Gif("a/b-c", "title", "https://media.giphy.com/p.gif", 1f, "https://media.giphy.com/s.gif", "https://giphy.com/gifs/abc")
    private val logs = mutableListOf<String>()

    @AfterTest
    fun cleanUp() {
        dir.deleteRecursively()
    }

    private fun file(name: String, age: Long) = File(dir, name).apply {
        writeText(name)
        setLastModified(now - age)
    }

    private fun send(source: GifDataSource, target: FakeTarget, gif: Gif = this.gif) =
        runBlocking { GifSend(dir, EmptyCoroutineContext, { now }, source, target) { logs += it }.send(gif) }

    private fun body(bytes: String) = FakeSource("KEY") { FakeReply.Body(200, bytes.encodeToByteArray()) }

    @Test
    fun `a finished download is renamed into place and sent`() {
        val target = FakeTarget(acceptsGifs = true, takesGif = true, takesText = true)
        assertEquals(GifSender.Outcome.SENT, send(body("GIF89a"), target))
        assertEquals("GIF89a", File(dir, "abc.gif").readText())
        assertEquals(listOf("abc.gif"), dir.list()!!.toList())
        assertEquals(listOf("accepts", "gif abc.gif 6"), target.calls)
    }

    @Test
    fun `a failed download leaves nothing that looks cached`() {
        val cut = object : GifDataSource {
            override val key = "KEY"

            override suspend fun get(url: String, request: IntArray) = error("not asked")

            override suspend fun save(url: String, file: File, request: IntArray): Boolean {
                file.writeText("GIF8")
                throw IOException("cut off")
            }
        }
        val target = FakeTarget(acceptsGifs = true, takesGif = true, takesText = true)
        assertEquals(GifSender.Outcome.FAILED, send(cut, target))
        assertEquals(emptyList(), dir.list()!!.toList())
        assertEquals(listOf("accepts"), target.calls, "a failed download types no link")
        assertEquals(listOf("Could not download the GIF: IOException"), logs, "the log names nothing sent")
    }

    @Test
    fun `a cached GIF is reused and its day starts again`() {
        val cached = file("abc.gif", age = day - 1)
        val target = FakeTarget(acceptsGifs = true, takesGif = true, takesText = true)
        val never = FakeSource("KEY") { error("should not download again") }
        assertEquals(GifSender.Outcome.SENT, send(never, target))
        assertEquals(now, cached.lastModified())
        now += day - 1
        assertEquals(GifSender.Outcome.SENT, send(never, target))
        assertTrue(cached.exists())
    }

    @Test
    fun `sending drops day-old GIFs and stale parts, and keeps the rest`() {
        val old = file("old.gif", age = day + 1)
        val recent = file("recent.gif", age = day - 1)
        val stalePart = file("cut.gif.part", age = 61_000)
        val livePart = file("live.gif.part", age = 1_000)
        send(body("GIF89a"), FakeTarget(acceptsGifs = true, takesGif = true, takesText = true))
        assertFalse(old.exists())
        assertTrue(recent.exists())
        assertFalse(stalePart.exists())
        assertTrue(livePart.exists())
    }

    @Test
    fun `an app that takes no GIFs, or refuses this one, gets the link`() {
        val noGifs = FakeTarget(acceptsGifs = false, takesGif = true, takesText = true)
        assertEquals(GifSender.Outcome.LINKED, send(body("GIF89a"), noGifs))
        assertEquals(listOf("accepts", "text ${gif.pageUrl.length}"), noGifs.calls)
        assertFalse(dir.exists() && dir.list()!!.isNotEmpty(), "nothing downloaded for a link")
        val refuses = FakeTarget(acceptsGifs = true, takesGif = false, takesText = true)
        assertEquals(GifSender.Outcome.LINKED, send(body("GIF89a"), refuses))
        assertEquals(listOf("accepts", "gif abc.gif 6", "text ${gif.pageUrl.length}"), refuses.calls)
    }

    @Test
    fun `with no link to fall back on, or a link refused, nothing is sent`() {
        val noPage = gif.copy(pageUrl = "")
        assertEquals(GifSender.Outcome.FAILED, send(body("x"), FakeTarget(acceptsGifs = false, takesGif = true, takesText = true), noPage))
        assertEquals(GifSender.Outcome.FAILED, send(body("x"), FakeTarget(acceptsGifs = false, takesGif = true, takesText = false)))
        assertEquals(GifSender.Outcome.FAILED, send(FakeSource("KEY") { FakeReply.Status(404) }, FakeTarget(true, true, true)))
    }
}
