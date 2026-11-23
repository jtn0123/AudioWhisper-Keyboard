package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import java.util.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class BoundaryLearnerMatchTest {
    @Test fun nativeSessionLearningMatchesFrozenCallbackPath() {
        val tape = GoldenTape.ofReference("boundary-learner-session")
        for (file in TestDictionaries.both) nativeSessionLearningMatches(file, tape)
        tape.finish()
    }

    private fun nativeSessionLearningMatches(file: File, tape: GoldenTape) {
        checkNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US")).use { engine ->
            repeat(100) { seed ->
                val random = Random(20261114L + seed)
                val expectedStore = LearnedWords(null, 32, 32)
                val kotlinStore = LearnedWords(null, 32, 32)
                val actualStore = LearnedWords(null, 32, 32)
                var dictionary = true
                val expected = ReferenceBoundaryLearner(expectedStore) { engine.takeIf { dictionary } }
                val kotlin = ReferenceLatin.WordLearner(kotlinStore, { engine.takeIf { dictionary } })
                val actual = WordLearner(actualStore) { engine.takeIf { dictionary } }
                repeat(300) {
                    val word = listOf("The", "HELLO", "John", "USA", "Zorblax", "mp3", "can't", "", "\ud800")[random.nextInt(9)]
                    val before = listOf("", "say ", "hello ", "one. ", "x, ")[random.nextInt(5)]
                    val start = random.nextInt(8) - 1
                    dictionary = random.nextBoolean()
                    when (random.nextInt(5)) {
                        0 -> {
                            val result = kotlin.finished(word, before, start)
                            tape.answer("finished", result)
                            assertEquals(result, expected.finished(word, before, start))
                            assertEquals(result, actual.finished(word, before, start))
                        }
                        1 -> {
                            val result = kotlin.swiped(word, before)
                            tape.answer("swiped", result)
                            assertEquals(result, expected.swiped(word, before))
                            assertEquals(result, actual.swiped(word, before))
                        }
                        2 -> {
                            val result = kotlin.undo(start)
                            tape.answer("undo", result)
                            assertEquals(result, expected.undo(start))
                            assertEquals(result, actual.undo(start))
                        }
                        3 -> { kotlin.reset(); expected.reset(); actual.reset() }
                        4 -> { kotlin.dropSwiped(); expected.dropSwiped(); actual.dropSwiped() }
                    }
                    tape.answer("saved", kotlinStore.store.savedText())
                    assertEquals(kotlinStore.store.savedText(), expectedStore.store.savedText(), "Kotlin seed $seed")
                    assertEquals(expectedStore.store.savedText(), actualStore.store.savedText(), "${file.name} seed $seed")
                }
            }
        }
    }

    @Test fun closedDictionaryIsAskedOnlyWhenSpellingNeedsIt() {
        val file = File("src/main/assets/dicts/main_en-US.dict")
        val engine = checkNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US"))
        engine.close()
        val oldStore = LearnedWords(null)
        val newStore = LearnedWords(null)
        val old = ReferenceBoundaryLearner(oldStore) { engine }
        val native = WordLearner(newStore) { engine }
        val tape = GoldenTape.ofReference("boundary-learner-closed")
        val finished = old.finished("hello", "", 0)
        tape.answer("finished", finished)
        assertEquals(finished, native.finished("hello", "", 0))
        val expected = assertFailsWith<IllegalArgumentException> { old.finished("HELLO", "say ", 4) }
        tape.answer("threw", expected.message)
        val actual = assertFailsWith<IllegalArgumentException> { native.finished("HELLO", "say ", 4) }
        assertEquals(expected.message, actual.message)
        tape.answer("saved", oldStore.store.savedText())
        assertEquals(oldStore.store.savedText(), newStore.store.savedText())
        tape.finish()
    }
}
