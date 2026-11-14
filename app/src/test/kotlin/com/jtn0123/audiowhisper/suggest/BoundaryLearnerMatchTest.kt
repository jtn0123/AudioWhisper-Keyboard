package com.jtn0123.audiowhisper.suggest

import java.io.File
import java.util.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class BoundaryLearnerMatchTest {
    @Test fun nativeSessionLearningMatchesFrozenCallbackPath() {
        val file = File("src/main/assets/dicts/main_en-US.dict")
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
                            assertEquals(result, expected.finished(word, before, start))
                            assertEquals(result, actual.finished(word, before, start))
                        }
                        1 -> {
                            val result = kotlin.swiped(word, before)
                            assertEquals(result, expected.swiped(word, before))
                            assertEquals(result, actual.swiped(word, before))
                        }
                        2 -> {
                            val result = kotlin.undo(start)
                            assertEquals(result, expected.undo(start))
                            assertEquals(result, actual.undo(start))
                        }
                        3 -> { kotlin.reset(); expected.reset(); actual.reset() }
                        4 -> { kotlin.dropSwiped(); expected.dropSwiped(); actual.dropSwiped() }
                    }
                    assertEquals(kotlinStore.store.savedText(), expectedStore.store.savedText(), "Kotlin seed $seed")
                    assertEquals(expectedStore.store.savedText(), actualStore.store.savedText(), "seed $seed")
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
        assertEquals(old.finished("hello", "", 0), native.finished("hello", "", 0))
        val expected = assertFailsWith<IllegalArgumentException> { old.finished("HELLO", "say ", 4) }
        val actual = assertFailsWith<IllegalArgumentException> { native.finished("HELLO", "say ", 4) }
        assertEquals(expected.message, actual.message)
        assertEquals(oldStore.store.savedText(), newStore.store.savedText())
    }
}
