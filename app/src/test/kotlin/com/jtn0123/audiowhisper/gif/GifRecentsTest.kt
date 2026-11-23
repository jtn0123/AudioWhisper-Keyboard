package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRules
import java.io.File
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/** Recents: their order decided in the session (crates/gif), their file read and written here. */
class GifRecentsTest {
    private fun gif(id: String) = Gif(id, "title $id", "p/$id", 1.5f, "s/$id", "u/$id")

    private val dir: File = Files.createTempDirectory("gifs").toFile()
    private val file = File(dir, GifSearchSession.RECENTS_FILE)

    private fun SessionRig.recents(): List<String> {
        session.load(GifRules.recents)
        return (session.results.value as GifSearch.Results.Loaded).gifs.map { it.id }
    }

    private fun SessionRig.send(vararg ids: String) = ids.forEach { session.sent(gif(it), incognito = false) }

    @Test
    fun theLastSentComesFirstAndOnlyOnce() {
        val rig = SessionRig()
        rig.send("a", "b", "a")
        assertEquals(listOf("a", "b"), rig.recents())
    }

    @Test
    fun theOldestGoWhenFull() {
        val rig = SessionRig()
        val ids = (0..40).map { "g$it" }
        rig.send(*ids.toTypedArray())
        val kept = rig.recents()
        assertEquals(ids.reversed().take(kept.size), kept)
        assertTrue(kept.size < ids.size)
    }

    @Test
    fun savesAndReadsEverything() {
        assertTrue(GifRecents.write(file, listOf(gif("b"), gif("a"))))
        assertFalse(File(dir, "${file.name}.part").exists())
        assertEquals(listOf(gif("b"), gif("a")), GifRecents.read(file))
    }

    @Test
    fun whatIsSentIsSavedForNextTime() {
        val rig = SessionRig().apply { session.attach(dir) }
        rig.threads.drain()
        rig.send("a", "b")
        rig.threads.drain()
        assertEquals(listOf("b", "a"), GifRecents.read(file)?.map { it.id })
        val next = SessionRig().apply { session.attach(dir) }
        next.threads.drain()
        assertEquals(listOf("b", "a"), next.recents())
    }

    @Test
    fun readingKeepsWhatWasSentBeforeItFinished() {
        GifRecents.write(file, listOf(gif("both"), gif("old")))
        val rig = SessionRig().apply { session.attach(dir) }
        rig.threads.main.runAll()
        rig.send("new", "both")
        rig.threads.drain()
        assertEquals(listOf("both", "new", "old"), rig.recents())
        assertEquals(listOf("both", "new", "old"), GifRecents.read(file)?.map { it.id }, "and saved so")
    }

    @Test
    fun aMissingOrDamagedFileIsNoGifs() {
        assertNull(GifRecents.read(file))
        file.writeText("[{\"id\": 3")
        assertNull(GifRecents.read(file))
        val rig = SessionRig().apply { session.attach(dir) }
        rig.threads.drain()
        assertEquals(emptyList(), rig.recents())
        assertFalse(GifRecents.write(File(dir, "missing/folder.json"), emptyList()))
    }
}
