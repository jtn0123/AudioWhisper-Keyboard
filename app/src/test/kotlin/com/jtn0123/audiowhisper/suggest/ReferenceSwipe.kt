package com.jtn0123.audiowhisper.suggest

import kotlin.math.exp
import kotlin.math.ln
import kotlin.math.max
import kotlin.math.min

/**
 * Swipe typing's ranking and word list as they were in Kotlin before moving to Rust
 * (lib/rust/crates/suggest), copied unchanged from commit d09f766c: GlideRanker.kt (less
 * [GlideGuess], which is the app's) and SwipeWords.kt. They are what [SwipeRulesMatchTest] holds
 * the Rust to; don't change them. [TypingHistory], [WordLookup], [PreviousWords] and
 * [PersonalWords] are the app's.
 */
object ReferenceSwipe {
    /**
     * Re-ranks swipe guesses with the words before them, so the swipe for "to" after "me" gives
     * "too" once the user has typed "me too". Each guess's cost goes down by what the context says
     * for it:
     *  - the user's habit: how often they typed it after that word ([TypingHistory.pairCount]);
     *  - the dictionary's n-gram, which the bundled one has for at most three next words a word.
     * A missing pair says little, so neither ever makes a guess worse.
     *
     * The bonuses are small on purpose: together at most [MAX_BONUS]. The classifier's own costs are
     * close together (common words get the same frequency, see [SwipeWords.frequencyOf]): words
     * with the same path ("to", "too") differ by about 0.07, and a neighbouring key's word is
     * within 0.1-0.5 of a clean swipe ("an" even beats "am" by 0.14). So context settles those, but
     * not a swipe that clearly traces a different word ("you" over "thou": 1.4-2.2).
     */
    class GlideRanker(private val history: TypingHistory, private val lookup: WordLookup?) {
        /** [guesses], best first in [previous]'s context. Ties keep the classifier's order. */
        fun rank(guesses: List<GlideGuess>, previous: PreviousWords): List<String> {
            if (guesses.size < 2 || previous.slotCount == 0) return guesses.map { it.word }
            return guesses.map { it to it.cost - bonus(it.word, previous) }
                .sortedBy { it.second }
                .map { it.first.word }
        }

        /** How much [previous] favours [word], in nats. */
        fun bonus(word: String, previous: PreviousWords): Float =
            habitBonus(history.pairCount(previous, word)) + dictionaryBonus(word, previous)

        private fun habitBonus(count: Int): Float =
            if (count <= 0) 0f else min(HABIT_WEIGHT * ln(1f + count), MAX_HABIT_BONUS)

        /**
         * An n-gram is the word's own 0-255 probability raised towards 255 by how often it follows
         * [previous]. That it exists at all is most of the evidence, given how few there are.
         */
        private fun dictionaryBonus(word: String, previous: PreviousWords): Float {
            val lookup = lookup ?: return 0f
            val ngram = lookup.ngramProbability(previous, word)
            if (ngram == NOT_A_PROBABILITY) return 0f
            val rise = (ngram - lookup.probability(word).coerceAtLeast(0)).coerceAtLeast(0)
            return min(NGRAM_BONUS + rise * NGRAM_BONUS_PER_STEP, MAX_NGRAM_BONUS)
        }

        companion object {
            /** Per ln(1 + uses): 0.31 for one use, 0.49 for two, 1.0 (the most) from about ten. */
            const val HABIT_WEIGHT = 0.45f
            const val MAX_HABIT_BONUS = 1f

            /** For a dictionary n-gram, plus a little per step it rises above the word alone. */
            const val NGRAM_BONUS = 0.25f
            const val NGRAM_BONUS_PER_STEP = 0.01f
            const val MAX_NGRAM_BONUS = 0.6f

            const val MAX_BONUS = MAX_HABIT_BONUS + MAX_NGRAM_BONUS
        }
    }

    /**
     * The words swipe typing can produce and how strongly the glide classifier favours each: the
     * dictionary's words, plus the user's personal and learned words so their names and slang can
     * be swiped too. Immutable; a change makes a new instance ([withUserWords]), which is how the
     * classifier knows to re-index.
     *
     * Frequencies come from the dictionary itself, not a copy: [probability] looks a word up, so the
     * 160,000 words cost a list and no map. How a 0-255 probability becomes the classifier's
     * frequency is [frequencyOf].
     */
    class SwipeWords private constructor(
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
        fun frequency(word: String): Double {
            val best = max(probability(word), userWords[word] ?: NOT_A_PROBABILITY)
            return if (best == NOT_A_PROBABILITY) 0.0 else frequencyOf(best)
        }

        /** These dictionary words with [userWords] (see [userWords]); this same instance if they are unchanged. */
        fun withUserWords(userWords: Map<String, Int>): SwipeWords =
            if (userWords == this.userWords) this else SwipeWords(dictionaryWords, probability, userWords)

        companion object {
            val NONE = SwipeWords(emptyList(), { NOT_A_PROBABILITY }, emptyMap())

            const val MAX_PROBABILITY = 255

            /** Words this common or more count the same; see [frequencyOf]. */
            const val COMMON = 170

            /** What each step of probability below [COMMON] costs a word, in nats; see [frequencyOf]. */
            const val COST_PER_STEP = 0.007

            /**
             * A word typed [TypingHistory.KNOWN_AFTER] times counts as fairly common (the dictionary
             * has "hello" at 120), whatever the dictionary says. One value for every learned word, so
             * using a word again doesn't change the list and re-index the classifier.
             */
            const val LEARNED_PROBABILITY = 120

            private val CONNECTORS = setOf('\'', '’', '-')

            /**
             * The dictionary's [words], each found again by [probability]. Words a swipe can't spell on
             * a letters layout ("MP3", "US$") are left out.
             */
            fun fromDictionary(words: List<String>, probability: (String) -> Int) =
                SwipeWords(words.filter(::isSwipeable), probability, emptyMap())

            /**
             * The user's words as probabilities: [personal] entries at their own frequency, [learned]
             * spellings at [LEARNED_PROBABILITY]; the higher one where a word is in both.
             */
            fun userWords(personal: List<PersonalWords.Entry>, learned: Collection<String>): Map<String, Int> {
                val probabilities = HashMap<String, Int>()
                learned.filter(::isSwipeable).forEach { probabilities[it] = LEARNED_PROBABILITY }
                for (entry in personal) {
                    if (!isSwipeable(entry.word)) continue
                    val frequency = entry.frequency.coerceIn(0, MAX_PROBABILITY)
                    probabilities[entry.word] = max(probabilities[entry.word] ?: 0, frequency)
                }
                return probabilities
            }

            /**
             * The classifier's frequency for a word of [probability] (0-255): 1 from [COMMON] up, then
             * e^-([COST_PER_STEP] x steps below it), down to 0.30 for the rarest word.
             *
             * Calibrated by replaying the classifier's scoring on the host over the whole dictionary,
             * with swipes that miss key centres by 22 and 40 px (it reproduces the emulator's results
             * swipe for swipe). Weighted by how often each of the 1,500 commonest words is used, it
             * gets 98.1% and 86.1% of them, against 97.5% and 80.5% with FlorisBoard's word list, and
             * far more of the rarer words that list lacked. Common words count the same because
             * words sharing a path differ only by frequency ("to", "too": 0.07 nats apart), and
             * context then settles them (see [GlideRanker]).
             *
             * The price: a clean swipe separates neighbouring keys by little, so a commoner neighbour
             * can win ("an" beats "am" by 0.14), until the user's habit tips it.
             */
            fun frequencyOf(probability: Int): Double =
                exp(-COST_PER_STEP * (COMMON - probability.coerceIn(0, MAX_PROBABILITY)).coerceAtLeast(0))

            /** Letters, with apostrophes or hyphens inside, which a swipe skips over. */
            fun isSwipeable(word: String): Boolean =
                word.any(Char::isLetter) && word.all { it.isLetter() || it in CONNECTORS }
        }
    }
}
