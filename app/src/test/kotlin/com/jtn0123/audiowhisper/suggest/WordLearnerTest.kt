package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class WordLearnerTest {
    private val history = LearnedWords(null)
    private val dictionary = setOf("the", "hello", "John", "later")
    private val lookup = object : WordLookup {
        override fun probability(word: String) = if (word in dictionary) 150 else NOT_A_PROBABILITY
        override fun ngramProbability(previous: PreviousWords, word: String) = NOT_A_PROBABILITY
    }
    private val learner = WordLearner(history, lookup = { lookup })

    private val sentenceStart = PreviousWords(emptyList(), isBeginningOfSentence = true)
    private fun after(word: String) = PreviousWords(listOf(word), isBeginningOfSentence = false)

    /** The words learned after [previous], spelled as stored. */
    private fun storedAfter(previous: PreviousWords) = history.followers(previous, minCount = 1, maxCount = 5)

    @Test
    fun learnsAFinishedWordAndTheWordBeforeIt() {
        assertTrue(learner.finished("later", "see you ", start = 8))
        assertEquals(1, history.pairCount(after("you"), "later"))
        assertTrue(learner.finished("later", "talk to you ", start = 20))
        assertEquals(2, history.pairCount(after("you"), "later"))
        assertEquals("later", history.spellingOf("later"))
    }

    @Test
    fun aWordFinishedTwiceInOnePlaceCountsOnce() {
        learner.finished("later", "see you ", start = 8)
        assertFalse(learner.finished("later", "see you ", start = 8))
        assertEquals(1, history.pairCount(after("you"), "later"))
    }

    @Test
    fun aWordChangedInPlaceReplacesTheOldOne() {
        learner.finished("teh", "", start = 0)
        assertTrue(learner.finished("the", "", start = 0))
        assertEquals(0, history.pairCount(sentenceStart, "teh"))
        assertEquals(1, history.pairCount(sentenceStart, "the"))
    }

    @Test
    fun undoingAnAutoCorrectionTakesItBack() {
        learner.finished("the", "", start = 0)
        assertFalse(learner.undo(start = 5))
        assertTrue(learner.undo(start = 0))
        assertEquals(0, history.pairCount(sentenceStart, "the"))
        assertFalse(learner.undo(start = 0))
    }

    @Test
    fun aSwipedWordIsKeptOnlyOnceTypingGoesOn() {
        assertFalse(learner.swiped("hello", ""))
        assertEquals(0, history.pairCount(sentenceStart, "hello"))
        learner.finished("world", "hello ", start = 6)
        assertEquals(1, history.pairCount(sentenceStart, "hello"))
        assertEquals(1, history.pairCount(after("hello"), "world"))
    }

    @Test
    fun aSecondSwipeKeepsTheFirst() {
        learner.swiped("hello", "")
        assertTrue(learner.swiped("world", "hello "))
        assertEquals(1, history.pairCount(sentenceStart, "hello"))
        assertEquals(0, history.pairCount(after("hello"), "world"))
    }

    @Test
    fun aSwipedWordFinishedAsItselfCountsOnce() {
        learner.swiped("hello", "")
        learner.finished("hello", "", start = 0)
        assertEquals(1, history.pairCount(sentenceStart, "hello"))
    }

    @Test
    fun aDroppedSwipeIsNeverLearned() {
        learner.swiped("helo", "")
        learner.dropSwiped()
        learner.finished("hello", "", start = 0)
        assertEquals(0, history.pairCount(sentenceStart, "helo"))
        assertEquals(1, history.pairCount(sentenceStart, "hello"))
    }

    @Test
    fun aSentenceStartCapitalIsNotLearned() {
        learner.finished("The", "", start = 0)
        assertEquals(listOf("the"), storedAfter(sentenceStart))
        // Mid-sentence, a capital is the user's choice.
        learner.finished("The", "see ", start = 4)
        assertEquals(listOf("The"), storedAfter(after("see")))
    }

    @Test
    fun capsLockIsNotLearnedForDictionaryWords() {
        learner.finished("HELLO", "say ", start = 4)
        learner.finished("NASA", "at ", start = 3)
        assertEquals(listOf("hello"), storedAfter(after("say")))
        assertEquals(listOf("NASA"), storedAfter(after("at")))
    }

    @Test
    fun keepsTheDictionarysOwnCapitals() {
        learner.finished("John", "", start = 0)
        assertEquals(listOf("John"), storedAfter(sentenceStart))
    }

    @Test
    fun aSentenceStartDoesNotRecapitaliseAnUnknownWord() {
        learner.finished("zorblax", "a ", start = 2)
        learner.finished("Zorblax", "", start = 0)
        assertEquals("zorblax", history.spellingOf("Zorblax"))
    }

    @Test
    fun anotherFieldForgetsWhereWordsWere() {
        learner.finished("teh", "", start = 0)
        learner.reset()
        learner.finished("the", "", start = 0)
        assertEquals(1, history.pairCount(sentenceStart, "teh"))
        assertEquals(1, history.pairCount(sentenceStart, "the"))
    }

    @Test
    fun onlyWordsAreLearned() {
        assertFalse(learner.finished("mp3", "", start = 0))
        assertFalse(learner.swiped("a1", ""))
        assertEquals(0, history.pairCount(sentenceStart, "mp3"))
    }

    @Test
    fun withoutADictionaryWordsAreKeptAsTyped() {
        val noDictionary = WordLearner(history, lookup = { null })
        noDictionary.finished("The", "", start = 0)
        assertEquals(listOf("The"), storedAfter(sentenceStart))
    }
}
