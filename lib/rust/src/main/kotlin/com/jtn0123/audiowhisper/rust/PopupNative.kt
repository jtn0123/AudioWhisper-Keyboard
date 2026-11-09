package com.jtn0123.audiowhisper.rust

/**
 * A key's long-press popup's entry points in libawnative: lib/rust/crates/awnative/src/popup.rs,
 * which documents each and how results are packed. [PopupGrid] and [PopupOrder] unpack them.
 */
internal object PopupNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun extend(
        count: Int,
        prioritized: Int,
        keyLeft: Float,
        keyTop: Float,
        keyRight: Float,
        keyBottom: Float,
        baseLeft: Float,
        baseTop: Float,
        baseRight: Float,
        baseBottom: Float,
        keyboardWidth: Float,
    ): IntArray

    @JvmStatic external fun pointer(
        keyLeft: Float,
        keyTop: Float,
        keyRight: Float,
        keyBottom: Float,
        baseLeft: Float,
        baseTop: Float,
        baseRight: Float,
        baseBottom: Float,
        anchorLeft: Boolean,
        anchorOffset: Int,
        row0Count: Int,
        row1Count: Int,
        x: Float,
        y: Float,
    ): Long

    @JvmStatic external fun elementAt(sizes: IntArray, index: Int): Long

    @JvmStatic external fun order(
        main: Boolean,
        relevant: Int,
        symbolHint: Boolean,
        numberHint: Boolean,
        symbolPopups: Int,
        numberPopups: Int,
        symbolMode: Int,
        numberMode: Int,
        mergeHintPopups: Boolean,
    ): IntArray

    @JvmStatic external fun merge(computed: BooleanArray, mainComputed: Boolean, useMain: Boolean, hasMain: Boolean): IntArray
}
