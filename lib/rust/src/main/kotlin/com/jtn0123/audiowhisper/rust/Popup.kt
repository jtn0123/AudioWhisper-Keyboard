package com.jtn0123.audiowhisper.rust

/**
 * A rectangle in the keyboard's coordinates, as upstream's `FlorisRect` holds it, for crossing
 * into Rust without the app's classes.
 */
class PopupRect(val left: Float, val top: Float, val right: Float, val bottom: Float)

/**
 * An extended popup laid out by [PopupGrid.extend]: up to 5 keys in one row (row 0), more in two,
 * row 1 above. Places are numbered row 1 first, left to right, then row 0.
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

/** The extended popup's grid and the key under the finger, in Rust (lib/rust/crates/popup/src/grid.rs). */
object PopupGrid {
    /** What [pointer] gives once the finger is off the popup. */
    const val OFF_THE_POPUP = Long.MIN_VALUE

    /**
     * The extended popup of [count] keys, [prioritized] of them prioritized, for the key at [key]
     * whose preview popup is [base], on a keyboard [keyboardWidth] wide; null if there are no keys.
     */
    fun extend(count: Int, prioritized: Int, key: PopupRect, base: PopupRect, keyboardWidth: Float): PopupLayout? {
        val packed = PopupNative.extend(
            count, prioritized, key.left, key.top, key.right, key.bottom,
            base.left, base.top, base.right, base.bottom, keyboardWidth,
        )
        return if (packed.isEmpty()) null else PopupLayout(packed)
    }

    /**
     * Which place of a popup laid out with these [anchorLeft], [anchorOffset], [row0Count] and
     * [row1Count] the finger at ([x], [y]) is on, for the key at [key] whose preview popup is
     * [base]; [OFF_THE_POPUP] once the finger has left it. Takes the sides as floats, not
     * rectangles, as it runs on every move of the finger.
     */
    fun pointer(
        keyLeft: Float, keyTop: Float, keyRight: Float, keyBottom: Float,
        baseLeft: Float, baseTop: Float, baseRight: Float, baseBottom: Float,
        anchorLeft: Boolean, anchorOffset: Int, row0Count: Int, row1Count: Int,
        x: Float, y: Float,
    ): Long = PopupNative.pointer(
        keyLeft, keyTop, keyRight, keyBottom, baseLeft, baseTop, baseRight, baseBottom,
        anchorLeft, anchorOffset, row0Count, row1Count, x, y,
    )

    /**
     * Where the element numbered [index] is in rows of these [sizes], counting from the last row
     * back: its row in the high half of the result and its place in the row in the low half; -1
     * if there is none.
     */
    fun elementAt(sizes: IntArray, index: Int): Long = PopupNative.elementAt(sizes, index)
}

/** The order of a key's popup keys for the hint settings, in Rust (lib/rust/crates/popup/src/order.rs). */
object PopupOrder {
    /** A set's keys, numbered: these three, then its relevant keys, the symbol hint's and the number hint's popup keys. */
    const val MAIN = 0
    const val SYMBOL_HINT = 1
    const val NUMBER_HINT = 2
    const val FIRST_LISTED = 3

    /** No hint, in [Order.hint]. */
    const val NO_HINT = -1

    /** In a merge's steps: the merged set's main key becomes the set's. */
    const val SET_MAIN = -1

    /** In [Order.ownList]: the other keys are these keys of the set's, numbered, not one of its own lists. */
    const val NUMBERED = 0

    /** In [Order.ownList]: the other keys are the set's own list of relevant keys, symbol popups or number popups. */
    const val OWN_RELEVANT = -1
    const val OWN_SYMBOL_POPUPS = -2
    const val OWN_NUMBER_POPUPS = -3

    /**
     * The keys in order, numbered as above. Unless [ownList] is [NUMBERED], the other keys are
     * that list of the set's itself, which keys merged later show in, and [other] is empty.
     */
    class Order(val hint: Int, val prioritized: IntArray, val other: IntArray, val ownList: Int)

    /** How a hint is offered, as `KeyHintMode` orders them. */
    enum class Mode { DISABLED, HINT_PRIORITY, ACCENT_PRIORITY, SMART_PRIORITY }

    /**
     * The order of a set's keys: whether it has a [main] key, a [symbolHint] and a [numberHint],
     * and how many [relevant] keys and popup keys of each hint, for these settings.
     */
    fun order(
        main: Boolean, relevant: Int, symbolHint: Boolean, numberHint: Boolean,
        symbolPopups: Int, numberPopups: Int, symbolMode: Mode, numberMode: Mode, mergeHintPopups: Boolean,
    ): Order {
        val packed = PopupNative.order(
            main, relevant, symbolHint, numberHint, symbolPopups, numberPopups,
            symbolMode.ordinal, numberMode.ordinal, mergeHintPopups,
        )
        val prioritized = packed[1]
        val other = packed[2]
        return Order(
            hint = packed[0],
            prioritized = packed.copyOfRange(3, 3 + prioritized),
            other = packed.copyOfRange(3 + prioritized, 3 + prioritized + other.coerceAtLeast(0)),
            ownList = other.coerceAtMost(NUMBERED),
        )
    }

    /**
     * Where a merged set's keys go: its relevant keys that [computed] and then its main key if it
     * [mainComputed]. Each step is the index of a key to add (the number of relevant keys for its
     * main) or [SET_MAIN], which the merge may do if it [useMain] and the set [hasMain] not.
     */
    fun merge(computed: BooleanArray, mainComputed: Boolean, useMain: Boolean, hasMain: Boolean): IntArray =
        PopupNative.merge(computed, mainComputed, useMain, hasMain)
}
