package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import java.nio.file.Files
import kotlin.random.Random
import kotlin.test.AfterTest
import kotlin.test.Test

/**
 * The word stores in Rust give exactly what the Kotlin they replaced gave (its answers are golden
 * files, docs/phase6-kotlin-references.md), through [StoreComparison]: random runs of learning,
 * keeping, saving and loading damaged files, and random personal dictionaries.
 */
class WordStoresMatchTest {
    private val dir: File = Files.createTempDirectory("word-stores").toFile()

    @AfterTest
    fun cleanUp() {
        dir.deleteRecursively()
    }

    private fun assertGolden(name: String, body: StoreComparison.() -> Unit) {
        GoldenTape.check("word-stores-$name") { StoreComparison(sigmaExact = false, dir, it).apply(body) }
    }

    @Test
    fun learnedWordsAnswerTheSame() = assertGolden("learned") { learned(Random(SEED), runs = 2_000, steps = 60) }

    @Test
    fun keptWordsAnswerTheSame() = assertGolden("kept") { kept(Random(SEED), runs = 2_000, steps = 40) }

    @Test
    fun personalWordsAnswerTheSame() = assertGolden("personal") { personal(Random(SEED), 100_000) }

    @Test
    fun everyTagAppliesAlike() = assertGolden("languages") { languages() }

    private companion object {
        const val SEED = 20261108L
    }
}
