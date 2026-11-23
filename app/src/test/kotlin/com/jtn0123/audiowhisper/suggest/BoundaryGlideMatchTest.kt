package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import java.util.Random
import kotlin.test.Test
import kotlin.test.assertFailsWith

class BoundaryGlideMatchTest {
    @Test fun closedDictionaryIsNotQueriedForEarlyReturnCohorts() {
        val file = File("src/main/assets/dicts/main_en-US.dict")
        val engine = checkNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US"))
        engine.close()
        val history = LearnedWords(null)
        val native = GlideRanker(history, engine)
        val guesses = listOf(GlideGuess("hello", 1f), GlideGuess("world", 2f))
        val previous = PreviousWords(listOf("hello"), false)
        GoldenTape.check("boundary-glide-closed") { tape ->
            for ((words, context) in listOf(emptyList<GlideGuess>() to previous,
                guesses.take(1) to previous, guesses to PreviousWords.NONE)) {
                tape.answer("rank", native.rank(words, context))
            }
            val actual = assertFailsWith<IllegalArgumentException> { native.rank(guesses, previous) }
            tape.answer("threw", actual.message)
        }
    }

    @Test fun nativeContextAndRankMatchFrozenPerGuessQueries() {
        GoldenTape.check("boundary-glide-ranks") { tape ->
            for (file in TestDictionaries.both) nativeContextAndRankMatch(file, tape)
        }
    }

    private fun nativeContextAndRankMatch(file: File, tape: GoldenTape) {
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
                tape.answer("rank", GlideRanker(history, lookup).rank(guesses, previous))
            }
        }
    }
}
