package com.jtn0123.audiowhisper.suggest

import java.text.Normalizer

/**
 * The AOSP auto-correction threshold check, ported from `AutoCorrectionUtils` (Java) and the
 * native `AutocorrectionThresholdUtils::calcNormalizedScore` / Damerau-Levenshtein edit distance
 * (Apache-2.0, The Android Open Source Project).
 */
object AutoCorrection {
    /** AOSP's "modest" auto-correction threshold, the HeliBoard default. */
    const val MODEST_THRESHOLD = 0.185f

    /** Native SUGGEST_INTERFACE_OUTPUT_SCALE: decoder scores are fixed-point with this scale. */
    private const val OUTPUT_SCALE = 1_000_000f

    /** Native BASE_CHARS covers code points below this; above it characters compare as-is. */
    private const val BASE_CHARS_SIZE = 0x500

    /**
     * Port of `AutoCorrectionUtils.suggestionExceedsThreshold`: whitelisted words always pass,
     * words the decoder did not mark as appropriate never do, and the rest need a normalized
     * score of at least [threshold].
     */
    fun suggestionExceedsThreshold(suggestion: RawSuggestion, typed: String, threshold: Float): Boolean {
        if (suggestion.isWhitelist) return true
        if (!suggestion.isAppropriateForAutoCorrection) return false
        return normalizedScore(typed, suggestion.word, suggestion.score) >= threshold
    }

    /**
     * Port of `calcNormalizedScore`: the decoder score scaled back to a float and weighted down
     * by how much of [after] had to be edited to get there from [before].
     */
    fun normalizedScore(before: String, after: String, score: Int): Float {
        val beforeCodePoints = before.codePoints().toArray()
        val afterCodePoints = after.codePoints().toArray()
        if (beforeCodePoints.isEmpty() || afterCodePoints.isEmpty()) return 0f
        if (afterCodePoints.all { it == ' '.code }) return 0f
        val distance = editDistance(beforeCodePoints, afterCodePoints)
        if (score <= 0 || distance >= afterCodePoints.size) return 0f
        val weight = 1f - distance.toFloat() / afterCodePoints.size.toFloat()
        return score.toFloat() / OUTPUT_SCALE * weight
    }

    /**
     * Port of the native `EditDistance` with `DamerauLevenshteinEditDistancePolicy`: unit cost
     * insert/delete/substitute plus adjacent transposition, comparing base lower-case letters.
     */
    fun editDistance(before: IntArray, after: IntArray): Int {
        val a = IntArray(before.size) { baseLowerCase(before[it]) }
        val b = IntArray(after.size) { baseLowerCase(after[it]) }
        val width = b.size + 1
        val dp = IntArray((a.size + 1) * width)
        for (i in 0..a.size) dp[width * i] = i
        for (j in 0..b.size) dp[j] = j
        for (i in a.indices) {
            for (j in b.indices) {
                val substitution = if (a[i] == b[j]) 0 else 1
                var best = minOf(dp[width * i + j + 1] + 1, dp[width * (i + 1) + j] + 1, dp[width * i + j] + substitution)
                if (i > 0 && j > 0 && a[i] == b[j - 1] && b[j] == a[i - 1]) {
                    best = minOf(best, dp[width * (i - 1) + j - 1] + substitution)
                }
                dp[width * (i + 1) + j + 1] = best
            }
        }
        return dp[dp.size - 1]
    }

    /** Native `CharUtils::toBaseLowerCase`: strip the accent (Latin range only), then lower-case. */
    internal fun baseLowerCase(codePoint: Int): Int {
        var base = codePoint
        if (codePoint < BASE_CHARS_SIZE) {
            val decomposed = Normalizer.normalize(String(Character.toChars(codePoint)), Normalizer.Form.NFD)
            base = decomposed.codePointAt(0)
        }
        return Character.toLowerCase(base)
    }
}
