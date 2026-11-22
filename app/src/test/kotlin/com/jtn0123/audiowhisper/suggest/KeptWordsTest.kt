package com.jtn0123.audiowhisper.suggest

import java.io.File
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class KeptWordsTest {
    private val dir: File = Files.createTempDirectory("kept-words").toFile()
    private val file = File(dir, "kept.txt")

    @AfterTest
    fun cleanUp() {
        dir.deleteRecursively()
    }

    @Test
    fun remembersAcrossRestartsIgnoringCase() {
        val kept = KeptWords(file)
        assertTrue(kept.store.add("Teh", remember = true))
        assertTrue(kept.save())
        val restarted = KeptWords(file).apply { load() }
        assertTrue("teh" in restarted)
        assertTrue("TEH" in restarted)
        assertFalse("the" in restarted)
    }

    @Test
    fun neverWritesAnIncognitoWord() {
        val kept = KeptWords(file)
        assertFalse(kept.store.add("secret", remember = false))
        assertTrue("secret" in kept)
        kept.store.add("teh", remember = true)
        kept.save()
        val restarted = KeptWords(file).apply { load() }
        assertFalse("secret" in restarted)
        assertTrue("teh" in restarted)
        assertFalse(file.readText().contains("secret"))
    }

    @Test
    fun keepsTheNewestWhenFull() {
        val kept = KeptWords(file, capacity = 2)
        kept.store.add("one", remember = true)
        kept.store.add("two", remember = true)
        kept.store.add("one", remember = true) // used again: now the newest
        kept.store.add("three", remember = true)
        assertFalse("two" in kept)
        assertTrue("one" in kept && "three" in kept)
    }

    @Test
    fun loadingKeepsWordsAddedBeforeIt() {
        KeptWords(file).apply { store.add("saved", remember = true); save() }
        val kept = KeptWords(file)
        kept.store.add("early", remember = true)
        kept.load()
        assertTrue("saved" in kept && "early" in kept)
        kept.save()
        assertEquals(listOf("saved", "early"), file.readLines().filter(String::isNotEmpty))
    }

    @Test
    fun aMissingFileLoadsNothing() {
        val kept = KeptWords(File(dir, "absent.txt")).apply { load() }
        assertFalse("anything" in kept)
        assertFalse(KeptWords(null).save())
    }
}
