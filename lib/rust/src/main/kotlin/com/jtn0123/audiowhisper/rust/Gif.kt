package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * The GIF panel's rules, in Rust (lib/rust/crates/gif): editing the search field, the chips under
 * it, which keys reach it, the order of the GIFs sent before, and what a failed GIPHY call tells
 * the user. The keyboard's `gif` package wraps these and documents them.
 *
 * A GIF in recents' new order is the index of one in the list before, or `-1 - i` for the `i`th
 * sent ([sent] reads it back).
 */
object GifRules {
    /** The longest search, in UTF-16 units: GIPHY's limit. */
    val maxLength: Int = GifNative.maxLength()

    /** The quick searches, [trending] first. */
    val categories: List<String> = GifNative.categories().asList()

    /** GIPHY's trending list's chip. */
    val trending: String = categories.first()

    /** The chip for the GIFs sent before. */
    val recents: String = GifNative.recents()

    /** How many GIFs recents keeps. */
    val recentsCapacity: Int = GifNative.recentsCapacity()

    /** Where a key goes while the user types into the search field. */
    enum class Route { SEARCH, KEYBOARD, IGNORE }

    /** The delete key's swipes the search field acts on. */
    enum class Swipe { DELETE_CHARACTERS, DELETE_WORDS, SELECT }

    /** How a GIPHY call failed: an answer that isn't the JSON expected, an HTTP status, or none. */
    enum class Failure { UNREADABLE, STATUS, UNREACHABLE }

    fun type(query: String, text: String): String = GifNative.type(query, text)

    fun typeWord(query: String, word: String): String = GifNative.typeWord(query, word)

    fun deleteLast(query: String, count: Int): String = GifNative.deleteLast(query, count)

    fun deleteWords(query: String, count: Int): String = GifNative.deleteWords(query, count)

    fun lastWord(text: CharSequence?): String? = GifNative.lastWord(text?.toString())

    fun chips(typedWord: String?, hasRecents: Boolean): List<String> = GifNative.chips(typedWord, hasRecents).asList()

    /** Where the key with [code] goes; [typing] says whether it is a character or numeric key. */
    fun route(code: Int, typing: Boolean): Route = Route.entries[GifNative.route(code, typing)]

    /** How many characters, or words, a delete-key [swipe] [unitsX] key widths along reaches back. */
    fun swipeDelete(swipe: Swipe, unitsX: Int): Int = GifNative.swipeDelete(swipe.ordinal, unitsX)

    /** What a [failure], with this HTTP [status] if it had one, tells the user. */
    fun problem(failure: Failure, status: Int = 0): String = GifNative.problem(failure.ordinal, status)

    /** Recents' order after the GIF with [id] was sent, given the [ids] before. */
    fun recentsAdd(ids: List<String>, id: String, capacity: Int): IntArray =
        GifNative.recentsAdd(ids.toTypedArray(), id, capacity)

    /** Recents' order once the [saved] ids are read, keeping those [sent] meanwhile, newest first. */
    fun recentsLoad(saved: List<String>, sent: List<String>, capacity: Int): IntArray =
        GifNative.recentsLoad(saved.toTypedArray(), sent.toTypedArray(), capacity)

    /** Whether [place] in recents' order is a GIF sent rather than one from before. */
    fun isSent(place: Int): Boolean = place < 0

    /** The index [place] in recents' order names, among those before or those sent. */
    fun index(place: Int): Int = if (place < 0) -1 - place else place
}

/**
 * Which searches the GIF result cache keeps, in Rust: each for [ttlMs], at most [capacity], the
 * least recently used going first. The GIFs stay in Kotlin under [key]. Safe to use from any
 * thread. Its native memory is freed once nothing can reach it.
 */
class GifResultKeys(ttlMs: Long = TTL_MS, capacity: Int = CAPACITY) {
    private val handle = GifNative.newCache(ttlMs, capacity)

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** Whether [key]'s GIFs, stored before, are still kept at [now]; a stale one is dropped. */
    fun get(key: String, now: Long): Boolean = GifNative.cacheGet(handle, key, now).also { Reference.reachabilityFence(this) }

    /** [key]'s GIFs were stored at [now]: the keys dropped to make room, eldest first. */
    fun put(key: String, now: Long): List<String> =
        GifNative.cachePut(handle, key, now).asList().also { Reference.reachabilityFence(this) }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = GifNative.freeCache(handle)
    }

    companion object {
        val TTL_MS: Long = GifNative.cacheTtlMs()
        val CAPACITY: Int = GifNative.cacheCapacity()

        /** The key a search is kept under: "Cats" and "cats " are the same. */
        fun key(search: String): String = GifNative.cacheKey(search)
    }
}
