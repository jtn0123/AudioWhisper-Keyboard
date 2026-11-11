package com.jtn0123.audiowhisper.dictation

import java.io.File
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

class ModelCheckTest {
    private val dir: File = Files.createTempDirectory("model-check").toFile()

    private class Remembered : VerifiedFiles {
        val keys = mutableSetOf<String>()
        override fun contains(key: String) = key in keys
        override fun add(key: String) {
            keys += key
        }
    }

    @AfterTest
    fun cleanUp() {
        dir.deleteRecursively()
    }

    private fun write(name: String, text: String): ExpectedFile {
        val file = File(dir, name).apply { writeText(text) }
        return ExpectedFile(name, file.length(), ModelCheck.sha256(file))
    }

    @Test
    fun `sha256 matches a known value`() {
        val file = File(dir, "abc").apply { writeText("abc") }
        assertEquals("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", ModelCheck.sha256(file))
    }

    @Test
    fun `whole files are ready`() {
        val expected = listOf(write("a", "one"), write("b", "two"))
        assertEquals(ModelState.Ready, ModelCheck.quick(dir, expected))
        assertEquals(ModelState.Ready, ModelCheck.full(dir, expected, Remembered()))
    }

    @Test
    fun `absent files are missing`() {
        val expected = listOf(write("a", "one"), ExpectedFile("b", 3, "00"))
        assertEquals(ModelState.Missing(listOf("b")), ModelCheck.quick(dir, expected))
        assertEquals(ModelState.Missing(listOf("b")), ModelCheck.full(dir, expected, Remembered()))
    }

    @Test
    fun `a file cut short is damaged, even to the quick check`() {
        val expected = listOf(write("a", "one"))
        File(dir, "a").writeText("on")
        assertEquals(ModelState.Damaged(listOf("a")), ModelCheck.quick(dir, expected))
    }

    @Test
    fun `a file of the right size but other content is damaged to the full check only`() {
        val expected = listOf(write("a", "one"))
        File(dir, "a").writeText("two")
        assertEquals(ModelState.Ready, ModelCheck.quick(dir, expected))
        assertEquals(ModelState.Damaged(listOf("a")), ModelCheck.full(dir, expected, Remembered()))
    }

    @Test
    fun `a file is hashed once, and again once it changes`() {
        val expected = listOf(write("a", "one"))
        val remembered = Remembered()
        assertEquals(ModelState.Ready, ModelCheck.full(dir, expected, remembered))
        assertEquals(1, remembered.keys.size)
        File(dir, "a").apply { writeText("two"); setLastModified(lastModified() + 10_000) }
        assertEquals(ModelState.Damaged(listOf("a")), ModelCheck.full(dir, expected, remembered))
    }
}
