package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.rust.ScreenWalkRules.AfterTurn
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.Move
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.NONE
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * The screen walk's rules through JNI, as the keyboard calls them: boxes as one array, scrollers
 * as indexes with -1 for none, choices as their enums, and bad input thrown. The rules themselves
 * are tested in Rust and, against the Kotlin they replace, in the app.
 */
class ScreenWalkRulesTest {
    private val screen = intArrayOf(0, 100, 1080, 1500)

    @Test
    fun aCarouselGoesSideways() {
        assertTrue(ScreenWalkRules.isSideways(1, 5, upDown = true, across = true, "RecyclerView"))
        assertTrue(ScreenWalkRules.isSideways(0, 0, upDown = false, across = false, "androidx.viewpager2.widget.ViewPager2"))
        assertFalse(ScreenWalkRules.isSideways(0, 0, upDown = true, across = true, "ViewPager"))
        assertFalse(ScreenWalkRules.isSideways(0, 0, upDown = false, across = false, ""))
    }

    @Test
    fun theAreaInSightIsWhatShows() {
        assertEquals(1080L * 300, ScreenWalkRules.visibleArea(intArrayOf(0, 1200, 1080, 2400) + screen))
        assertEquals(0L, ScreenWalkRules.visibleArea(intArrayOf(0, 1600, 1080, 2400) + screen))
        // Kotlin's Int sides wrap, then the area is a Long.
        val wide = intArrayOf(Int.MIN_VALUE, 0, Int.MAX_VALUE, 2, Int.MIN_VALUE, 0, Int.MAX_VALUE, 2)
        assertEquals(-2L, ScreenWalkRules.visibleArea(wide))
        assertContentEquals(intArrayOf(0, 1200, 1080, 1500), ScreenWalkRules.visiblePart(intArrayOf(0, 1200, 1080, 2400) + screen))
        assertContentEquals(screen, ScreenWalkRules.visiblePart(intArrayOf(0, 1600, 1080, 2400) + screen))
        assertFailsWith<IllegalArgumentException> { ScreenWalkRules.visibleArea(screen) }
        assertFailsWith<IllegalArgumentException> { ScreenWalkRules.visiblePart(screen + screen + 1) }
    }

    @Test
    fun theListHoldsTheMostText() {
        assertEquals(1, ScreenWalkRules.mainScroller(intArrayOf(1, 1, 1, 0, NONE), longArrayOf(1_000, 800)))
        assertEquals(0, ScreenWalkRules.mainScroller(intArrayOf(0, 1), longArrayOf(800, 800)))
        assertEquals(NONE, ScreenWalkRules.mainScroller(intArrayOf(NONE), longArrayOf()))
        assertFailsWith<IllegalArgumentException> { ScreenWalkRules.mainScroller(intArrayOf(2), longArrayOf(1, 1)) }
        assertFailsWith<IllegalArgumentException> { ScreenWalkRules.mainScroller(intArrayOf(-2), longArrayOf(1)) }
    }

    @Test
    fun textInsideTheListIsInIt() {
        val parents = intArrayOf(NONE, 0, 1, NONE)
        val pieces = intArrayOf(2, 1, 0, 3, NONE)
        assertContentEquals(booleanArrayOf(true, true, false, false, false), ScreenWalkRules.inList(pieces, parents, 1))
        assertContentEquals(BooleanArray(5), ScreenWalkRules.inList(pieces, parents, NONE))
        // A parent after its child could loop, so it is refused, as is a scroller that isn't one.
        assertFailsWith<IllegalArgumentException> { ScreenWalkRules.inList(pieces, intArrayOf(NONE, 2, 1, NONE), 1) }
        assertFailsWith<IllegalArgumentException> { ScreenWalkRules.inList(intArrayOf(4), parents, 1) }
        assertFailsWith<IllegalArgumentException> { ScreenWalkRules.inList(pieces, parents, 4) }
    }

    @Test
    fun aWebPagesPieceIsBroughtIntoView() {
        assertEquals(Move.STAY, ScreenWalkRules.move(200, 300, 100, 1500, canTurn = true))
        assertEquals(Move.SHOW_ON_SCREEN, ScreenWalkRules.move(50, 150, 100, 1500, canTurn = true))
        assertEquals(Move.SHOW_ON_SCREEN, ScreenWalkRules.move(1400, 1600, 100, 1500, canTurn = false))
        assertEquals(Move.TURN_PAGES, ScreenWalkRules.move(1400, 1600, 100, 1500, canTurn = true))
        assertEquals(AfterTurn.SHOW_ON_SCREEN, ScreenWalkRules.afterTurn(50, 150, 100, 1500))
        assertEquals(AfterTurn.DONE, ScreenWalkRules.afterTurn(1200, 1400, 100, 1500))
        assertEquals(AfterTurn.TURN_AGAIN, ScreenWalkRules.afterTurn(1400, 1600, 100, 1500))
    }

    @Test
    fun anAppsViewWellInsideIsLeftAlone() {
        assertTrue(ScreenWalkRules.isInside(200, 300, 100, 1500))
        assertFalse(ScreenWalkRules.isInside(100, 300, 100, 1500))
        assertFalse(ScreenWalkRules.isInside(300, 300, 100, 1500))
        // The height wraps as Kotlin's Int does.
        assertFalse(ScreenWalkRules.isInside(Int.MIN_VALUE + 5, Int.MAX_VALUE - 5, Int.MIN_VALUE, Int.MAX_VALUE))
    }
}
