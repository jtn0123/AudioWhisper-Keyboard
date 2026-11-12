package com.jtn0123.audiowhisper.gif

import kotlinx.coroutines.runBlocking
import java.io.File
import java.io.IOException
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class GifSenderFilesTest {
    private val dir: File = Files.createTempDirectory("gifs").toFile()
    private val now = 1_000_000_000_000L
    private val day = 24 * 60 * 60 * 1000L

    @AfterTest
    fun cleanUp() {
        dir.deleteRecursively()
    }

    private fun file(name: String, age: Long) = File(dir, name).apply {
        writeText(name)
        setLastModified(now - age)
    }

    @Test
    fun `a finished download is renamed into place`() = runBlocking {
        val gif = File(dir, "abc.gif")
        GifSender.fetchInto(gif, now) { it.writeText("GIF89a") }
        assertEquals("GIF89a", gif.readText())
        assertEquals(listOf("abc.gif"), dir.list()!!.toList())
    }

    @Test
    fun `a failed download leaves nothing that looks cached`() = runBlocking {
        val gif = File(dir, "abc.gif")
        assertFailsWith<IOException> {
            GifSender.fetchInto(gif, now) {
                it.writeText("GIF8")
                throw IOException("cut off")
            }
        }
        assertFalse(gif.exists())
        assertEquals(emptyList(), dir.list()!!.toList())
    }

    @Test
    fun `a cached GIF is reused and its day starts again`() = runBlocking {
        val gif = file("abc.gif", age = day - 1)
        GifSender.fetchInto(gif, now) { error("should not download again") }
        assertEquals(now, gif.lastModified())
        GifSender.prune(dir, now + day - 1)
        assertTrue(gif.exists())
    }

    @Test
    fun `prune drops day-old GIFs and stale parts, and keeps the rest`() {
        val old = file("old.gif", age = day + 1)
        val recent = file("recent.gif", age = day - 1)
        val stalePart = file("cut.gif.part", age = 61_000)
        val livePart = file("live.gif.part", age = 1_000)
        GifSender.prune(dir, now)
        assertFalse(old.exists())
        assertTrue(recent.exists())
        assertFalse(stalePart.exists())
        assertTrue(livePart.exists())
    }
}
