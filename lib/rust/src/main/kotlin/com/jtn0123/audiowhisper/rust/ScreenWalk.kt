package com.jtn0123.audiowhisper.rust

/**
 * The decisions a walk of the screen and following along make, in Rust
 * (lib/rust/crates/reader/src/screen.rs): whether a scrollable view goes sideways, how much of a
 * box shows, which of the screen's lists is the one that scrolls and what is inside it, and how to
 * keep the words being read in view. The keyboard's `reader.ScreenRules` wraps these.
 *
 * A scroller is its index in the walk's list of them, and [NONE] is none. Two boxes go in as one
 * array of eight: a box's left, top, right and bottom, then the visible box's.
 */
object ScreenWalkRules {
    /** No scroller. */
    const val NONE = -1

    /** How to bring a web page's piece into view. */
    enum class Move { STAY, SHOW_ON_SCREEN, TURN_PAGES }

    /** What to do once the page has turned. */
    enum class AfterTurn { SHOW_ON_SCREEN, DONE, TURN_AGAIN }

    /**
     * Whether a scrollable view goes sideways (a carousel or a pager): a collection of one row and
     * several [columns], a view that only scrolls [across], or a horizontal scroll view or pager,
     * by its [className], that can't go [upDown]. [rows] and [columns] are 0 for a view that isn't a
     * collection.
     */
    fun isSideways(rows: Int, columns: Int, upDown: Boolean, across: Boolean, className: String): Boolean =
        ScreenNative.isSideways(rows, columns, upDown, across, className)

    /** How much of the first of [boxes] shows in the second, in square pixels. */
    fun visibleArea(boxes: IntArray): Long = ScreenNative.visibleArea(boxes)

    /** The part of the first of [boxes] that shows in the second, or all of the second: its four sides. */
    fun visiblePart(boxes: IntArray): IntArray = ScreenNative.visiblePart(boxes)

    /**
     * The list, among scrollers with these visible [areas], given which scroller each piece of text
     * is directly in ([pieceScrollers]): the one in sight that holds the most text itself, the
     * bigger on a tie, the first of equals; [NONE] if none is in sight.
     */
    fun mainScroller(pieceScrollers: IntArray, areas: LongArray): Int = ScreenNative.mainScroller(pieceScrollers, areas)

    /**
     * For each piece of text, whether the scroller it is directly in ([pieceScrollers]) is [main] or
     * inside it, given each scroller's [parents]; a parent comes before its child.
     */
    fun inList(pieceScrollers: IntArray, parents: IntArray, main: Int): BooleanArray =
        ScreenNative.inList(pieceScrollers, parents, main)

    /** How to bring a web page's piece spanning [first] to [last] into a view from [top] to [bottom]. */
    fun move(first: Int, last: Int, top: Int, bottom: Int, canTurn: Boolean): Move =
        Move.entries[ScreenNative.moveIntoView(first, last, top, bottom, canTurn)]

    /** What to do once the page has turned, the piece spanning [first] to [last]. */
    fun afterTurn(first: Int, last: Int, top: Int, bottom: Int): AfterTurn =
        AfterTurn.entries[ScreenNative.afterTurn(first, last, top, bottom)]

    /** Whether an app's own view from [top] to [bottom] is well inside a view from [viewTop] to [viewBottom]. */
    fun isInside(top: Int, bottom: Int, viewTop: Int, viewBottom: Int): Boolean =
        ScreenNative.isInside(top, bottom, viewTop, viewBottom)
}
