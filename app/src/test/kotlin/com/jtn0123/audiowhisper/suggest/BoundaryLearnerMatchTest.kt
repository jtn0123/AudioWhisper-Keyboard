package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import java.util.Random
import kotlin.test.Test
import kotlin.test.assertFailsWith

class BoundaryLearnerMatchTest {
    @Test fun nativeSessionLearningMatchesFrozenCallbackPath() {
        GoldenTape.check("boundary-learner-session") { tape ->
            for (file in TestDictionaries.both) nativeSessionLearningMatches(file, tape)
        }
    }

    private fun nativeSessionLearningMatches(file: File, tape: GoldenTape) {
        checkNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US")).use { engine ->
            repeat(100) { seed ->
                val random = Random(20261114L + seed)
                val actualStore = LearnedWords(null, 32, 32)
                var dictionary = true
                val actual = WordLearner(actualStore) { engine.takeIf { dictionary } }
                repeat(300) {
                    val word = listOf("The", "HELLO", "John", "USA", "Zorblax", "mp3", "can't", "", "\ud800")[random.nextInt(9)]
                    val before = listOf("", "say ", "hello ", "one. ", "x, ")[random.nextInt(5)]
                    val start = random.nextInt(8) - 1
                    dictionary = random.nextBoolean()
                    when (random.nextInt(5)) {
                        0 -> tape.answer("finished", actual.finished(word, before, start))
                        1 -> tape.answer("swiped", actual.swiped(word, before))
                        2 -> tape.answer("undo", actual.undo(start))
                        3 -> actual.reset()
                        4 -> actual.dropSwiped()
                    }
                    tape.answer("saved", actualStore.store.savedText())
                }
            }
        }
    }

    @Test fun closedDictionaryIsAskedOnlyWhenSpellingNeedsIt() {
        val file = File("src/main/assets/dicts/main_en-US.dict")
        val engine = checkNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US"))
        engine.close()
        val newStore = LearnedWords(null)
        val native = WordLearner(newStore) { engine }
        GoldenTape.check("boundary-learner-closed") { tape ->
            tape.answer("finished", native.finished("hello", "", 0))
            val actual = assertFailsWith<IllegalArgumentException> { native.finished("HELLO", "say ", 4) }
            tape.answer("threw", actual.message)
            tape.answer("saved", newStore.store.savedText())
        }
    }
}
