package com.jtn0123.audiowhisper.suggest

import java.io.File
import java.nio.file.Files
import kotlin.random.Random
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The word stores in Rust give exactly what the Kotlin they replaced gave ([ReferenceStores]),
 * through [StoreComparison]: random runs of learning, keeping, saving and loading damaged files,
 * and random personal dictionaries. The same comparison runs on the emulator, whose Java judges
 * some characters differently.
 */
class WordStoresMatchTest {
    private val dir: File = Files.createTempDirectory("word-stores").toFile()

    @AfterTest
    fun cleanUp() {
        dir.deleteRecursively()
    }

    private fun assertNone(body: StoreComparison.() -> Unit) {
        val comparison = StoreComparison(sigmaExact = false, dir).apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
    }

    @Test
    fun learnedWordsAnswerTheSame() = assertNone { learned(Random(SEED), runs = 2_000, steps = 60) }

    @Test
    fun keptWordsAnswerTheSame() = assertNone { kept(Random(SEED), runs = 2_000, steps = 40) }

    @Test
    fun personalWordsAnswerTheSame() = assertNone { personal(Random(SEED), 100_000) }

    @Test
    fun everyTagAppliesAlike() = assertNone { languages() }

    private companion object {
        const val SEED = 20261108L
    }
}
