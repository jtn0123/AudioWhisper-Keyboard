package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRules

/** Editing the search field, in Rust (lib/rust/crates/gif), kept free of Android so it can be unit tested. */
internal object SearchText {
    /** The longest query, in UTF-16 units: GIPHY's limit. */
    val MAX_LENGTH: Int = GifRules.maxLength

    /** [query] with [text] typed after it, cut at [MAX_LENGTH]. */
    fun type(query: String, text: String): String = GifRules.type(query, text)

    /** A glide-typed [word], spaced from the one before. */
    fun typeWord(query: String, word: String): String = GifRules.typeWord(query, word)

    /** [query] without its last character, an emoji counting as one. */
    fun deleteLast(query: String): String = deleteLast(query, 1)

    /** [query] without its last [count] characters. */
    fun deleteLast(query: String, count: Int): String = GifRules.deleteLast(query, count)

    /** Deletes [count] words from the end, and the spaces after them; the space before stays. */
    fun deleteWords(query: String, count: Int): String = GifRules.deleteWords(query, count)

    /**
     * The word just typed before the cursor, to offer as a GIF search: "congrats" from "Big
     * congrats! ". Null when the text doesn't end in a word of 2 to 30 letters.
     */
    fun lastWord(text: CharSequence?): String? = GifRules.lastWord(text)
}
