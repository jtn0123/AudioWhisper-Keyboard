package com.jtn0123.audiowhisper.rust

/**
 * Swipe typing's entry points in libawnative: lib/rust/crates/awnative/src/suggest/swipe.rs, which
 * documents each. [SwipeRules] wraps them.
 */
internal object SwipeNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun maxGlideBonus(): Float

    @JvmStatic external fun glideBonus(pairCount: Int, ngram: Int, probability: Int): Float

    @JvmStatic external fun rankGlide(costs: FloatArray, pairCounts: IntArray, ngrams: IntArray, probabilities: IntArray): IntArray

    @JvmStatic external fun maxProbability(): Int

    @JvmStatic external fun common(): Int

    @JvmStatic external fun costPerStep(): Double

    @JvmStatic external fun learnedProbability(): Int

    @JvmStatic external fun isSwipeable(word: String): Boolean

    @JvmStatic external fun swipeable(text: String, lengths: IntArray): IntArray

    @JvmStatic external fun frequencyOf(probability: Int): Double

    @JvmStatic external fun frequency(probability: Int, userProbability: Int): Double

    @JvmStatic external fun userWords(learned: Array<String>, personalWords: Array<String>, personalFrequencies: IntArray): IntArray
}
