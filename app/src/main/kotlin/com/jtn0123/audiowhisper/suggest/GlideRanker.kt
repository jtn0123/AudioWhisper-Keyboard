package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SwipeRules

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
 * A missing pair says little, so neither ever makes a guess worse, and together they take off at
 * most [MAX_BONUS]: enough to settle words with the same path ("to", "too"), not to override a
 * swipe that clearly traces another word.
 *
 * The rules and their numbers are in Rust ([SwipeRules], lib/rust/crates/suggest/src/
 * glide_ranker.rs); this asks the learned words and the dictionary what they know of each guess.
 */
class GlideRanker(private val history: TypingHistory, private val lookup: WordLookup?) {
    /** [guesses], best first in [previous]'s context. Ties keep the classifier's order. */
    fun rank(guesses: List<GlideGuess>, previous: PreviousWords): List<String> {
        if (guesses.size < 2 || previous.slotCount == 0) return guesses.map { it.word }
        val costs = FloatArray(guesses.size) { guesses[it].cost }
        val order = SwipeRules.rankGlide(costs, guesses.map { context(it.word, previous) })
        return order.map { guesses[it].word }
    }

    /** How much [previous] favours [word], in nats. */
    fun bonus(word: String, previous: PreviousWords): Float {
        val context = context(word, previous)
        return SwipeRules.glideBonus(context.pairCount, context.ngram, context.probability)
    }

    /** What [previous] says for [word]: the user's habit, and any n-gram with the word's own probability. */
    private fun context(word: String, previous: PreviousWords): SwipeRules.Context {
        val ngram = lookup?.ngramProbability(previous, word) ?: NOT_A_PROBABILITY
        val probability = if (ngram == NOT_A_PROBABILITY) NOT_A_PROBABILITY else lookup?.probability(word) ?: NOT_A_PROBABILITY
        return SwipeRules.Context(history.pairCount(previous, word), ngram, probability)
    }

    companion object {
        /** The most the words before a guess can take off its cost, in nats. */
        val MAX_BONUS: Float = SwipeRules.maxGlideBonus
    }
}
