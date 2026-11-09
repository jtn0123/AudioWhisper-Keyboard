package com.jtn0123.audiowhisper.rust

/**
 * The GIF panel's entry points in libawnative: lib/rust/crates/awnative/src/gif.rs, which
 * documents each. [GifRules] and [GifResultKeys] wrap them.
 */
internal object GifNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun maxLength(): Int

    @JvmStatic external fun categories(): Array<String>

    @JvmStatic external fun recents(): String

    @JvmStatic external fun cacheTtlMs(): Long

    @JvmStatic external fun cacheCapacity(): Int

    @JvmStatic external fun recentsCapacity(): Int

    @JvmStatic external fun type(query: String, text: String): String

    @JvmStatic external fun typeWord(query: String, word: String): String

    @JvmStatic external fun deleteLast(query: String, count: Int): String

    @JvmStatic external fun deleteWords(query: String, count: Int): String

    @JvmStatic external fun lastWord(text: String?): String?

    @JvmStatic external fun chips(typedWord: String?, hasRecents: Boolean): Array<String>

    @JvmStatic external fun route(code: Int, typing: Boolean): Int

    @JvmStatic external fun swipeDelete(swipe: Int, unitsX: Int): Int

    @JvmStatic external fun problem(failure: Int, status: Int): String

    @JvmStatic external fun cacheKey(search: String): String

    @JvmStatic external fun newCache(ttlMs: Long, capacity: Int): Long

    @JvmStatic external fun freeCache(cache: Long)

    @JvmStatic external fun cacheGet(cache: Long, key: String, now: Long): Boolean

    @JvmStatic external fun cachePut(cache: Long, key: String, now: Long): Array<String>

    @JvmStatic external fun recentsAdd(ids: Array<String>, id: String, capacity: Int): IntArray

    @JvmStatic external fun recentsLoad(saved: Array<String>, sent: Array<String>, capacity: Int): IntArray
}
