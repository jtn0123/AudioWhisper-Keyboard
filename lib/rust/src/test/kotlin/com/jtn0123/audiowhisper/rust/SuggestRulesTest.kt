package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * The suggestion rules through JNI, as the keyboard calls them: results unpacked whole, text
 * passed through unit for unit, bad input thrown as Kotlin threw it, and a tap trail's native
 * memory. The rules themselves are tested in Rust and, against the Kotlin they replace, in the app.
 */
class SuggestRulesTest {
    @Test
    fun casesRoundTrip() {
        assertEquals(SuggestRules.Case.AS_TYPED, SuggestRules.case("iPhone"))
        assertEquals(SuggestRules.Case.CAPITALIZED, SuggestRules.case("Teh"))
        assertEquals(SuggestRules.Case.ALL_CAPS, SuggestRules.case("TEH"))
        assertEquals("Straße", SuggestRules.applyCase(SuggestRules.Case.CAPITALIZED, "straße"))
        assertEquals("STRASSE", SuggestRules.applyCase(SuggestRules.Case.ALL_CAPS, "straße"))
        assertEquals("teh", SuggestRules.decoderInput("TEH"))
    }

    @Test
    fun oddUtf16PassesThroughUnchanged() {
        val odd = "a\uD800b😀\u0000﻿"
        assertEquals(odd, SuggestRules.applyCase(SuggestRules.Case.AS_TYPED, odd))
        assertEquals(odd, SuggestRules.decoderInput(odd))
        assertEquals(listOf("${odd}z"), SuggestRules.previousWords("${odd}z ").words)
    }

    @Test
    fun previousWordsUnpackTheSentenceStart() {
        val started = SuggestRules.previousWords("Hi. you are ")
        assertEquals(listOf("are", "you"), started.words)
        assertTrue(started.isBeginningOfSentence)
        val unknown = SuggestRules.previousWords("well, I ")
        assertEquals(listOf("I"), unknown.words)
        assertFalse(unknown.isBeginningOfSentence)
        // The sentence start crosses JNI as a first string of "."; it is not a word.
        val afterAStop = SuggestRules.previousWords(". ")
        assertEquals(emptyList(), afterAStop.words)
        assertTrue(afterAStop.isBeginningOfSentence)
    }

    @Test
    fun anUndoLongerThanAUnitHoldsUnpacksWhole() {
        // The length crosses JNI as two UTF-16 units; one past 65,535 needs both.
        val corrected = "x".repeat(70_000)
        val revert = SuggestRules.revertAt("typed", corrected, 0, corrected.length + 1, "$corrected ")
        assertEquals(70_001, revert?.length)
        assertEquals("typed", revert?.text)
        assertEquals(true, revert?.phantomSpace)
        val kept = SuggestRules.revertAt("teh", "the", 0, 4, "the,")
        assertEquals(4, kept?.length)
        assertEquals("teh,", kept?.text)
        assertEquals(false, kept?.phantomSpace)
        assertNull(SuggestRules.revertAt("teh", "the", 0, 4, "tha "))
    }

    @Test
    fun badInputThrowsAsKotlinDid() {
        val notACodePoint = assertFailsWith<IllegalArgumentException> {
            SuggestRules.editDistance(intArrayOf(-1), intArrayOf('a'.code))
        }
        assertEquals("Not a valid Unicode code point: 0xFFFFFFFF", notACodePoint.message)
        val nan = assertFailsWith<IllegalArgumentException> {
            QwertyLayout.touchPoint('q'.code, true, Float.NaN, 0f)
        }
        assertEquals("Cannot round NaN value.", nan.message)
        assertFailsWith<IllegalArgumentException> { TapPositions(-1) }
    }

    @Test
    fun theLayoutUnpacks() {
        assertEquals(26 * 3, QwertyLayout.keys().size)
        assertEquals(QwertyLayout.gridWidth * QwertyLayout.gridHeight * QwertyLayout.maxProximityChars, QwertyLayout.proximityGrid().size)
        assertContentEquals(intArrayOf(50, 75), QwertyLayout.keyCenter('Q'.code))
        assertNull(QwertyLayout.keyCenter('1'.code))
        assertContentEquals(intArrayOf(50, 75), QwertyLayout.touchPoint('q'.code, false, 0.4f, 0.4f))
        assertContentEquals(intArrayOf(90, 135), QwertyLayout.touchPoint('q'.code, true, 0.4f, 0.4f))
        assertContentEquals(floatArrayOf(0.25f, -0.6f), SuggestRules.tapOffset(125f, -100f, 100f, 50f, 100f, 50f))
        assertNull(SuggestRules.tapOffset(1f, 1f, 0f, 0f, 0f, 10f))
    }

    @Test
    fun aTrailPacksEachCharactersTap() {
        val trail = TapPositions()
        trail.pressed('t'.code, true, 0.1f, -0.2f)
        trail.typed("t")
        trail.pressed('h'.code, false, 0f, 0f)
        trail.typed("h")
        trail.pressed('e'.code, true, -0.3f, 0.4f)
        trail.typed("e")
        assertContentEquals(floatArrayOf(1f, 0.1f, -0.2f, 0f, 0f, 0f, 1f, -0.3f, 0.4f), trail.offsetsFor("THE"))
        assertContentEquals(floatArrayOf(), trail.offsetsFor("she"))
        trail.deleted()
        assertContentEquals(floatArrayOf(1f, 0.1f, -0.2f, 0f, 0f, 0f), trail.offsetsFor("th"))
        trail.clear()
        assertContentEquals(floatArrayOf(), trail.offsetsFor("th"))
    }

    @Test
    fun manyTrailsAreFreed() {
        // Each trail's native memory goes once it is unreachable; making many must not run out.
        repeat(100_000) { TapPositions(4).typed("abc") }
        System.gc()
    }

    @Test
    fun theRulesNumbersComeFromRust() {
        assertEquals(0.185f, SuggestRules.modestThreshold)
        assertEquals(3, SuggestRules.maxPreviousWords)
        assertEquals(0.6f, SuggestRules.tapLimit)
        assertEquals(64, SuggestRules.tapTrailCapacity)
        assertEquals(listOf(100, 150, 1000, 450, 32, 16, 16), with(QwertyLayout) {
            listOf(keyWidth, keyHeight, keyboardWidth, keyboardHeight, gridWidth, gridHeight, maxProximityChars)
        })
        assertTrue(SuggestRules.continuesWord('@'.code))
        assertFalse(SuggestRules.continuesWord(-1))
        assertTrue(SuggestRules.isMostlyCaps("THe"))
        assertTrue(SuggestRules.exceedsThreshold("I'm", 0, 3, "im", 0.185f))
        assertEquals(0.5f, SuggestRules.normalizedScore("ab", "ac", 1_000_000))
    }
}
