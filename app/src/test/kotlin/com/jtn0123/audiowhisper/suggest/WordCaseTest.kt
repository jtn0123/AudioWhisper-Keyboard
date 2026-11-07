package com.jtn0123.audiowhisper.suggest

import java.util.Locale
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class WordCaseTest {
    private val us = Locale.US

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
        assertEquals("the", WordCase.AS_TYPED.apply("the", us))
        assertEquals("NASA", WordCase.AS_TYPED.apply("NASA", us))
        assertEquals("The", WordCase.CAPITALIZED.apply("the", us))
        assertEquals("I'm", WordCase.CAPITALIZED.apply("I'm", us))
        assertEquals("IPhone", WordCase.CAPITALIZED.apply("iPhone", us))
        assertEquals("THE", WordCase.ALL_CAPS.apply("the", us))
        assertEquals("", WordCase.CAPITALIZED.apply("", us))
    }

    @Test
    fun decoderInputDropsOnlyCapitalsTheDecoderShouldIgnore() {
        assertEquals("im", WordCase.decoderInput("Im", us))
        assertEquals("teh", WordCase.decoderInput("TEH", us))
        assertEquals("iPhone", WordCase.decoderInput("iPhone", us))
        assertEquals("teh", WordCase.decoderInput("teh", us))
    }

    @Test
    fun mostlyCapsMeansMoreThanOneCapital() {
        assertFalse(WordCase.isMostlyCaps("Teh"))
        assertFalse(WordCase.isMostlyCaps("iPhone"))
        assertTrue(WordCase.isMostlyCaps("THe"))
        assertTrue(WordCase.isMostlyCaps("TEH"))
    }
}
