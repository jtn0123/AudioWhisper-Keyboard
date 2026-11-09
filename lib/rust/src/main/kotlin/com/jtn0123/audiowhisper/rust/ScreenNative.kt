package com.jtn0123.audiowhisper.rust

/**
 * The screen walk's and following along's entry points in libawnative:
 * lib/rust/crates/awnative/src/screen.rs, which documents each. [ScreenWalkRules] wraps them.
 */
internal object ScreenNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun isSideways(rows: Int, columns: Int, upDown: Boolean, across: Boolean, className: String): Boolean

    @JvmStatic external fun visibleArea(boxes: IntArray): Long

    @JvmStatic external fun visiblePart(boxes: IntArray): IntArray

    @JvmStatic external fun mainScroller(pieceScrollers: IntArray, areas: LongArray): Int

    @JvmStatic external fun inList(pieceScrollers: IntArray, parents: IntArray, main: Int): BooleanArray

    @JvmStatic external fun moveIntoView(first: Int, last: Int, top: Int, bottom: Int, canTurn: Boolean): Int

    @JvmStatic external fun afterTurn(first: Int, last: Int, top: Int, bottom: Int): Int

    @JvmStatic external fun isInside(top: Int, bottom: Int, viewTop: Int, viewBottom: Int): Boolean
}
