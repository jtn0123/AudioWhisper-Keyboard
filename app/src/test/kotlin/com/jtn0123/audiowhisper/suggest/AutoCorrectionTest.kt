package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class AutoCorrectionTest {
    private val correction = RawSuggestion.KIND_CORRECTION or RawSuggestion.FLAG_APPROPRIATE_FOR_AUTO_CORRECTION

    private fun exceeds(suggestion: RawSuggestion, typed: String, threshold: Float) =
        SuggestRules.exceedsThreshold(suggestion.word, suggestion.score, suggestion.kindAndFlags, typed, threshold)

    private fun distance(before: String, after: String) =
        SuggestRules.editDistance(before.codePoints().toArray(), after.codePoints().toArray())

    @Test
    fun adjacentSwapCostsOne() {
        assertEquals(1, distance("teh", "the"))
        assertEquals(1, distance("recieve", "receive"))
    }

    @Test
    fun insertDeleteAndSubstituteCostOneEach() {
        assertEquals(1, distance("dont", "don't"))
        assertEquals(1, distance("thw", "the"))
        assertEquals(2, distance("helo", "help!"))
        assertEquals(3, distance("", "abc"))
    }

    @Test
    fun caseAndAccentsAreFree() {
        assertEquals(0, distance("Teh", "teh"))
        assertEquals(0, distance("cafe", "café"))
        assertEquals(1, distance("TEH", "the"))
    }

    @Test
    fun normalizedScoreMatchesTheNativeFormula() {
        // Values measured from the LatinIME engine (the C++, which the Rust port matches): "teh" -> "the" scored 477893, normalized 0.319.
        assertEquals(0.3186f, SuggestRules.normalizedScore("teh", "the", 477_893), 0.0005f)
        assertEquals(0.6377f, SuggestRules.normalizedScore("dont", "don't", 797_125), 0.0005f)
    }

    @Test
    fun normalizedScoreIsZeroForDegenerateInput() {
        assertEquals(0f, SuggestRules.normalizedScore("", "the", 500_000))
        assertEquals(0f, SuggestRules.normalizedScore("teh", "", 500_000))
        assertEquals(0f, SuggestRules.normalizedScore("teh", "  ", 500_000))
        assertEquals(0f, SuggestRules.normalizedScore("teh", "the", 0))
        // Editing every letter of the suggestion ("u" -> "I") is no correction at all.
        assertEquals(0f, SuggestRules.normalizedScore("u", "I", 484_164))
    }

    @Test
    fun scoresJustAboveTheThresholdPassAndJustBelowFail() {
        val atThreshold = (SuggestRules.modestThreshold * 1_000_000 * 3 / 2).toInt() + 1
        val suggestion = RawSuggestion("the", atThreshold, correction)
        assertTrue(exceeds(suggestion, "teh", SuggestRules.modestThreshold))
        val below = suggestion.copy(score = atThreshold - 10)
        assertFalse(exceeds(below, "teh", SuggestRules.modestThreshold))
    }

    @Test
    fun modestThresholdSeparatesMeasuredCases() {
        // "thx" -> "the" (0.231) passes; "yeah" -> "year" (0.177) does not.
        val the = RawSuggestion("the", 345_784, correction)
        val year = RawSuggestion("year", 235_587, correction)
        assertTrue(exceeds(the, "thx", SuggestRules.modestThreshold))
        assertFalse(exceeds(year, "yeah", SuggestRules.modestThreshold))
    }

    @Test
    fun whitelistAlwaysPassesAndInappropriateNeverDoes() {
        val whitelisted = RawSuggestion("I'm", 1, RawSuggestion.KIND_WHITELIST)
        assertTrue(exceeds(whitelisted, "im", SuggestRules.modestThreshold))
        val inappropriate = RawSuggestion("the", 2_000_000, RawSuggestion.KIND_CORRECTION)
        assertFalse(exceeds(inappropriate, "teh", SuggestRules.modestThreshold))
    }
}
