package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.LegacySuggester
import com.jtn0123.audiowhisper.rust.SuggestDictionary
import com.jtn0123.audiowhisper.rust.SuggestRules

/**
 * The Rust suggester (lib/rust/crates/suggest/src/suggester.rs, which documents its rules) over any
 * [TypingDecoder], so tests can drive it with a fake one: it runs [decoder] for the Rust, as the
 * Rust asks. [LatinSuggester] runs the same rules over the native session's decoder. Once named
 * ReferenceLatinSuggester, though it copies no Kotlin (docs/phase6-kotlin-references.md).
 *
 * Case is matched and ignored as Locale.US does, the only locale the keyboard suggests in.
 */
class FixtureSuggester(
    private val decoder: TypingDecoder,
    private val threshold: Float = SuggestRules.modestThreshold,
    /** The user's personal dictionaries as they stand now (read on every call, so it can be swapped). */
    private val personalWords: () -> PersonalWords = { PersonalWords.NONE },
    /** What the user has typed before ([LearnedWords]), or null if not learning. */
    private val history: () -> LearnedWords? = { null },
) {
    /** [decoder] around one request for [state]'s word, as the Rust asks it. */
    private class Request(
        private val decoder: TypingDecoder,
        private val state: TypingState,
        private val blockOffensiveWords: Boolean,
    ) : SuggestDictionary {
        override fun probability(word: String): Int = decoder.probability(word)

        override fun ngramProbability(word: String): Int = decoder.ngramProbability(state.previousWords, word)

        override fun decode(input: String, inContext: Boolean, withTaps: Boolean): IntArray {
            val previous = if (inContext) state.previousWords else PreviousWords.NONE
            val taps = if (withTaps) state.taps else emptyList()
            val results = decoder.decode(input, previous, blockOffensiveWords, taps)
            return SuggestDictionary.pack(
                results.map { it.word },
                IntArray(results.size) { results[it].score },
                IntArray(results.size) { results[it].kindAndFlags },
            )
        }
    }

    /** The candidate strip for [state]: at most [maxCount] words, auto-correction first. */
    fun suggest(state: TypingState, maxCount: Int, blockOffensiveWords: Boolean): List<ArrangedWord> {
        val previous = state.previousWords
        val strip = LegacySuggester.suggest(
            personalWords().store, history()?.store, Request(decoder, state, blockOffensiveWords), state.typed,
            previous.words, previous.isBeginningOfSentence, state.isCursorMidWord, maxCount, threshold,
        )
        return strip.words.mapIndexed { index, word ->
            ArrangedWord(word, isAutoCorrection = index == strip.autoCorrection, isTypedWord = index == strip.typedWord)
        }
    }

    /** Spell-checks [word] with no context. */
    fun spell(word: String, maxCount: Int, blockOffensiveWords: Boolean): SpellCheck {
        val request = Request(decoder, TypingState(word, PreviousWords.NONE, isCursorMidWord = false), blockOffensiveWords)
        val verdict = LegacySuggester.spell(personalWords().store, history()?.store, request, word, maxCount, threshold)
        return SpellCheck(verdict.isValid, verdict.suggestions, verdict.isHighConfidence)
    }
}
