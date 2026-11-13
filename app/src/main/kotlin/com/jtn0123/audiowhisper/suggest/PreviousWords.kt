package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules

/**
 * The n-gram context before the word being typed: up to [MAX_COUNT] previous words, most recent
 * first, and whether a beginning-of-sentence marker follows them. Mirrors AOSP `NgramContext`.
 */
data class PreviousWords(val words: List<String>, val isBeginningOfSentence: Boolean) {
    val slotCount: Int get() = SuggestRules.previousSlotCount(words.size, isBeginningOfSentence)

    companion object {
        /** Native MAX_PREV_WORD_COUNT_FOR_N_GRAM. */
        val MAX_COUNT: Int = SuggestRules.maxPreviousWords

        /** No usable context (for example after a comma). */
        val NONE = PreviousWords(emptyList(), isBeginningOfSentence = false)

        /**
         * The context for a word that starts right after [textBeforeWord]. Same rules as AOSP
         * `NgramContextUtils.getNgramContextFromNthPreviousWord`: only the current line counts,
         * words are split on whitespace, the start of the line or a word ending in `.?!` is a
         * sentence start, and any other trailing punctuation leaves the context unknown. A word
         * connector (`'`, `’`, `-`) glued to the next word leaves it unknown too. The rules are in
         * Rust (lib/rust/crates/suggest).
         */
        fun from(textBeforeWord: CharSequence): PreviousWords {
            val context = SuggestRules.previousWords(textBeforeWord.toString())
            return PreviousWords(context.words, context.isBeginningOfSentence)
        }
    }
}
