package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifResultKeys

/**
 * The GIFs each recent search returned, so tapping back to a category, or ✕ back to trending,
 * doesn't spend another of GIPHY's 100 calls an hour. Entries last [ttlMs]: trending changes,
 * and a search may have new GIFs. In memory only, and at most [capacity] searches. Which searches
 * are kept is decided in Rust ([GifResultKeys]); the GIFs stay here.
 */
internal class GifResultCache(
    private val clock: () -> Long,
    ttlMs: Long = TTL_MS,
    capacity: Int = CAPACITY,
) {
    private val keys = GifResultKeys(ttlMs, capacity)
    private val entries = HashMap<String, List<Gif>>()

    /** What [search] returned within [ttlMs], or null. "Cats" and "cats " are the same search. */
    fun get(search: String): List<Gif>? {
        val key = GifResultKeys.key(search)
        if (keys.get(key, clock())) return entries[key]
        entries.remove(key)
        return null
    }

    /** How many searches' GIFs are held. */
    val size: Int get() = entries.size

    fun put(search: String, gifs: List<Gif>) {
        val key = GifResultKeys.key(search)
        entries[key] = gifs
        for (dropped in keys.put(key, clock())) entries.remove(dropped)
    }

    companion object {
        val TTL_MS: Long = GifResultKeys.TTL_MS
        val CAPACITY: Int = GifResultKeys.CAPACITY
    }
}
