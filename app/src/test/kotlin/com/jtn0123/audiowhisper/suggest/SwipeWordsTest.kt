package com.jtn0123.audiowhisper.suggest

import kotlin.math.exp
import kotlin.math.ln
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotSame
import kotlin.test.assertSame
import kotlin.test.assertTrue

class SwipeWordsTest {
    private val dictionary = linkedMapOf("the" to 222, "I'm" to 185, "e-mail" to 90, "MP3" to 80, "rare" to 0)
    private val fromDictionary = SwipeWords.fromDictionary(dictionary.keys.toList()) { dictionary[it] ?: NOT_A_PROBABILITY }

    private fun weight(probability: Int) = SwipeWords.frequencyOf(probability)

    @Test
    fun theDictionarysWordsAreSwipeableExceptThoseALettersLayoutCantSpell() {
        assertEquals(listOf("the", "I'm", "e-mail", "rare"), fromDictionary.words)
    }

    @Test
    fun commonWordsCountTheSameAndRarerOnesLessAndLess() {
        assertEquals(1.0, SwipeWords.frequencyOf(255))
        assertEquals(1.0, SwipeWords.frequencyOf(SwipeWords.COMMON))
        assertEquals(exp(-0.007 * 70), SwipeWords.frequencyOf(100), 1e-9)
        assertEquals(exp(-0.007 * 170), SwipeWords.frequencyOf(0), 1e-9)
        assertEquals(1.0, fromDictionary.frequency("the"))
        assertEquals(SwipeWords.frequencyOf(0), fromDictionary.frequency("rare"))
        assertEquals(0.0, fromDictionary.frequency("tou"), "an unknown word has none")
    }

    @Test
    fun wordsWithTheSamePathStayCloseSoContextCanSettleThem() {
        // "to" (215) and "too" (160): one use of "me too" is a 0.31 bonus (see GlideRanker).
        val gap = ln(SwipeWords.frequencyOf(215) / SwipeWords.frequencyOf(160))
        assertTrue(gap in 0.0..0.1, "gap $gap")
    }

    @Test
    fun theUsersWordsAreAddedAndTheHigherProbabilityCounts() {
        val personal = listOf(PersonalWords.Entry("Zorblax", 250), PersonalWords.Entry("the", 10))
        val words = fromDictionary.withUserWords(SwipeWords.userWords(personal, learned = listOf("yeet", "rare")))
        assertEquals(listOf("the", "I'm", "e-mail", "rare"), words.words.take(4))
        assertEquals(setOf("Zorblax", "yeet"), words.words.drop(4).toSet(), "words the dictionary lacks come after")
        assertEquals(weight(250), words.frequency("Zorblax"))
        assertEquals(weight(SwipeWords.LEARNED_PROBABILITY), words.frequency("yeet"))
        assertEquals(weight(SwipeWords.LEARNED_PROBABILITY), words.frequency("rare"), "a word the user uses is common for them")
        assertEquals(weight(222), words.frequency("the"), "a lower personal frequency doesn't demote a word")
    }

    @Test
    fun theSameUserWordsKeepTheSameListSoTheClassifierDoesntReIndex() {
        val user = SwipeWords.userWords(emptyList(), listOf("yeet"))
        val words = fromDictionary.withUserWords(user)
        assertSame(words, words.withUserWords(SwipeWords.userWords(emptyList(), listOf("yeet"))))
        val forgotten = words.withUserWords(emptyMap())
        assertNotSame(words, forgotten)
        assertFalse("yeet" in forgotten.words, "a forgotten word can't be swiped any more")
        assertEquals(0.0, forgotten.frequency("yeet"))
    }

    @Test
    fun aPersonalEntryOutranksTheSameLearnedWordOnlyWhenHigher() {
        val user = SwipeWords.userWords(
            personal = listOf(PersonalWords.Entry("Kaito", 200), PersonalWords.Entry("bae", 5), PersonalWords.Entry("R2D2", 255)),
            learned = listOf("Kaito", "bae"),
        )
        assertEquals(mapOf("Kaito" to 200, "bae" to SwipeWords.LEARNED_PROBABILITY), user)
    }

    @Test
    fun onlyLettersWithInnerConnectorsAreSwipeable() {
        assertTrue(SwipeWords.isSwipeable("don't"))
        assertTrue(SwipeWords.isSwipeable("rock’n’roll"))
        assertTrue(SwipeWords.isSwipeable("café"))
        assertFalse(SwipeWords.isSwipeable("US$"))
        assertFalse(SwipeWords.isSwipeable("'"))
        assertFalse(SwipeWords.isSwipeable(""))
    }
}
