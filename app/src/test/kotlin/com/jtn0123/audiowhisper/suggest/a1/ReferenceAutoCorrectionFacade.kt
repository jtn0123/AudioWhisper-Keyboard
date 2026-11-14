package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules

/**
 * The AOSP auto-correction threshold check, ported from `AutoCorrectionUtils` (Java) and the
 * native `AutocorrectionThresholdUtils::calcNormalizedScore` / Damerau-Levenshtein edit distance
 * (Apache-2.0, The Android Open Source Project). The port is in Rust (lib/rust/crates/suggest).
 */
object AutoCorrection {
    /** AOSP's "modest" auto-correction threshold, the HeliBoard default. */
    val MODEST_THRESHOLD: Float = SuggestRules.modestThreshold

    /**
     * Port of `AutoCorrectionUtils.suggestionExceedsThreshold`: whitelisted words always pass,
     * words the decoder did not mark as appropriate never do, and the rest need a normalized
     * score of at least [threshold].
     */
    fun suggestionExceedsThreshold(suggestion: RawSuggestion, typed: String, threshold: Float): Boolean =
        SuggestRules.exceedsThreshold(suggestion.word, suggestion.score, suggestion.kindAndFlags, typed, threshold)

    /**
     * Port of `calcNormalizedScore`: the decoder score scaled back to a float and weighted down
     * by how much of [after] had to be edited to get there from [before].
     */
    fun normalizedScore(before: String, after: String, score: Int): Float =
        SuggestRules.normalizedScore(before, after, score)

    /**
     * Port of the native `EditDistance` with `DamerauLevenshteinEditDistancePolicy`: unit cost
     * insert/delete/substitute plus adjacent transposition, comparing base lower-case letters
     * (native `CharUtils::toBaseLowerCase`: the accent stripped below U+0500, then lower-cased).
     * A negative value, which is no code point, throws [IllegalArgumentException].
     */
    fun editDistance(before: IntArray, after: IntArray): Int = SuggestRules.editDistance(before, after)
}
