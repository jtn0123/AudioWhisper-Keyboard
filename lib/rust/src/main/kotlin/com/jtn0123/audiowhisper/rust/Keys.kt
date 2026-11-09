package com.jtn0123.audiowhisper.rust

/**
 * The text keyboard's geometry, in Rust (lib/rust/crates/keys): how wide each key wants to be, the
 * desired key a keyboard is laid out with, where each key goes, and which key a touch lands on.
 * Keyboard modes are `KeyboardMode.value`s. A rect is four Floats (left, top, right, bottom), and a
 * keyboard's keys are one flat array in row order with [rowSizes][layout] beside it.
 */
object KeyGeometry {
    /** A visible key's flex factors: how much it shrinks and grows, and how wide it wants to be. */
    class Flex(val shrink: Float, val grow: Float, val widthFactor: Float)

    /** The factors of a visible key with this [code] on a keyboard of this [mode]; [numeric] if its type is. */
    fun flex(mode: Int, code: Int, numeric: Boolean): Flex {
        val flex = KeysNative.flex(mode, code, numeric)
        return Flex(shrink = flex[0], grow = flex[1], widthFactor = flex[2])
    }

    /**
     * The desired key's touch bounds and then its visible bounds, eight Floats: a tenth of the
     * keyboard wide, a row high (at most 1.12 base rows, for character and symbol keyboards) and
     * the visible bounds inside the margins.
     */
    fun desiredKey(
        mode: Int,
        keyboardWidth: Float,
        keyboardHeight: Float,
        rowCount: Int,
        rowBaseHeight: Float,
        keyMarginH: Float,
        keyMarginV: Float,
    ): FloatArray = KeysNative.desiredKey(
        mode, keyboardWidth, keyboardHeight, rowCount, rowBaseHeight, keyMarginH, keyMarginV,
    )

    /**
     * Lays out rows of [rowSizes] keys, whose [flex] is three Floats each (width factor, shrink,
     * grow), with the [desired] key's touch and visible bounds: each key's touch and visible
     * bounds, eight Floats a key, or null if nothing is to be laid out and each key keeps its bounds.
     */
    fun layout(
        rowSizes: IntArray,
        flex: FloatArray,
        desired: FloatArray,
        keyboardWidth: Float,
        keyboardHeight: Float,
        extendDown: Boolean,
    ): FloatArray? = KeysNative.layout(rowSizes, flex, desired, keyboardWidth, keyboardHeight, extendDown)

    /**
     * The index of the first key whose [touchBounds] (four Floats a key) hold ([x], [y]), or -1.
     * The search stops at the first row without keys.
     */
    fun keyAt(rowSizes: IntArray, touchBounds: FloatArray, x: Float, y: Float): Int =
        KeysNative.keyAt(rowSizes, touchBounds, x, y)
}
