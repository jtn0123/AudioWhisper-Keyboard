package com.jtn0123.audiowhisper.rust

/**
 * The text composers' entry points in libawnative: lib/rust/crates/awnative/src/composing.rs,
 * which documents each and how an action is packed into a string. [Composing] and
 * [ComposingRules] unpack them.
 */
internal object ComposingNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun hangul(precedingText: String, toInsert: String): String?

    @JvmStatic external fun kana(precedingText: String, toInsert: String, sticky: Boolean): String?

    @JvmStatic external fun newRules(keys: String, keyLengths: IntArray, values: String, valueLengths: IntArray): Long

    @JvmStatic external fun freeRules(rules: Long)

    @JvmStatic external fun rules(rules: Long, precedingText: String, toInsert: String): String?
}
