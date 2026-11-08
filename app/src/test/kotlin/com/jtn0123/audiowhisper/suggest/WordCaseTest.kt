package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class WordCaseTest {
    @Test
    fun detectsTheThreeCases() {
        assertEquals(WordCase.AS_TYPED, WordCase.of("teh"))
        assertEquals(WordCase.AS_TYPED, WordCase.of("iPhone"))
        assertEquals(WordCase.CAPITALIZED, WordCase.of("Teh"))
        assertEquals(WordCase.CAPITALIZED, WordCase.of("I"))
        assertEquals(WordCase.ALL_CAPS, WordCase.of("TEH"))
        assertEquals(WordCase.ALL_CAPS, WordCase.of("DON'T"))
        assertEquals(WordCase.AS_TYPED, WordCase.of("McDonald"))
    }

    @Test
    fun appliesTheCaseToSuggestions() {
        assertEquals("the", WordCase.AS_TYPED.apply("the"))
        assertEquals("NASA", WordCase.AS_TYPED.apply("NASA"))
        assertEquals("The", WordCase.CAPITALIZED.apply("the"))
        assertEquals("I'm", WordCase.CAPITALIZED.apply("I'm"))
        assertEquals("IPhone", WordCase.CAPITALIZED.apply("iPhone"))
        assertEquals("THE", WordCase.ALL_CAPS.apply("the"))
        assertEquals("", WordCase.CAPITALIZED.apply(""))
    }

    @Test
    fun decoderInputDropsOnlyCapitalsTheDecoderShouldIgnore() {
        assertEquals("im", WordCase.decoderInput("Im"))
        assertEquals("teh", WordCase.decoderInput("TEH"))
        assertEquals("iPhone", WordCase.decoderInput("iPhone"))
        assertEquals("teh", WordCase.decoderInput("teh"))
    }

    @Test
    fun mostlyCapsMeansMoreThanOneCapital() {
        assertFalse(WordCase.isMostlyCaps("Teh"))
        assertFalse(WordCase.isMostlyCaps("iPhone"))
        assertTrue(WordCase.isMostlyCaps("THe"))
        assertTrue(WordCase.isMostlyCaps("TEH"))
    }
}
