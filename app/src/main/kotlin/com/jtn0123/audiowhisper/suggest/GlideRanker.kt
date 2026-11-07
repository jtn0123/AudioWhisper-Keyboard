package com.jtn0123.audiowhisper.suggest

import kotlin.math.ln
import kotlin.math.min

/**
 * A swipe-typing guess: [word] and how badly the swipe fits it, in nats (the glide classifier's
 * -ln of shape fit x location fit x word frequency). Lower is better.
 */
data class GlideGuess(val word: String, val cost: Float)

/**
 * Re-ranks swipe guesses with the words before them, so the swipe for "to" after "me" gives
 * "too" once the user has typed "me too". Each guess's cost goes down by what the context says
 * for it:
 *  - the user's habit: how often they typed it after that word ([TypingHistory.pairCount]);
 *  - the dictionary's n-gram, which the bundled one has for at most three next words a word.
 * A missing pair says little, so neither ever makes a guess worse.
 *
 * The bonuses are small on purpose: together at most [MAX_BONUS]. The classifier's own costs are
 * close together (its word frequencies are squeezed into 128-255): words with the same path
 * ("to", "too") differ by about 0.01, and even a clean swipe beats a neighbouring key's word
 * ("am", "an") by only 0.1-0.5. So context settles those, but not a swipe that clearly traces a
 * different word ("you" over "thou": 1.4-2.2).
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
