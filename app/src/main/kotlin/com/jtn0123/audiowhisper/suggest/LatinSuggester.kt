package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestDictionary
import com.jtn0123.audiowhisper.rust.Suggester

/** What surrounds the cursor when suggestions are requested. */
data class TypingState(
    /** The word being typed, left of the cursor; empty for next-word prediction. */
    val typed: String,
    val previousWords: PreviousWords,
    /** The cursor sits inside a word (letters follow it); AOSP never auto-corrects then. */
    val isCursorMidWord: Boolean,
    /** Where each character of [typed] was tapped, if known ([TapTrail.offsetsFor]). */
    val taps: List<TapOffset?> = emptyList(),
) {
    companion object {
        /** The state for [typed], which starts right after [textBeforeWord] and ends at the cursor. */
        fun of(textBeforeWord: CharSequence, typed: String, textAfterCursor: CharSequence) = TypingState(
            typed = typed,
            previousWords = PreviousWords.from(textBeforeWord),
            isCursorMidWord = textAfterCursor.firstOrNull()?.isLetterOrDigit() == true,
        )
    }
}

/** One entry of the candidate strip. At most one per list has [isAutoCorrection] set, the first. */
data class ArrangedWord(
    val text: String,
    val isAutoCorrection: Boolean = false,
    /** The literal typed word, kept so the user can always tap it to keep what they typed. */
    val isTypedWord: Boolean = false,
)

/** A spell-check verdict for one word. */
data class SpellCheck(val isValid: Boolean, val suggestions: List<String>, val isHighConfidence: Boolean)

/**
 * Turns decoder output into the candidate strip and decides auto-correction, following AOSP
 * `Suggest.getSuggestedWordsForNonBatchInput` and `AutoCorrectionUtils`, plus three narrow rules
 * of our own. The rules are in Rust (lib/rust/crates/suggest/src/suggester.rs, which documents
 * them); this runs [decoder] for it, as the Rust asks.
 *
 * Case is matched and ignored as Locale.US does, the only locale the keyboard suggests in.
 */
class LatinSuggester(
    private val decoder: TypingDecoder,
    private val threshold: Float = AutoCorrection.MODEST_THRESHOLD,
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
        val strip = Suggester.suggest(
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
        val verdict = Suggester.spell(personalWords().store, history()?.store, request, word, maxCount, threshold)
        return SpellCheck(verdict.isValid, verdict.suggestions, verdict.isHighConfidence)
    }
}
