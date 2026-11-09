package com.jtn0123.audiowhisper.reader

import kotlin.math.max
import kotlin.math.min

/** A rectangle on the screen, in pixels, as Android's `Rect` has it. */
internal data class Box(val left: Int, val top: Int, val right: Int, val bottom: Int)

/**
 * The decisions a walk of the screen and following along make, apart from Android's types: which
 * of the screen's lists is the one that scrolls, how much of a list is in sight, whether a list
 * goes sideways, and how to keep the words being read in view.
 */
internal object ScreenRules {
    /** No list. */
    const val NONE = -1

    /**
     * Whether a scrollable view goes sideways (a carousel or a pager), so it is never the list: a
     * collection of one row and several columns, a view that only scrolls sideways, or a horizontal
     * scroll view or pager that can't go up and down. [rows] and [columns] are 0 for a view that
     * isn't a collection.
     */
    fun isSideways(rows: Int, columns: Int, upDown: Boolean, across: Boolean, className: String): Boolean {
        if (rows == 1 && columns > 1) return true
        if (!upDown && across) return true
        return !upDown && (className.endsWith("HorizontalScrollView") || className.contains("ViewPager"))
    }

    /** Where [a] and [b] overlap, as `Rect.intersect` finds it; null if they don't. */
    fun intersect(a: Box, b: Box): Box? =
        if (a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom) {
            Box(max(a.left, b.left), max(a.top, b.top), min(a.right, b.right), min(a.bottom, b.bottom))
        } else {
            null
        }

    /** How much of [bounds] shows in [visible], in square pixels. */
    fun visibleArea(bounds: Box, visible: Box): Long =
        intersect(bounds, visible)?.let { (it.right - it.left).toLong() * (it.bottom - it.top) } ?: 0L

    /** The part of [bounds] that shows in [visible], or all of [visible] if none does. */
    fun visiblePart(bounds: Box, visible: Box): Box = intersect(bounds, visible) ?: visible

    /**
     * The list, among scrollers with these visible [areas], given which scroller each piece of
     * text is directly in ([pieceScrollers], [NONE] for none): the one in sight that holds the most
     * text itself, the bigger on a tie, the first of equals. Not simply the biggest — a settings
     * page wraps its list in a scroll view whose only travel is folding the title away.
     */
    fun mainScroller(pieceScrollers: IntArray, areas: LongArray): Int {
        val holds = IntArray(areas.size)
        for (scroller in pieceScrollers) if (scroller != NONE) holds[scroller]++
        return areas.indices
            .filter { areas[it] > 0 }
            .maxWithOrNull(compareBy<Int>({ holds[it] }, { areas[it] })) ?: NONE
    }

    /** Whether [scroller], whose parents are [parents], is [main] or inside it. */
    fun isWithin(parents: IntArray, scroller: Int, main: Int): Boolean {
        if (main == NONE) return false
        var at = scroller
        while (at != NONE) {
            if (at == main) return true
            at = parents[at]
        }
        return false
    }

    /** How to bring a web page's piece spanning [first] to [last] into a view from [top] to [bottom]. */
    enum class Move { STAY, SHOW_ON_SCREEN, TURN_PAGES }

    /**
     * It stays if it fits; it is shown on screen if it is above the view or there is no list to
     * turn ([canTurn]), which "show on screen" handles; otherwise the page is turned.
     */
    fun move(first: Int, last: Int, top: Int, bottom: Int, canTurn: Boolean): Move = when {
        FollowAlong.fits(first..last, top, bottom) -> Move.STAY
        first < top || !canTurn -> Move.SHOW_ON_SCREEN
        else -> Move.TURN_PAGES
    }

    /** What to do once the page has turned. */
    enum class AfterTurn { SHOW_ON_SCREEN, DONE, TURN_AGAIN }

    /** Turned past it, it is shown on screen; once it fits, turning stops; else the page turns again. */
    fun afterTurn(first: Int, last: Int, top: Int, bottom: Int): AfterTurn = when {
        first < top -> AfterTurn.SHOW_ON_SCREEN
        FollowAlong.fits(first..last, top, bottom) -> AfterTurn.DONE
        else -> AfterTurn.TURN_AGAIN
    }

    /** Whether an app's own view from [top] to [bottom] is well inside a view from [viewTop] to [viewBottom]. */
    fun isInside(top: Int, bottom: Int, viewTop: Int, viewBottom: Int): Boolean =
        bottom - top > 0 && top > viewTop && bottom < viewBottom
}
