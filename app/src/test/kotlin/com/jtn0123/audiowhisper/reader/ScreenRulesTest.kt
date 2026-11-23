package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ScreenWalkRules
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.AfterTurn
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.Move
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.NONE
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ScreenRulesTest {
    private val screen = Box(0, 100, 1080, 1500)

    @Test
    fun aCarouselOrPagerGoesSideways() {
        assertTrue(ScreenWalkRules.isSideways(1, 5, upDown = true, across = true, "RecyclerView"))
        assertTrue(ScreenWalkRules.isSideways(0, 0, upDown = false, across = true, "RecyclerView"))
        assertTrue(ScreenWalkRules.isSideways(0, 0, upDown = false, across = false, "android.widget.HorizontalScrollView"))
        assertTrue(ScreenWalkRules.isSideways(0, 0, upDown = false, across = false, "androidx.viewpager2.widget.ViewPager2"))
    }

    @Test
    fun aListGoingUpAndDownDoesNot() {
        assertFalse(ScreenWalkRules.isSideways(0, 0, upDown = true, across = false, "RecyclerView"))
        assertFalse(ScreenWalkRules.isSideways(5, 1, upDown = false, across = false, "ListView"))
        assertFalse(ScreenWalkRules.isSideways(1, 1, upDown = false, across = false, "ScrollView"))
        // One that can go up and down isn't sideways for its name, nor for also going across.
        assertFalse(ScreenWalkRules.isSideways(0, 0, upDown = true, across = true, "ViewPager"))
    }

    @Test
    fun boxesOverlapAsRectIntersectSays() {
        assertEquals(Box(0, 100, 500, 400), Box(-20, 50, 500, 400).partIn(screen))
        // Touching edges don't overlap. A box with no height inside does, as Android has it, with
        // no area.
        assertEquals(screen, Box(0, 1500, 1080, 1600).partIn(screen))
        assertEquals(Box(0, 300, 1080, 300), Box(0, 300, 1080, 300).partIn(screen))
        assertEquals(0L, Box(0, 300, 1080, 300).areaIn(screen))
    }

    @Test
    fun theAreaInSightIsWhatShows() {
        assertEquals(1080L * 300, Box(0, 1200, 1080, 2400).areaIn(screen))
        assertEquals(0L, Box(0, 1600, 1080, 2400).areaIn(screen))
        assertEquals(Box(0, 1200, 1080, 1500), Box(0, 1200, 1080, 2400).partIn(screen))
        assertEquals(screen, Box(0, 1600, 1080, 2400).partIn(screen))
    }

    @Test
    fun theListHoldsTheMostText() {
        // A scroll view (0) wrapping the list (1) that holds the text: the list, though smaller.
        assertEquals(1, ScreenWalkRules.mainScroller(intArrayOf(1, 1, 1, 0, NONE), longArrayOf(1_000, 800)))
        // Out of sight, it isn't the list however much it holds.
        assertEquals(0, ScreenWalkRules.mainScroller(intArrayOf(1, 1, 0), longArrayOf(1_000, 0)))
        // Holding as much, the bigger; as big too, the first.
        assertEquals(1, ScreenWalkRules.mainScroller(intArrayOf(0, 1), longArrayOf(500, 800)))
        assertEquals(0, ScreenWalkRules.mainScroller(intArrayOf(0, 1), longArrayOf(800, 800)))
        assertEquals(NONE, ScreenWalkRules.mainScroller(intArrayOf(NONE), longArrayOf()))
    }

    @Test
    fun textInsideTheListIsInIt() {
        val parents = intArrayOf(NONE, 0, 1, NONE)
        val pieces = intArrayOf(2, 1, 0, 3, NONE)
        assertContentEquals(booleanArrayOf(true, true, false, false, false), ScreenWalkRules.inList(pieces, parents, 1))
        assertContentEquals(BooleanArray(pieces.size), ScreenWalkRules.inList(pieces, parents, NONE))
    }

    @Test
    fun aWebPagesPieceIsBroughtIntoView() {
        assertEquals(Move.STAY, ScreenWalkRules.move(200, 300, 100, 1500, canTurn = true))
        assertEquals(Move.SHOW_ON_SCREEN, ScreenWalkRules.move(50, 150, 100, 1500, canTurn = true))
        assertEquals(Move.SHOW_ON_SCREEN, ScreenWalkRules.move(1400, 1600, 100, 1500, canTurn = false))
        assertEquals(Move.TURN_PAGES, ScreenWalkRules.move(1400, 1600, 100, 1500, canTurn = true))
    }

    @Test
    fun turningStopsOnceItFits() {
        assertEquals(AfterTurn.SHOW_ON_SCREEN, ScreenWalkRules.afterTurn(50, 150, 100, 1500))
        assertEquals(AfterTurn.DONE, ScreenWalkRules.afterTurn(1200, 1400, 100, 1500))
        assertEquals(AfterTurn.TURN_AGAIN, ScreenWalkRules.afterTurn(1400, 1600, 100, 1500))
    }

    @Test
    fun anAppsViewWellInsideIsLeftAlone() {
        assertTrue(ScreenWalkRules.isInside(200, 300, 100, 1500))
        assertFalse(ScreenWalkRules.isInside(100, 300, 100, 1500))
        assertFalse(ScreenWalkRules.isInside(1400, 1500, 100, 1500))
        assertFalse(ScreenWalkRules.isInside(300, 300, 100, 1500))
    }
}
