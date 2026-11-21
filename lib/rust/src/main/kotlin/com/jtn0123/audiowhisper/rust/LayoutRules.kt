package com.jtn0123.audiowhisper.rust

/**
 * Merges the [main], [modifier] and [extension] rows of keys as keys::assembly orders them. Only
 * row sizes and the modifier keys' [code]s cross; the keys stay in Kotlin.
 */
fun <T> LayoutRules.merge(main: List<List<T>>?, modifier: List<List<T>>?, extension: List<List<T>>?, code: (T) -> Int): List<List<T>> {
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

/** The size of each of [rows], or a lone -1 for no rows. */
fun <T> LayoutRules.sizes(rows: List<List<T>>?): IntArray = rows?.map { it.size }?.toIntArray() ?: intArrayOf(-1)
