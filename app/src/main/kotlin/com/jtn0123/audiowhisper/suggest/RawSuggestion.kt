package com.jtn0123.audiowhisper.suggest

/** Native NOT_A_PROBABILITY: the dictionary has no such word (or no such n-gram). */
const val NOT_A_PROBABILITY = -1

/**
 * One result of the native decoder: the word as the dictionary spells it, its score, and the
 * AOSP kind plus flag bits (`Dictionary::KIND_*` in the native `dictionary.h`).
 */
data class RawSuggestion(val word: String, val score: Int, val kindAndFlags: Int) {
    val kind: Int get() = kindAndFlags and KIND_MASK

    /** A dictionary-curated replacement such as "im" -> "I'm"; AOSP always auto-corrects these. */
    val isWhitelist: Boolean get() = kind == KIND_WHITELIST

    /** A next-word prediction (decoder called with no typed input). */
    val isPrediction: Boolean get() = kind == KIND_PREDICTION

    /** A shortcut expansion; AOSP never auto-corrects to one. */
    val isShortcut: Boolean get() = kind == KIND_SHORTCUT

    /** The decoder judged this word a plausible auto-correction target. */
    val isAppropriateForAutoCorrection: Boolean
        get() = kindAndFlags and FLAG_APPROPRIATE_FOR_AUTO_CORRECTION != 0

    companion object {
        const val KIND_MASK = 0xFF
        const val KIND_CORRECTION = 1
        const val KIND_WHITELIST = 3
        const val KIND_SHORTCUT = 7
        const val KIND_PREDICTION = 8
        const val FLAG_APPROPRIATE_FOR_AUTO_CORRECTION = 0x10000000
        const val FLAG_EXACT_MATCH = 0x40000000
    }
}

/** Read-only word statistics from the dictionary. */
interface WordLookup {
    /** Unigram probability on the dictionary's 0-255 log scale, or [NOT_A_PROBABILITY]. */
    fun probability(word: String): Int

    /**
     * Probability of [word] directly after [previous], or [NOT_A_PROBABILITY] when the dictionary
     * has no such n-gram or there is no previous word.
     */
    fun ngramProbability(previous: PreviousWords, word: String): Int
}

/** The native typing decoder, behind an interface so the policy can be tested without the .so. */
interface TypingDecoder : WordLookup {
    /**
     * Corrections and completions for [input] (or next-word predictions when it is empty), sorted
     * by score, highest first.
     */
    fun decode(input: String, previous: PreviousWords, blockOffensiveWords: Boolean): List<RawSuggestion>
}
