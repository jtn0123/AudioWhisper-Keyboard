package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.WordStoreRules

/** The words and word pairs the user has typed, as suggestions read them. Kept by [LearnedWords]. */
interface TypingHistory {
    /** How the user spells [word] (ignoring case), once they have used it [KNOWN_AFTER] times. */
    fun spellingOf(word: String): String?

    /**
     * Up to [maxCount] words used [KNOWN_AFTER] times that start with [typed] (ignoring case) and
     * are longer, most used first, keeping only those [accept] lets through.
     */
    fun completions(typed: String, maxCount: Int, accept: (String) -> Boolean = { true }): List<String>

    /** Up to [maxCount] words typed right after [previous] at least [minCount] times, most used first. */
    fun followers(previous: PreviousWords, minCount: Int, maxCount: Int): List<String>

    /** How many times [word] was typed right after [previous]. */
    fun pairCount(previous: PreviousWords, word: String): Int

    companion object {
        /** A word typed this many times is one of the user's words. */
        val KNOWN_AFTER: Int = WordStoreRules.knownAfter

        val NONE: TypingHistory = object : TypingHistory {
            override fun spellingOf(word: String): String? = null
            override fun completions(typed: String, maxCount: Int, accept: (String) -> Boolean) = emptyList<String>()
            override fun followers(previous: PreviousWords, minCount: Int, maxCount: Int) = emptyList<String>()
            override fun pairCount(previous: PreviousWords, word: String) = 0
        }
    }
}
