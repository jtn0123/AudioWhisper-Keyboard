package com.jtn0123.audiowhisper.gif

/**
 * Android's `android.util.LruCache`, as main's GifImages used it, which a unit test can't run:
 * the parts it called (get, put, evictAll and sizeOf), ported from AOSP's source with the same
 * order of eviction. [ReferenceGifSearch] holds previews in it.
 */
open class ReferenceLruCache<K : Any, V : Any>(private val maxSize: Int) {
    private val map = LinkedHashMap<K, V>(0, 0.75f, true)
    private var size = 0

    @Synchronized
    fun get(key: K): V? = map[key]

    @Synchronized
    fun put(key: K, value: V): V? {
        size += sizeOf(key, value)
        val previous = map.put(key, value)
        if (previous != null) size -= sizeOf(key, previous)
        trimToSize(maxSize)
        return previous
    }

    @Synchronized
    fun evictAll() = trimToSize(-1)

    private fun trimToSize(maxSize: Int) {
        while (size > maxSize && map.isNotEmpty()) {
            val eldest = map.entries.first()
            map.remove(eldest.key)
            size -= sizeOf(eldest.key, eldest.value)
        }
    }

    protected open fun sizeOf(key: K, value: V): Int = 1
}
