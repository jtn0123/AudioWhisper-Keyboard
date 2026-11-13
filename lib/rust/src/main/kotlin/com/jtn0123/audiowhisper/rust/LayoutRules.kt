package com.jtn0123.audiowhisper.rust

/** Layout rules in keys::assembly. Kotlin retains decoded keys as opaque objects. */
object LayoutRules {
    init { RustLibrary.load() }

    @JvmStatic external fun plan(mode: Int, numberRow: Boolean): IntArray
    @JvmStatic external fun modifierType(mainType: Int): Int
    @JvmStatic private external fun mergeRows(main: IntArray, modifier: IntArray, extension: IntArray, modCodes: IntArray): IntArray
    @JvmStatic external fun hints(mode: Int, numberHints: Boolean, main: IntArray, symbols: IntArray, types: IntArray): IntArray
    @JvmStatic external fun cacheKey(mode: Int, subtypeHash: Int): Long
    @JvmStatic external fun invalidated(keys: LongArray, mode: Int, subtypeHash: Int, filter: Int): LongArray

    fun <T> merge(main: List<List<T>>?, modifier: List<List<T>>?, extension: List<List<T>>?, code: (T) -> Int): List<List<T>> {
        val keys = main.orEmpty().flatten() + modifier.orEmpty().flatten() + extension.orEmpty().flatten()
        val packed = mergeRows(sizes(main), sizes(modifier), sizes(extension), modifier.orEmpty().flatten().map(code).toIntArray())
        var at = 0
        return buildList {
            while (at < packed.size) {
                val size = packed[at++]
                add(List(size) { keys[packed[at++]] })
            }
        }
    }

    fun <T> sizes(rows: List<List<T>>?): IntArray = rows?.map { it.size }?.toIntArray() ?: intArrayOf(-1)
}
