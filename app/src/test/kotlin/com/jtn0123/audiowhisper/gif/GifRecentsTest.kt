package com.jtn0123.audiowhisper.gif

import java.io.File
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class GifRecentsTest {
    private fun gif(id: String) = Gif(id, "title $id", "p/$id", 1.5f, "s/$id", "u/$id")

    private fun ids(recents: GifRecents) = recents.all().map { it.id }

    private fun tempFile(): File = Files.createTempDirectory("gifs").toFile().resolve("gif-recents.json")

    @Test
    fun theLastSentComesFirstAndOnlyOnce() {
        val recents = GifRecents(file = null)
        recents.add(gif("a"))
        recents.add(gif("b"))
        recents.add(gif("a"))
        assertEquals(listOf("a", "b"), ids(recents))
    }

    @Test
    fun theOldestGoWhenFull() {
        val recents = GifRecents(file = null, capacity = 2)
        listOf("a", "b", "c").forEach { recents.add(gif(it)) }
        assertEquals(listOf("c", "b"), ids(recents))
    }

    @Test
    fun savesAndLoadsEverything() {
        val file = tempFile()
        val recents = GifRecents(file)
        recents.add(gif("a"))
        recents.add(gif("b"))
        assertTrue(recents.save())
        assertFalse(File(file.parentFile, "${file.name}.part").exists())
        val reloaded = GifRecents(file).apply { load() }
        assertEquals(listOf(gif("b"), gif("a")), reloaded.all())
    }

    @Test
    fun loadKeepsWhatWasSentBeforeItFinished() {
        val file = tempFile()
        GifRecents(file).apply { add(gif("old")); add(gif("both")); save() }
        val recents = GifRecents(file)
        recents.add(gif("new"))
        recents.add(gif("both"))
        recents.load()
        assertEquals(listOf("both", "new", "old"), ids(recents))
    }

    @Test
    fun aMissingOrDamagedFileIsNoGifs() {
        val file = tempFile()
        GifRecents(file).load()
        file.writeText("[{\"id\": 3")
        val recents = GifRecents(file).apply { load() }
        assertEquals(emptyList(), recents.all())
        assertFalse(GifRecents(file = null).save())
    }
}
