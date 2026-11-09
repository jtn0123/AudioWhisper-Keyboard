package com.jtn0123.audiowhisper.rust

/**
 * The text keyboard's geometry's entry points in libawnative: lib/rust/crates/awnative/src/keys.rs,
 * which documents each and how rects are packed into arrays. [KeyGeometry] wraps them.
 */
internal object KeysNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun flex(mode: Int, code: Int, numeric: Boolean): FloatArray

    @JvmStatic external fun desiredKey(
        mode: Int,
        keyboardWidth: Float,
        keyboardHeight: Float,
        rowCount: Int,
        rowBaseHeight: Float,
        keyMarginH: Float,
        keyMarginV: Float,
    ): FloatArray

    @JvmStatic external fun layout(
        rowSizes: IntArray,
        flex: FloatArray,
        desired: FloatArray,
        keyboardWidth: Float,
        keyboardHeight: Float,
        extendDown: Boolean,
    ): FloatArray?

    @JvmStatic external fun keyAt(rowSizes: IntArray, touchBounds: FloatArray, x: Float, y: Float): Int
}
