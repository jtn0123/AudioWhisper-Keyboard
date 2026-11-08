package com.jtn0123.audiowhisper.rust

/**
 * Swipe typing's rules, in Rust (lib/rust/crates/suggest's glide_ranker.rs and swipe_words.rs):
 * which words a swipe can spell, how strongly the glide classifier favours each, and how the words
 * before a swipe re-rank its guesses. The keyboard's `GlideRanker` and `SwipeWords` call these and
 * document the rules; they ask the dictionary and the learned words, and pass what those say in.
 */
object SwipeRules {
    /** What the words before a guess say for it; see [rankGlide]. */
    class Context(val pairCount: Int, val ngram: Int, val probability: Int)

    /** The most [glideBonus] gives. */
    val maxGlideBonus: Float = SwipeNative.maxGlideBonus()

    val maxProbability: Int = SwipeNative.maxProbability()

    /** Words this common or more count the same. */
    val common: Int = SwipeNative.common()

    /** What each step of probability below [common] costs a word, in nats. */
    val costPerStep: Double = SwipeNative.costPerStep()

    /** The probability every learned word counts as. */
    val learnedProbability: Int = SwipeNative.learnedProbability()

    /**
     * How much the words before a guess favour it, in nats, given how many times the user typed it
     * after them ([pairCount]) and the dictionary's [ngram] for it there and [probability] alone.
     */
    fun glideBonus(pairCount: Int, ngram: Int, probability: Int): Float =
        SwipeNative.glideBonus(pairCount, ngram, probability)

    /** The guesses with these [costs] and [contexts], best first, as their indexes. */
    fun rankGlide(costs: FloatArray, contexts: List<Context>): IntArray = SwipeNative.rankGlide(
        costs,
        IntArray(contexts.size) { contexts[it].pairCount },
        IntArray(contexts.size) { contexts[it].ngram },
        IntArray(contexts.size) { contexts[it].probability },
    )

    fun isSwipeable(word: String): Boolean = SwipeNative.isSwipeable(word)

    /**
     * The [words] a swipe can spell, in order. They cross to Rust as one string and their lengths:
     * a dictionary's 160,000 words one by one took longer in JNI calls than the Kotlin it replaced.
     */
    fun swipeable(words: List<String>): List<String> {
        val lengths = IntArray(words.size) { words[it].length }
        val text = buildString(lengths.sum()) { words.forEach(::append) }
        return SwipeNative.swipeable(text, lengths).map { words[it] }
    }

    /** The glide classifier's frequency for a word of [probability] (0-255). */
    fun frequencyOf(probability: Int): Double = SwipeNative.frequencyOf(probability)

    /** The frequency of a word the dictionary gives [probability] and the user's words [userProbability]. */
    fun frequency(probability: Int, userProbability: Int): Double = SwipeNative.frequency(probability, userProbability)

    /**
     * The user's words with their probabilities, from their [learned] spellings and their personal
     * dictionary's [personalWords] at [personalFrequencies]. A HashMap filled in the order the
     * Kotlin it replaced filled one, so it lists its words in the same order.
     */
    fun userWords(learned: List<String>, personalWords: List<String>, personalFrequencies: IntArray): Map<String, Int> {
        val packed = SwipeNative.userWords(learned.toTypedArray(), personalWords.toTypedArray(), personalFrequencies)
        val probabilities = HashMap<String, Int>()
        for (i in packed.indices step 2) {
            val place = packed[i]
            val word = if (place < learned.size) learned[place] else personalWords[place - learned.size]
            probabilities[word] = packed[i + 1]
        }
        return probabilities
    }
}
