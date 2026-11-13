package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SwipeRules

/**
 * The words swipe typing can produce and how strongly the glide classifier favours each: the
 * dictionary's words, plus the user's personal and learned words so their names and slang can
 * be swiped too. Immutable; a change makes a new instance ([withUserWords]), which is how the
 * classifier knows to re-index.
 *
 * Frequencies come from the dictionary itself, not a copy: [probability] looks a word up, so the
 * 160,000 words cost a list and no map. The rules (which words a swipe can spell, and how a 0-255
 * probability becomes the classifier's frequency) are in Rust ([SwipeRules],
 * lib/rust/crates/suggest/src/swipe_words.rs).
 */
class ReferenceNlpSwipeWords private constructor(
    private val dictionaryWords: List<String>,
    private val probability: (String) -> Int,
    private val userWords: Map<String, Int>,
) {
    /** Every word: the dictionary's first, then the user's that it lacks. */
    val words: List<String> = when {
        userWords.isEmpty() -> dictionaryWords
        else -> dictionaryWords + userWords.keys.filter { probability(it) == NOT_A_PROBABILITY }
    }

    /** How strongly the classifier favours [word] (see [frequencyOf]); 0 for a word it doesn't know. */
    fun frequency(word: String): Double = SwipeRules.frequency(probability(word), userWords[word] ?: NOT_A_PROBABILITY)

    /** These dictionary words with [userWords] (see [userWords]); this same instance if they are unchanged. */
    fun withUserWords(userWords: Map<String, Int>): ReferenceNlpSwipeWords =
        if (userWords == this.userWords) this else ReferenceNlpSwipeWords(dictionaryWords, probability, userWords)

    companion object {
        val NONE = ReferenceNlpSwipeWords(emptyList(), { NOT_A_PROBABILITY }, emptyMap())

        /** Words this common or more count the same; see [frequencyOf]. */
        val COMMON: Int = SwipeRules.common

        /** The probability every word the user has typed [TypingHistory.KNOWN_AFTER] times counts as. */
        val LEARNED_PROBABILITY: Int = SwipeRules.learnedProbability

        /**
         * The dictionary's [words], each found again by [probability]. Words a swipe can't spell on
         * a letters layout ("MP3", "US$") are left out.
         */
        fun fromDictionary(words: List<String>, probability: (String) -> Int) =
            ReferenceNlpSwipeWords(SwipeRules.swipeable(words), probability, emptyMap())

        /**
         * The user's words as probabilities: [personal] entries at their own frequency, [learned]
         * spellings at [LEARNED_PROBABILITY]; the higher one where a word is in both.
         */
        fun userWords(personal: List<PersonalWords.Entry>, learned: Collection<String>): Map<String, Int> =
            SwipeRules.userWords(learned.toList(), personal.map { it.word }, IntArray(personal.size) { personal[it].frequency })

        /**
         * The classifier's frequency for a word of [probability] (0-255): 1 from [COMMON] up, then
         * less for each step below it, down to 0.30 for the rarest word. Common words count the same
         * because words sharing a path differ only by frequency, and context then settles them (see
         * [GlideRanker]).
         */
        fun frequencyOf(probability: Int): Double = SwipeRules.frequencyOf(probability)

        /** Letters, with apostrophes or hyphens inside, which a swipe skips over. */
        fun isSwipeable(word: String): Boolean = SwipeRules.isSwipeable(word)
    }
}
