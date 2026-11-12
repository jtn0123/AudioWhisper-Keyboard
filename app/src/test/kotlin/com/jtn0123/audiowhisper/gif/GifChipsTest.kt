package com.jtn0123.audiowhisper.gif

import kotlin.test.Test
import kotlin.test.assertEquals

class GifChipsTest {
    private val categories = GifSearch.categories

    @Test
    fun recentsComeBeforeTheCategoriesOnceAGifWasSent() {
        assertEquals(categories, GifSearch.chips(typedWord = null, hasRecents = false))
        assertEquals(listOf(GifSearch.RECENTS) + categories, GifSearch.chips(typedWord = null, hasRecents = true))
    }

    @Test
    fun theWordJustTypedComesFirst() {
        assertEquals(listOf("pizza", GifSearch.RECENTS) + categories, GifSearch.chips("pizza", hasRecents = true))
    }

    @Test
    fun aTypedWordThatIsAlreadyAChipMovesThatChipFirst() {
        val chips = GifSearch.chips("congrats", hasRecents = false)
        assertEquals(listOf("Congrats") + (categories - "Congrats"), chips)
        assertEquals(categories.size, chips.size)
    }

    @Test
    fun aPrivateFieldOffersNoTypedWord() {
        assertEquals("congrats", GifSearch.typedWordFor("well done, congrats", private = false))
        assertEquals(null, GifSearch.typedWordFor("hunter2", private = true))
    }
}
