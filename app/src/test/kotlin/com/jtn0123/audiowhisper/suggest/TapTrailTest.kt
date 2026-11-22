package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class TapTrailTest {
    private val trail = TapTrail()
    private val left = TapOffset(-0.4f, 0f)
    private val right = TapOffset(0.45f, 0.1f)

    private fun tap(letter: Char, offset: TapOffset?) {
        trail.pressed(letter.code, offset)
        trail.positions.typed(letter.toString())
    }

    private fun tapAll(word: String, offset: TapOffset = left) = word.forEach { tap(it, offset) }

    @Test
    fun givesEachLetterOfTheWordWhereItWasTapped() {
        tap('h', left)
        tap('i', right)
        assertEquals(listOf(left, right), trail.offsetsFor("hi"))
        assertEquals(listOf(left, right), trail.offsetsFor("HI"))
        assertEquals(listOf(right), trail.offsetsFor("i"))
    }

    @Test
    fun aShiftedKeyStillMatchesItsLetter() {
        tap('H', left)
        tap('i', right)
        assertEquals(listOf(left, right), trail.offsetsFor("Hi"))
    }

    @Test
    fun aWordTheTrailDoesNotEndWithGetsNoPositions() {
        tapAll("hi")
        assertEquals(emptyList(), trail.offsetsFor("ho"))
        assertEquals(emptyList(), trail.offsetsFor("ohi"))
        assertEquals(emptyList(), trail.offsetsFor(""))
    }

    @Test
    fun aCharacterNotFromThePressHasNoPosition() {
        tap('h', left)
        trail.pressed('e'.code, right)
        trail.positions.typed("é")
        assertEquals(listOf(left, null), trail.offsetsFor("hé"))
        // A press counts for one character only.
        trail.positions.typed("y")
        assertEquals(listOf(left, null, null), trail.offsetsFor("héy"))
    }

    @Test
    fun noPositionsAtAllIsTheSameAsNone() {
        trail.positions.typed("hi")
        assertEquals(emptyList(), trail.offsetsFor("hi"))
    }

    @Test
    fun deletingALetterTakesItsPositionToo() {
        tapAll("hel")
        tap('p', right)
        trail.positions.deleted()
        tap('o', left)
        assertEquals(List(4) { left }, trail.offsetsFor("helo"))
        assertEquals(emptyList(), trail.offsetsFor("help"))
    }

    @Test
    fun theNextWordLinesUpAfterPunctuation() {
        tapAll("ok")
        tap(',', null)
        tap('s', right)
        tap('o', right)
        assertEquals(listOf(right, right), trail.offsetsFor("so"))
    }

    @Test
    fun keepsOnlyTheMostRecentCharacters() {
        val short = TapTrail(capacity = 3)
        "hello".forEach { short.pressed(it.code, right); short.positions.typed(it.toString()) }
        assertEquals(List(3) { right }, short.offsetsFor("llo"))
        assertEquals(emptyList(), short.offsetsFor("ello"))
    }

    @Test
    fun clearForgetsEverything() {
        tapAll("hi")
        trail.pressed('a'.code, right)
        trail.positions.clear()
        assertEquals(emptyList(), trail.offsetsFor("hi"))
        trail.positions.typed("a")
        assertEquals(emptyList(), trail.offsetsFor("a"))
    }

    @Test
    fun offsetsAreInKeySizesAndLimited() {
        assertEquals(TapOffset(0.25f, -0.5f), TapOffset.of(125f, 25f, 100f, 50f, 100f, 50f))
        assertEquals(TapOffset(TapOffset.LIMIT, -TapOffset.LIMIT), TapOffset.of(300f, -300f, 100f, 50f, 100f, 50f))
        assertNull(TapOffset.of(1f, 1f, 0f, 0f, 0f, 10f))
    }
}
