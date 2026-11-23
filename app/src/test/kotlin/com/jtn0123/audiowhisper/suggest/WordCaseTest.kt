package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class WordCaseTest {
    @Test
    fun detectsTheThreeCases() {
        assertEquals(SuggestRules.Case.AS_TYPED, SuggestRules.case("teh"))
        assertEquals(SuggestRules.Case.AS_TYPED, SuggestRules.case("iPhone"))
        assertEquals(SuggestRules.Case.CAPITALIZED, SuggestRules.case("Teh"))
        assertEquals(SuggestRules.Case.CAPITALIZED, SuggestRules.case("I"))
        assertEquals(SuggestRules.Case.ALL_CAPS, SuggestRules.case("TEH"))
        assertEquals(SuggestRules.Case.ALL_CAPS, SuggestRules.case("DON'T"))
        assertEquals(SuggestRules.Case.AS_TYPED, SuggestRules.case("McDonald"))
    }

    @Test
    fun appliesTheCaseToSuggestions() {
        assertEquals("the", SuggestRules.applyCase(SuggestRules.Case.AS_TYPED, "the"))
        assertEquals("NASA", SuggestRules.applyCase(SuggestRules.Case.AS_TYPED, "NASA"))
        assertEquals("The", SuggestRules.applyCase(SuggestRules.Case.CAPITALIZED, "the"))
        assertEquals("I'm", SuggestRules.applyCase(SuggestRules.Case.CAPITALIZED, "I'm"))
        assertEquals("IPhone", SuggestRules.applyCase(SuggestRules.Case.CAPITALIZED, "iPhone"))
        assertEquals("THE", SuggestRules.applyCase(SuggestRules.Case.ALL_CAPS, "the"))
        assertEquals("", SuggestRules.applyCase(SuggestRules.Case.CAPITALIZED, ""))
    }

    @Test
    fun decoderInputDropsOnlyCapitalsTheDecoderShouldIgnore() {
        assertEquals("im", SuggestRules.decoderInput("Im"))
        assertEquals("teh", SuggestRules.decoderInput("TEH"))
        assertEquals("iPhone", SuggestRules.decoderInput("iPhone"))
        assertEquals("teh", SuggestRules.decoderInput("teh"))
    }

    @Test
    fun mostlyCapsMeansMoreThanOneCapital() {
        assertFalse(SuggestRules.isMostlyCaps("Teh"))
        assertFalse(SuggestRules.isMostlyCaps("iPhone"))
        assertTrue(SuggestRules.isMostlyCaps("THe"))
        assertTrue(SuggestRules.isMostlyCaps("TEH"))
    }
}
