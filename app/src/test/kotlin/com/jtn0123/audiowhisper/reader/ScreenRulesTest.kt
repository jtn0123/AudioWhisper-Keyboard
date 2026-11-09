package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.reader.ScreenRules.NONE
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.AfterTurn
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.Move
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ScreenRulesTest {
    private val screen = Box(0, 100, 1080, 1500)

    @Test
    fun aCarouselOrPagerGoesSideways() {
        assertTrue(ScreenRules.isSideways(1, 5, upDown = true, across = true, "RecyclerView"))
        assertTrue(ScreenRules.isSideways(0, 0, upDown = false, across = true, "RecyclerView"))
        assertTrue(ScreenRules.isSideways(0, 0, upDown = false, across = false, "android.widget.HorizontalScrollView"))
        assertTrue(ScreenRules.isSideways(0, 0, upDown = false, across = false, "androidx.viewpager2.widget.ViewPager2"))
    }

    @Test
    fun aListGoingUpAndDownDoesNot() {
        assertFalse(ScreenRules.isSideways(0, 0, upDown = true, across = false, "RecyclerView"))
        assertFalse(ScreenRules.isSideways(5, 1, upDown = false, across = false, "ListView"))
        assertFalse(ScreenRules.isSideways(1, 1, upDown = false, across = false, "ScrollView"))
        // One that can go up and down isn't sideways for its name, nor for also going across.
        assertFalse(ScreenRules.isSideways(0, 0, upDown = true, across = true, "ViewPager"))
    }

    @Test
    fun boxesOverlapAsRectIntersectSays() {
        assertEquals(Box(0, 100, 500, 400), ScreenRules.visiblePart(Box(-20, 50, 500, 400), screen))
        // Touching edges don't overlap. A box with no height inside does, as Android has it, with
        // no area.
        assertEquals(screen, ScreenRules.visiblePart(Box(0, 1500, 1080, 1600), screen))
        assertEquals(Box(0, 300, 1080, 300), ScreenRules.visiblePart(Box(0, 300, 1080, 300), screen))
        assertEquals(0L, ScreenRules.visibleArea(Box(0, 300, 1080, 300), screen))
    }

    @Test
    fun theAreaInSightIsWhatShows() {
        assertEquals(1080L * 300, ScreenRules.visibleArea(Box(0, 1200, 1080, 2400), screen))
        assertEquals(0L, ScreenRules.visibleArea(Box(0, 1600, 1080, 2400), screen))
        assertEquals(Box(0, 1200, 1080, 1500), ScreenRules.visiblePart(Box(0, 1200, 1080, 2400), screen))
        assertEquals(screen, ScreenRules.visiblePart(Box(0, 1600, 1080, 2400), screen))
    }

    @Test
    fun theListHoldsTheMostText() {
        // A scroll view (0) wrapping the list (1) that holds the text: the list, though smaller.
        assertEquals(1, ScreenRules.mainScroller(intArrayOf(1, 1, 1, 0, NONE), longArrayOf(1_000, 800)))
        // Out of sight, it isn't the list however much it holds.
        assertEquals(0, ScreenRules.mainScroller(intArrayOf(1, 1, 0), longArrayOf(1_000, 0)))
        // Holding as much, the bigger; as big too, the first.
        assertEquals(1, ScreenRules.mainScroller(intArrayOf(0, 1), longArrayOf(500, 800)))
        assertEquals(0, ScreenRules.mainScroller(intArrayOf(0, 1), longArrayOf(800, 800)))
        assertEquals(NONE, ScreenRules.mainScroller(intArrayOf(NONE), longArrayOf()))
    }

    @Test
    fun textInsideTheListIsInIt() {
        val parents = intArrayOf(NONE, 0, 1, NONE)
        val pieces = intArrayOf(2, 1, 0, 3, NONE)
        assertContentEquals(booleanArrayOf(true, true, false, false, false), ScreenRules.inList(pieces, parents, 1))
        assertContentEquals(BooleanArray(pieces.size), ScreenRules.inList(pieces, parents, NONE))
    }

    @Test
    fun aWebPagesPieceIsBroughtIntoView() {
        assertEquals(Move.STAY, ScreenRules.move(200, 300, 100, 1500, canTurn = true))
        assertEquals(Move.SHOW_ON_SCREEN, ScreenRules.move(50, 150, 100, 1500, canTurn = true))
        assertEquals(Move.SHOW_ON_SCREEN, ScreenRules.move(1400, 1600, 100, 1500, canTurn = false))
        assertEquals(Move.TURN_PAGES, ScreenRules.move(1400, 1600, 100, 1500, canTurn = true))
    }

    @Test
    fun turningStopsOnceItFits() {
        assertEquals(AfterTurn.SHOW_ON_SCREEN, ScreenRules.afterTurn(50, 150, 100, 1500))
        assertEquals(AfterTurn.DONE, ScreenRules.afterTurn(1200, 1400, 100, 1500))
        assertEquals(AfterTurn.TURN_AGAIN, ScreenRules.afterTurn(1400, 1600, 100, 1500))
    }

    @Test
    fun anAppsViewWellInsideIsLeftAlone() {
        assertTrue(ScreenRules.isInside(200, 300, 100, 1500))
        assertFalse(ScreenRules.isInside(100, 300, 100, 1500))
        assertFalse(ScreenRules.isInside(1400, 1500, 100, 1500))
        assertFalse(ScreenRules.isInside(300, 300, 100, 1500))
    }
}
