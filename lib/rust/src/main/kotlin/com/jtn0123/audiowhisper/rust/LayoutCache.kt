package com.jtn0123.audiowhisper.rust

/** Rust owns key identity and invalidation; managed keyboard objects remain on the JVM. */
class LayoutCache<T> {
    private val objects = mutableMapOf<Long, T>()
    fun get(mode: Int, subtypeHash: Int): T? = objects[LayoutRules.cacheKey(mode, subtypeHash)]
    fun set(mode: Int, subtypeHash: Int, value: T) { objects[LayoutRules.cacheKey(mode, subtypeHash)] = value }
    fun clear() = objects.clear()
    fun clearMode(mode: Int) = remove(mode, 0, 1)
    fun clearSubtype(subtypeHash: Int) = remove(0, subtypeHash, 2)
    fun clear(mode: Int, subtypeHash: Int) = remove(mode, subtypeHash, 3)
    private fun remove(mode: Int, subtypeHash: Int, filter: Int) {
        LayoutRules.invalidated(objects.keys.toLongArray(), mode, subtypeHash, filter).forEach(objects::remove)
    }
}
