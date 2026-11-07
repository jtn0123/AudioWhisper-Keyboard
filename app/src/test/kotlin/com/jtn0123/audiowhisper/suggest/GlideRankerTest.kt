package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals

/** Costs are the classifier's for clean swipes on the emulator. */
class GlideRankerTest {
    private val history = LearnedWords(null)

    /** The dictionary: probabilities by word, n-grams as (previous word, word) -> value. */
    private class FakeLookup(
        private val probabilities: Map<String, Int>,
        private val ngrams: Map<Pair<String, String>, Int>,
    ) : WordLookup {
        override fun probability(word: String) = probabilities[word] ?: NOT_A_PROBABILITY
        override fun ngramProbability(previous: PreviousWords, word: String): Int {
            val previousWord = previous.words.firstOrNull()?.lowercase() ?: return NOT_A_PROBABILITY
            return ngrams[previousWord to word] ?: NOT_A_PROBABILITY
        }
    }

    private val lookup = FakeLookup(
        probabilities = mapOf("to" to 215, "too" to 160, "of" to 214, "off" to 159, "you" to 171, "thou" to 152),
        ngrams = mapOf(("turn" to "off") to 200, ("thank" to "you") to 176),
    )
    private val ranker = GlideRanker(history, lookup)

    private val toPath = listOf(GlideGuess("to", 3.29f), GlideGuess("too", 3.30f), GlideGuess("rio", 3.59f))
    private val ofPath = listOf(GlideGuess("of", 3.29f), GlideGuess("off", 3.30f), GlideGuess("if", 3.46f))
    private val youPath = listOf(GlideGuess("you", 3.29f), GlideGuess("tou", 4.19f), GlideGuess("thou", 5.50f))

    private fun after(text: String) = PreviousWords.from(text)

    @Test
    fun withoutContextTheClassifiersOrderStands() {
        history.add("too", "me")
        assertEquals(listOf("to", "too", "rio"), ranker.rank(toPath, PreviousWords.NONE))
        assertEquals(listOf("to", "too", "rio"), ranker.rank(toPath, after("going ")))
    }

    @Test
    fun aHabitSettlesWordsWithTheSamePath() {
        history.add("too", "me")
        assertEquals(listOf("too", "to", "rio"), ranker.rank(toPath, after("me ")))
        assertEquals(listOf("too", "to", "rio"), ranker.rank(toPath, after("You and Me ")))
    }

    @Test
    fun aDictionaryNgramSettlesThemToo() {
        assertEquals(listOf("off", "of", "if"), ranker.rank(ofPath, after("turn ")))
        assertEquals(listOf("of", "off", "if"), ranker.rank(ofPath, after("out ")))
    }

    @Test
    fun contextNeverOverridesASwipeThatClearlyTracesAnotherWord() {
        repeat(1000) { history.add("thou", "thank") }
        assertEquals(listOf("you", "tou", "thou"), ranker.rank(youPath, after("thank ")))
    }

    @Test
    fun bonusesAreCapped() {
        repeat(1000) { history.add("off", "turn") }
        val bonus = ranker.bonus("off", after("turn "))
        assertEquals(GlideRanker.MAX_BONUS, bonus, absoluteTolerance = 0.001f)
        assertEquals(0f, ranker.bonus("off", after("out ")))
    }

    @Test
    fun aRareNgramStillCounts() {
        // "thank you" is the weakest n-gram there is: 5 steps above "you" alone.
        assertEquals(0.30f, ranker.bonus("you", after("thank ")), absoluteTolerance = 0.001f)
    }

    @Test
    fun guessesThatDoNotFitAtAllStayLast() {
        history.add("roo", "me")
        val guesses = listOf(GlideGuess("to", 3.29f), GlideGuess("roo", Float.POSITIVE_INFINITY))
        assertEquals(listOf("to", "roo"), ranker.rank(guesses, after("me ")))
    }

    @Test
    fun worksWithoutADictionary() {
        history.add("off", "turn")
        assertEquals(listOf("off", "of", "if"), GlideRanker(history, lookup = null).rank(ofPath, after("turn ")))
        assertEquals(listOf("of", "off", "if"), GlideRanker(TypingHistory.NONE, lookup = null).rank(ofPath, after("turn ")))
    }
}
