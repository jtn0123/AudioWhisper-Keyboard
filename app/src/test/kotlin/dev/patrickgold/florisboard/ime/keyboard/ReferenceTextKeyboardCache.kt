package dev.patrickgold.florisboard.ime.keyboard

/** TextKeyboardCache's original EnumMap<SparseArrayCompat<T>> policy, with JVM maps.
 * Subtype.hashCode is the key, deliberately retaining collisions (not subtype equality). */
internal class ReferenceTextKeyboardCache<T> {
    private val cache = KeyboardMode.entries.associateWith { mutableMapOf<Int, T>() }
    fun get(mode: KeyboardMode, hash: Int) = cache.getValue(mode)[hash]
    fun set(mode: KeyboardMode, hash: Int, value: T) { cache.getValue(mode)[hash] = value }
    fun clear() { for (mode in KeyboardMode.entries) cache.getValue(mode).clear() }
    fun clear(mode: KeyboardMode) { cache.getValue(mode).clear() }
    fun clear(hash: Int) { for (mode in KeyboardMode.entries) cache.getValue(mode).remove(hash) }
    fun clear(mode: KeyboardMode, hash: Int) { cache.getValue(mode).remove(hash) }
}
