package com.jtn0123.audiowhisper.gif

import java.util.Locale

/**
 * The GIFs each recent search returned, so tapping back to a category, or ✕ back to trending,
 * doesn't spend another of GIPHY's 100 calls an hour. Entries last [ttlMs]: trending changes,
 * and a search may have new GIFs. In memory only, and at most [capacity] searches.
 */
internal class GifResultCache(
    private val clock: () -> Long,
    private val ttlMs: Long = TTL_MS,
    private val capacity: Int = CAPACITY,
) {
    private class Entry(val gifs: List<Gif>, val at: Long)

    /** Least recently used first, so the eldest goes when full. */
    private val entries = LinkedHashMap<String, Entry>(capacity, 0.75f, true)

    /** What [search] returned within [ttlMs], or null. "Cats" and "cats " are the same search. */
    fun get(search: String): List<Gif>? {
        val key = key(search)
        val entry = entries[key] ?: return null
        if (clock() - entry.at >= ttlMs) {
            entries.remove(key)
            return null
        }
        return entry.gifs
    }

    fun put(search: String, gifs: List<Gif>) {
        entries[key(search)] = Entry(gifs, clock())
        while (entries.size > capacity) entries.remove(entries.keys.first())
    }

    private fun key(search: String) = search.trim().lowercase(Locale.ROOT)

    companion object {
        const val TTL_MS = 10 * 60 * 1000L
        const val CAPACITY = 20
    }
}
