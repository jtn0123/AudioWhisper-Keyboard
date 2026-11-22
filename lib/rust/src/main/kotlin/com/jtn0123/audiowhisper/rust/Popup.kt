package com.jtn0123.audiowhisper.rust

/**
 * An extended popup laid out by [PopupGrid.extend]: up to 5 keys in one row (row 0), more in two,
 * row 1 above. Places are numbered row 1 first, left to right, then row 0. Hand-written, as the
 * generator makes no class that reads a row by place: lib/rust/crates/awnative/src/popup.rs
 * documents the row.
 */
class PopupLayout internal constructor(private val packed: IntArray) {
    /** The popup is anchored at the key's left edge, else at its right. */
    val anchorLeft: Boolean get() = packed[0] == 1

    /** How many keys the popup reaches past the key, towards the keyboard's middle. */
    val anchorOffset: Int get() = packed[1]
    val row0Count: Int get() = packed[2]
    val row1Count: Int get() = packed[3]

    /** The place under the finger when the popup opens. */
    val initialIndex: Int get() = packed[4]
    val left: Float get() = Float.fromBits(packed[5])
    val top: Float get() = Float.fromBits(packed[6])
    val right: Float get() = Float.fromBits(packed[7])
    val bottom: Float get() = Float.fromBits(packed[8])

    /** How many places the popup has. */
    val count: Int get() = (packed.size - HEADER) / 2

    /** Which of the popup's keys the place at [uiIndex] shows: negative for a prioritized one. */
    fun popupIndex(uiIndex: Int): Int = packed[HEADER + uiIndex]

    /** The row the place at [uiIndex] is in. */
    fun row(uiIndex: Int): Int = packed[HEADER + count + uiIndex]

    private companion object {
        const val HEADER = 9
    }
}
