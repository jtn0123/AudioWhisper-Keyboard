package com.jtn0123.audiowhisper.reader

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** The view the text is read in: the page from 200 down to the keyboard at 1,400. */
class FollowAlongTest {
    private fun fits(span: IntRange) = FollowAlong.fits(span, top = 200, bottom = 1_400)

    @Test
    fun aParagraphWhollyInSightIsLeftWhereItIs() {
        assertTrue(fits(400..600))
    }

    @Test
    fun aParagraphBehindTheKeyboardIsBroughtUp() {
        assertFalse(fits(1_300..1_500))
        assertFalse(fits(1_600..1_800))
    }

    @Test
    fun aParagraphAboveTheViewIsBroughtDown() {
        assertFalse(fits(100..300))
    }

    @Test
    fun aParagraphTallerThanTheViewFitsOnceItStartsInTheTopHalf() {
        assertTrue(fits(500..2_000))
        assertFalse(fits(1_000..2_500))
    }
}
