package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * One emoji of a file [EmojiTable.parse] read: its value, its name (its set's first name, for a
 * variant), its keywords, and its skin tone and hair style as ordinals of the keyboard's
 * `EmojiSkinTone` and `EmojiHairStyle` (see [EmojiRules.tones]).
 */
class EmojiEntry(val value: String, val name: String, val keywords: List<String>, val skinTone: Int, val hairStyle: Int)

/** A set of emoji, a base and its variants: [size] entries from [start], in category [category]. */
class EmojiEntrySet(val category: Int, val start: Int, val size: Int)

/** An emoji's skin tone and hair style, as ordinals of `EmojiSkinTone` and `EmojiHairStyle`. */
data class EmojiTones(val skinTone: Int, val hairStyle: Int)

/**
 * An emoji file (one of the app's assets in ime/media/emoji) parsed in Rust (lib/rust/crates/emoji), which
 * suggestions then search without its text crossing again. Read-only, so any number of threads
 * may use it; its native memory is freed once nothing can reach it.
 */
class EmojiTable private constructor(private val handle: Long, text: String) {
    /** Every emoji, in the file's order. */
    val entries: List<EmojiEntry>

    /** Every set, in the file's order. */
    val sets: List<EmojiEntrySet>

    /** For each skin tone, by ordinal, the entries to show with it chosen. */
    val bySkinTone: List<IntArray>

    init {
        RustLibrary.cleaner.register(this, Free(handle))
        val layout = EmojiNative.layout(handle)
        val (emojis, keywords) = layout[0] as IntArray to layout[1] as IntArray
        val (setInts, tones) = layout[2] as IntArray to layout[3] as IntArray
        var keyword = 0
        entries = List(emojis.size / PER_EMOJI) { i ->
            val at = i * PER_EMOJI
            val words = List(emojis[at + 6]) {
                text.substring(keywords[keyword], keywords[keyword + 1]).also { keyword += 2 }
            }
            EmojiEntry(
                text.substring(emojis[at], emojis[at + 1]),
                text.substring(emojis[at + 2], emojis[at + 3]),
                words,
                emojis[at + 4],
                emojis[at + 5],
            )
        }
        sets = List(setInts.size / PER_SET) { i ->
            EmojiEntrySet(setInts[i * PER_SET], setInts[i * PER_SET + 1], setInts[i * PER_SET + 2])
        }
        var at = 1 + tones[0]
        bySkinTone = List(tones[0]) { tone -> tones.copyOfRange(at, at + tones[1 + tone]).also { at += tones[1 + tone] } }
    }

    /**
     * The emoji for [query] among those shown for [skinTone], best first: positions in
     * [bySkinTone]'s list for it, at most [max]. A name holding the query scores the share of it
     * the query is, times 0.7, and a keyword holding it 0.3 (both ignoring case as Kotlin's
     * `contains` does); ties keep the list's order, and emoji scoring 0 are left out. Throws
     * [IllegalArgumentException] for a negative [max], as Java's `Stream.limit` does.
     */
    fun suggest(query: String, skinTone: Int, max: Int): IntArray =
        EmojiNative.suggest(handle, query, skinTone, max).also { Reference.reachabilityFence(this) }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = EmojiNative.free(handle)
    }

    companion object {
        /** The ints [EmojiNative.layout] gives for each emoji and each set. */
        private const val PER_EMOJI = 7
        private const val PER_SET = 3

        /** Parses an emoji file's [text], with [categoryIds] those of `EmojiCategory`, in its order. */
        fun parse(text: String, categoryIds: List<String>): EmojiTable {
            val lengths = IntArray(categoryIds.size) { categoryIds[it].length }
            return EmojiTable(EmojiNative.parse(text, categoryIds.joinToString(""), lengths), text)
        }
    }
}

/** Emoji rules that need no parsed file. */
object EmojiRules {
    /** What [resolveAsset] gives when no file fits. */
    const val NO_ASSET = -1

    /**
     * The query in the [composing] text: what follows [prefix] (`:`, or nothing), if the text
     * starts with it, is at least [minLength] plus the prefix long, and is only ASCII letters.
     * Null otherwise.
     */
    fun validateQuery(composing: String, prefix: String, minLength: Int): String? =
        EmojiNative.validateQuery(composing, prefix, minLength)

    /**
     * The skin tone and hair style of the emoji [value]: the first of each enum's entries whose
     * code point it holds, or `DEFAULT`.
     */
    fun tones(value: String): EmojiTones {
        val tones = EmojiNative.tones(value)
        return EmojiTones(tones and 0xFF, tones shr 8)
    }

    /**
     * Which of the emoji files [assets] a locale reads: `{language}_{country}_{variant}.txt`,
     * then `{language}_{country}.txt`, then `{language}.txt` (the language in lower case, and a
     * blank country or variant left out), as its position, or [NO_ASSET].
     */
    fun resolveAsset(assets: List<String>, language: String, country: String, variant: String): Int {
        val lengths = IntArray(assets.size) { assets[it].length }
        return EmojiNative.resolveAsset(assets.joinToString(""), lengths, arrayOf(language, country, variant))
    }
}
