package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * Swipe typing's words, packed for the scorer: word `i` is [chars] from `starts[i]` to
 * `starts[i + 1]`, in UTF-16 as Kotlin stores it.
 */
class GlideWords(words: List<String>) {
    val size: Int = words.size
    internal val chars: CharArray
    internal val starts = IntArray(size + 1)

    init {
        var total = 0
        for (word in words) total += word.length
        chars = CharArray(total)
        var at = 0
        for ((i, word) in words.withIndex()) {
            starts[i] = at
            word.toCharArray(chars, at)
            at += word.length
        }
        starts[size] = at
    }

    /** Every character the words use, once each, ascending. */
    fun distinctChars(): CharArray = GlideNative.distinctChars(chars)
}

/**
 * What a character means on the layout being indexed. A word's path visits [pathKey] for each of
 * its characters that has one; a word is filed under the [endKey] of its first and last
 * characters, and left out if either has none. A letter repeats the one before (its path loops)
 * when their [lower] cases match.
 */
class GlideChar(val lower: Char, val pathKey: Int, val endKey: Int) {
    companion object {
        /** A [GlideChar.pathKey] or [GlideChar.endKey] for a character with no key. */
        const val NO_KEY = -1
    }
}

/**
 * A layout's keys as drawn, in layout order: each key's code, and its visible bounds as four
 * values in [bounds]: centre x, centre y, width and height.
 */
class GlideKeys(val codes: IntArray, val bounds: FloatArray) {
    init {
        require(bounds.size == codes.size * 4) { "four bounds per key" }
    }
}

/** A swipe's likeliest words, most likely first, by number in the [GlideWords], with weights. */
class GlideScores(val words: IntArray, val weights: FloatArray)

/**
 * Swipe typing's words indexed for one layout by the Rust scorer, lib/rust/crates/glide, a port of
 * FlorisBoard's statistical glide classifier. Read-only, so any number of threads can query it.
 * Its native memory is freed once nothing can reach it.
 */
class GlideIndex private constructor(private val handle: Long) {
    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** Opens a query for the swipe through [xs] and [ys], drawn on [keys]. Close it when done. */
    fun query(xs: FloatArray, ys: FloatArray, keys: GlideKeys): GlideQuery {
        val query = GlideNative.query(handle, xs, ys, keys.codes, keys.bounds)
        return GlideQuery(this, query)
    }

    /**
     * As [query], with when each point was drawn: [times] in milliseconds, one per point, or none.
     * [GlideQuery.scoreTuned] weighs the points the finger slowed at.
     */
    fun query(xs: FloatArray, ys: FloatArray, times: FloatArray, keys: GlideKeys): GlideQuery {
        val query = GlideNative.queryTimed(handle, xs, ys, times, keys.codes, keys.bounds)
        return GlideQuery(this, query)
    }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = GlideNative.free(handle)
    }

    companion object {
        /** Indexes [words], asking [describe] about each character they use. */
        fun build(words: GlideWords, describe: (Char) -> GlideChar): GlideIndex {
            val chars = words.distinctChars()
            val table = IntArray(chars.size * 4)
            for ((i, c) in chars.withIndex()) {
                val glideChar = describe(c)
                table[i * 4] = c.code
                table[i * 4 + 1] = glideChar.lower.code
                table[i * 4 + 2] = glideChar.pathKey
                table[i * 4 + 3] = glideChar.endKey
            }
            return GlideIndex(GlideNative.build(words.chars, words.starts, table))
        }
    }
}

/**
 * One swipe against a [GlideIndex]: [candidates] gives the words worth scoring, and [score] ranks
 * them given their frequencies, which the keyboard looks up in between.
 */
class GlideQuery internal constructor(
    /** Held so the index outlives the query. */
    private val index: GlideIndex,
    private var handle: Long,
) : AutoCloseable {
    private val cleanable = RustLibrary.cleaner.register(this, Free(handle))

    /** The words worth scoring, by number, in the order to score them. */
    @Synchronized
    fun candidates(): IntArray = GlideNative.candidates(handle).also { Reference.reachabilityFence(index) }

    /**
     * The [max] likeliest of [candidates], each at its frequency in [frequencies] (0 to 1, the
     * same length), with weights: the lower, the likelier.
     */
    @Synchronized
    fun score(candidates: IntArray, frequencies: FloatArray, max: Int): GlideScores {
        val words = IntArray(max)
        val weights = FloatArray(max)
        val count = GlideNative.score(handle, candidates, frequencies, words, weights)
        Reference.reachabilityFence(index)
        return GlideScores(words.copyOf(count), weights.copyOf(count))
    }

    /**
     * As [score], as the keyboard scores: FlorisBoard's classifier, which [score] keeps exactly,
     * plus what measured better (lib/rust/crates/glide/src/tuning.rs): how near the swipe's ends
     * are to the word's, how far it strays from the word's path, and where the finger slowed.
     */
    @Synchronized
    fun scoreTuned(candidates: IntArray, frequencies: FloatArray, max: Int): GlideScores {
        val words = IntArray(max)
        val weights = FloatArray(max)
        val count = GlideNative.scoreTuned(handle, candidates, frequencies, words, weights)
        Reference.reachabilityFence(index)
        return GlideScores(words.copyOf(count), weights.copyOf(count))
    }

    /** Frees the query; using it afterwards throws. */
    @Synchronized
    override fun close() {
        handle = 0
        cleanable.clean()
    }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = GlideNative.freeQuery(handle)
    }
}
