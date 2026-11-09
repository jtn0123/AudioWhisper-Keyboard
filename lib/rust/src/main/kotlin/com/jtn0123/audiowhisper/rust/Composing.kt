package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * FlorisBoard's text composers, in Rust (lib/rust/crates/composing): each takes the text before
 * the cursor and the character about to be typed, and answers how many characters before the
 * cursor to delete and what to type instead, `0 to toInsert` when it has nothing to do.
 */
object Composing {
    /** Korean jamo joined into syllables, and split again when a vowel takes a final. */
    fun hangul(precedingText: String, toInsert: String): Pair<Int, String> =
        unpack(ComposingNative.hangul(precedingText, toInsert), toInsert)

    /**
     * A voicing mark or the small-kana key (〓) applied to the kana before it, or taken off it;
     * [sticky] keeps a mark on instead.
     */
    fun kana(precedingText: String, toInsert: String, sticky: Boolean): Pair<Int, String> =
        unpack(ComposingNative.kana(precedingText, toInsert, sticky), toInsert)

    /** An action as composing.rs packs it: null for none, else the count's two halves, then the text. */
    internal fun unpack(packed: String?, toInsert: String): Pair<Int, String> {
        if (packed == null) return 0 to toInsert
        return ((packed[0].code shl 16) or packed[1].code) to packed.substring(2)
    }
}

/**
 * A table of rules, each an ending and what it becomes (such as Telex for Vietnamese), held in
 * Rust and tried in the order given: the first whose ending the text, with the keystroke, ends
 * with when lower-cased replaces it, upper-cased if the ending began upper-case. Safe to share
 * between threads, as it never changes. Its native memory is freed once nothing can reach it.
 */
class ComposingRules(rules: List<Pair<String, String>>) {
    private val handle: Long

    init {
        val keys = rules.map { it.first }
        val values = rules.map { it.second }
        handle = ComposingNative.newRules(
            keys.joinToString(""),
            IntArray(keys.size) { keys[it].length },
            values.joinToString(""),
            IntArray(values.size) { values[it].length },
        )
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** What typing [toInsert] after [precedingText] does. */
    fun actions(precedingText: String, toInsert: String): Pair<Int, String> {
        val packed = ComposingNative.rules(handle, precedingText, toInsert)
        Reference.reachabilityFence(this)
        return Composing.unpack(packed, toInsert)
    }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = ComposingNative.freeRules(handle)
    }
}
