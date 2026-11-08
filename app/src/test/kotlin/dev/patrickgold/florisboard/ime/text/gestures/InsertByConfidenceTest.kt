package dev.patrickgold.florisboard.ime.text.gestures

import dev.patrickgold.florisboard.ime.text.gestures.StatisticalGlideTypingClassifier.Companion.insertByConfidence
import kotlin.test.Test
import kotlin.test.assertEquals

class InsertByConfidenceTest {
    private val words = arrayListOf<String>()
    private val weights = arrayListOf<Float>()

    private fun insert(word: String, weight: Float, max: Int = 3) =
        insertByConfidence(words, weights, word, weight, max)

    @Test
    fun keepsTheMostLikelyFirstUpToTheLimit() {
        insert("poll", 4f)
        insert("pill", 2f)
        insert("pull", 6f)
        insert("pal", 3f)

        assertEquals(listOf("pill", "pal", "poll"), words)
        assertEquals(listOf(2f, 3f, 4f), weights)
    }

    @Test
    fun aLessLikelyCopyIsIgnored() {
        insert("pool", 3f)
        insert("poll", 4f)
        insert("pool", 5f)

        assertEquals(listOf("pool", "poll"), words)
        assertEquals(listOf(3f, 4f), weights)
    }

    @Test
    fun aMoreLikelyCopyReplacesTheFirst() {
        // A doubled letter's second gesture, with the loop, scoring better than the plain one.
        insert("pool", 5f)
        insert("poll", 4f)
        insert("pool", 3f)

        assertEquals(listOf("pool", "poll"), words)
        assertEquals(listOf(3f, 4f), weights)
    }

    @Test
    fun aMoreLikelyCopyFreesItsSlotForTheNextWord() {
        insert("a", 1f)
        insert("b", 2f)
        insert("pool", 5f)
        insert("pool", 1.5f)
        insert("c", 3f)

        assertEquals(listOf("a", "pool", "b"), words)
        insert("d", 2.5f, max = 4)
        assertEquals(listOf("a", "pool", "b", "d"), words)
    }
}
