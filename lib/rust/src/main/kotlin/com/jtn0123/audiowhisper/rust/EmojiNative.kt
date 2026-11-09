package com.jtn0123.audiowhisper.rust

/**
 * Emoji data's and suggestions' entry points in libawnative: lib/rust/crates/awnative/src/emoji.rs,
 * which documents each and how answers are packed. [EmojiTable] and [EmojiRules] wrap them.
 */
internal object EmojiNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun parse(text: String, ids: String, idLengths: IntArray): Long

    @JvmStatic external fun free(data: Long)

    @JvmStatic external fun layout(data: Long): Array<Any>

    @JvmStatic external fun validateQuery(composing: String, prefix: String, minLength: Int): String?

    @JvmStatic external fun suggest(data: Long, query: String, skinTone: Int, max: Int): IntArray

    @JvmStatic external fun tones(value: String): Int

    @JvmStatic external fun resolveAsset(names: String, lengths: IntArray, locale: Array<String>): Int
}
