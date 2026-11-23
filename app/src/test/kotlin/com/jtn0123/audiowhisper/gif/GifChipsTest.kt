package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRules
import kotlin.test.Test
import kotlin.test.assertEquals

/** The chips the panel's session offers (crates/gif), as the panel sees them. */
class GifChipsTest {
    private val rig = SessionRig()
    private val categories = GifRules.categories
    private val recents = GifRules.recents
    private val gif = Gif("a", "title", "p", 1f, "s", "u")

    private fun chips() = rig.session.chips.value

    @Test
    fun recentsComeBeforeTheCategoriesOnceAGifWasSent() {
        assertEquals(categories, chips())
        rig.session.sent(gif, incognito = false)
        assertEquals(listOf(recents) + categories, chips())
    }

    @Test
    fun aGifSentInIncognitoIsNotRemembered() {
        rig.session.sent(gif, incognito = true)
        assertEquals(categories, chips())
    }

    @Test
    fun theWordJustTypedComesFirst() {
        rig.session.sent(gif, incognito = false)
        rig.session.offerTypedWord("I want pizza", private = false)
        assertEquals(listOf("pizza", recents) + categories, chips())
    }

    @Test
    fun aTypedWordThatIsAlreadyAChipMovesThatChipFirst() {
        rig.session.offerTypedWord("congrats", private = false)
        assertEquals(listOf("Congrats") + (categories - "Congrats"), chips())
        assertEquals(categories.size, chips().size)
    }

    @Test
    fun aPrivateFieldOrNoTextOffersNoTypedWord() {
        rig.session.offerTypedWord("well done, congrats", private = true)
        assertEquals(categories, chips())
        rig.session.offerTypedWord(null, private = false)
        assertEquals(categories, chips())
    }
}
