package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ScreenWalkRules
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.AfterTurn
import com.jtn0123.audiowhisper.rust.ScreenWalkRules.Move

/** A rectangle on the screen, in pixels, as Android's `Rect` has it. */
internal data class Box(val left: Int, val top: Int, val right: Int, val bottom: Int)

/**
 * The decisions a walk of the screen and following along make, apart from Android's types, in
 * Rust (lib/rust/crates/reader/src/screen.rs): which of the screen's lists is the one that
 * scrolls and what is in it, how much of a list is in sight, whether a list goes sideways, and how
 * to keep the words being read in view.
 */
internal object ScreenRules {
    /** No list. */
    const val NONE = ScreenWalkRules.NONE

    /**
     * Whether a scrollable view goes sideways (a carousel or a pager), so it is never the list: a
     * collection of one row and several columns, a view that only scrolls sideways, or a horizontal
     * scroll view or pager that can't go up and down. [rows] and [columns] are 0 for a view that
     * isn't a collection.
     */
    fun isSideways(rows: Int, columns: Int, upDown: Boolean, across: Boolean, className: String): Boolean =
        ScreenWalkRules.isSideways(rows, columns, upDown, across, className)

    /** How much of [bounds] shows in [visible], in square pixels. */
    fun visibleArea(bounds: Box, visible: Box): Long = ScreenWalkRules.visibleArea(sides(bounds, visible))

    /** The part of [bounds] that shows in [visible], or all of [visible] if none does. */
    fun visiblePart(bounds: Box, visible: Box): Box {
        val (left, top, right, bottom) = ScreenWalkRules.visiblePart(sides(bounds, visible))
        return Box(left, top, right, bottom)
    }

    /**
     * The list, among scrollers with these visible [areas], given which scroller each piece of
     * text is directly in ([pieceScrollers], [NONE] for none): the one in sight that holds the most
     * text itself, the bigger on a tie, the first of equals. Not simply the biggest — a settings
     * page wraps its list in a scroll view whose only travel is folding the title away.
     */
    fun mainScroller(pieceScrollers: IntArray, areas: LongArray): Int = ScreenWalkRules.mainScroller(pieceScrollers, areas)

    /**
     * For each piece of text, whether the scroller it is directly in ([pieceScrollers]) is [main] or
     * inside it, given each scroller's [parents], which come before it.
     */
    fun inList(pieceScrollers: IntArray, parents: IntArray, main: Int): BooleanArray =
        ScreenWalkRules.inList(pieceScrollers, parents, main)

    /**
     * How to bring a web page's piece spanning [first] to [last] into a view from [top] to
     * [bottom]: it stays if it fits; it is shown on screen if it is above the view or there is no
     * list to turn ([canTurn]), which "show on screen" handles; otherwise the page is turned.
     */
    fun move(first: Int, last: Int, top: Int, bottom: Int, canTurn: Boolean): Move =
        ScreenWalkRules.move(first, last, top, bottom, canTurn)

    /** Turned past it, it is shown on screen; once it fits, turning stops; else the page turns again. */
    fun afterTurn(first: Int, last: Int, top: Int, bottom: Int): AfterTurn = ScreenWalkRules.afterTurn(first, last, top, bottom)

    /** Whether an app's own view from [top] to [bottom] is well inside a view from [viewTop] to [viewBottom]. */
    fun isInside(top: Int, bottom: Int, viewTop: Int, viewBottom: Int): Boolean =
        ScreenWalkRules.isInside(top, bottom, viewTop, viewBottom)

    private fun sides(bounds: Box, visible: Box) = intArrayOf(
        bounds.left, bounds.top, bounds.right, bounds.bottom, visible.left, visible.top, visible.right, visible.bottom,
    )
}
