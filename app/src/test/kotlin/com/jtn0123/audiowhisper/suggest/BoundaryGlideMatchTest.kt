package com.jtn0123.audiowhisper.suggest

import java.io.File
import java.util.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class BoundaryGlideMatchTest {
    @Test fun nativeContextAndRankMatchFrozenPerGuessQueries() {
        val file = File("src/main/assets/dicts/main_en-US.dict")
        checkNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US")).use { engine ->
            val random = Random(20261114L)
            val history = LearnedWords(null)
            val words = listOf("the", "The", "hello", "world", "you", "too", "to", "off", "", "\ud800")
            repeat(500) { history.add(words[random.nextInt(words.size)], words[random.nextInt(words.size)]) }
            repeat(10_000) {
                val previous = PreviousWords(List(random.nextInt(4)) { words[random.nextInt(words.size)] }, random.nextBoolean())
                val guesses = List(random.nextInt(48)) { GlideGuess(words[random.nextInt(words.size)],
                    when (random.nextInt(8)) {
                        0 -> Float.NaN
                        1 -> Float.POSITIVE_INFINITY
                        2 -> Float.NEGATIVE_INFINITY
                        3 -> -0f
                        else -> random.nextInt(10) / 4f
                    }) }
                val lookup = engine.takeIf { random.nextBoolean() }
                assertEquals(ReferenceBoundaryGlideRanker(history, lookup).rank(guesses, previous),
                    GlideRanker(history, lookup).rank(guesses, previous))
            }
        }
    }
}
