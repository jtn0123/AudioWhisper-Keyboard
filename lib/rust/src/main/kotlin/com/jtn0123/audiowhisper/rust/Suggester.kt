package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * A dictionary's probability for a word, which Rust calls back (see lib/rust/consumer-rules.pro):
 * on its 0-255 log scale, or -1 for a word it lacks.
 */
interface WordProbabilities {
    fun probability(word: String): Int
}

/**
 * The decoder and its dictionary around one request for suggestions, which Rust calls back (see
 * lib/rust/consumer-rules.pro). It knows the words before the one typed and where its letters were
 * tapped.
 */
interface SuggestDictionary : WordProbabilities {
    /** [word]'s probability right after the request's previous words, or -1 for none. */
    fun ngramProbability(word: String): Int

    /**
     * The decoder's results for [input], highest score first, [pack]ed: after the request's
     * previous words if [inContext], with where its letters were tapped if [withTaps].
     */
    fun decode(input: String, inContext: Boolean, withTaps: Boolean): IntArray

    companion object {
        /** Decoder results as Rust reads them: each one's score, kind and flags, length, then UTF-16 units. */
        fun pack(words: List<String>, scores: IntArray, kindsAndFlags: IntArray): IntArray {
            require(scores.size == words.size && kindsAndFlags.size == words.size) { "results differ in length" }
            val packed = IntArray(words.sumOf { it.length + 3 })
            var at = 0
            words.forEachIndexed { index, word ->
                packed[at++] = scores[index]
                packed[at++] = kindsAndFlags[index]
                packed[at++] = word.length
                for (char in word) packed[at++] = char.code
            }
            return packed
        }
    }
}

/**
 * The candidate strip and the spell checker, in Rust (lib/rust/crates/suggest/src/suggester.rs),
 * drawing on the personal and learned words in their stores and asking a [SuggestDictionary]. The
 * keyboard's `LatinSuggester` calls these and documents the rules.
 */
object Suggester {
    /** A candidate strip: its words, best first, and where its auto-correction and typed word are (-1 for none). */
    class Strip(val words: List<String>, val autoCorrection: Int, val typedWord: Int)

    /** A spell checker's verdict on one word. */
    class Verdict(val isValid: Boolean, val suggestions: List<String>, val isHighConfidence: Boolean)

    /**
     * At most [maxCount] words for [typed], typed after [previousWords] (most recent first) and a
     * sentence start if [isBeginningOfSentence], with letters after the cursor if
     * [isCursorMidWord]. [learned] is null if the keyboard isn't learning.
     */
    fun suggest(
        personal: PersonalWordStore,
        learned: LearnedWordStore?,
        dictionary: SuggestDictionary,
        typed: String,
        previousWords: List<String>,
        isBeginningOfSentence: Boolean,
        isCursorMidWord: Boolean,
        maxCount: Int,
        threshold: Float,
    ): Strip {
        val marks = IntArray(2)
        val words = SuggesterNative.suggest(
            personal.handle, learned?.handle ?: 0L, dictionary, typed, previousWords.toTypedArray(),
            isBeginningOfSentence, isCursorMidWord, maxCount, threshold, marks,
        )
        Reference.reachabilityFence(personal)
        Reference.reachabilityFence(learned)
        return Strip(words.asList(), marks[0], marks[1])
    }

    /**
     * Spell-checks [word] with no context, suggesting at most [maxCount] corrections; a negative
     * [maxCount] throws [IllegalArgumentException] once it comes to suggesting.
     */
    fun spell(
        personal: PersonalWordStore,
        learned: LearnedWordStore?,
        dictionary: SuggestDictionary,
        word: String,
        maxCount: Int,
        threshold: Float,
    ): Verdict {
        val verdict = IntArray(2)
        val suggestions = SuggesterNative.spell(
            personal.handle, learned?.handle ?: 0L, dictionary, word, maxCount, threshold, verdict,
        )
        Reference.reachabilityFence(personal)
        Reference.reachabilityFence(learned)
        return Verdict(verdict[0] != 0, suggestions.asList(), verdict[1] != 0)
    }
}

/**
 * What the keyboard learns from each finished word, in Rust (lib/rust/crates/suggest/src/
 * word_learner.rs): it feeds [store], asking the dictionary, if one is open, how to spell what it
 * learns. The keyboard's `WordLearner` calls this and documents the rules. Call from one thread.
 * Its native memory is freed once nothing can reach it.
 */
class Learner(private val store: LearnedWordStore) {
    private val handle = SuggesterNative.newLearner()

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** [word], which starts at [start] right after [textBefore], is finished. True if [store] changed. */
    fun finished(word: String, textBefore: String, start: Int, dictionary: WordProbabilities?): Boolean =
        SuggesterNative.learnerFinished(handle, store.handle, dictionary, word, textBefore, start)
            .also { Reference.reachabilityFence(this) }

    /** A swiped [word] went in after [textBefore]. True if [store] changed. */
    fun swiped(word: String, textBefore: String, dictionary: WordProbabilities?): Boolean =
        SuggesterNative.learnerSwiped(handle, store.handle, dictionary, word, textBefore)
            .also { Reference.reachabilityFence(this) }

    /** The swiped word waiting to be kept was replaced or deleted. */
    fun dropSwiped() {
        SuggesterNative.learnerDropSwiped(handle)
        Reference.reachabilityFence(this)
    }

    /** The auto-correction that starts at [start] was undone. True if [store] changed. */
    fun undo(start: Int): Boolean =
        SuggesterNative.learnerUndo(handle, store.handle, start).also { Reference.reachabilityFence(this) }

    /** Another field: forgets where words were. */
    fun reset() {
        SuggesterNative.learnerReset(handle)
        Reference.reachabilityFence(this)
    }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = SuggesterNative.freeLearner(handle)
    }
}
