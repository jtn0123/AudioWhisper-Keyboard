package com.jtn0123.audiowhisper.rust

/**
 * The swipe scorer's entry points in libawnative: lib/rust/crates/awnative/src/glide.rs, which
 * documents each. Handles are owned by [GlideIndex] and [GlideQuery], which free them.
 */
internal object GlideNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun distinctChars(chars: CharArray): CharArray

    @JvmStatic external fun build(chars: CharArray, starts: IntArray, table: IntArray): Long

    @JvmStatic external fun free(index: Long)

    @JvmStatic external fun query(index: Long, xs: FloatArray, ys: FloatArray, keyCodes: IntArray, keyBounds: FloatArray): Long

    @JvmStatic external fun freeQuery(query: Long)

    @JvmStatic external fun candidates(query: Long): IntArray

    @JvmStatic external fun score(
        query: Long,
        candidates: IntArray,
        frequencies: FloatArray,
        outWords: IntArray,
        outWeights: FloatArray,
    ): Int
}
