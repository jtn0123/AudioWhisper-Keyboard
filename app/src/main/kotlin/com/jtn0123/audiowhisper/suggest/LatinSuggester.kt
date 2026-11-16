package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules


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
            isCursorMidWord = com.jtn0123.audiowhisper.rust.SuggestRules.isCursorMidWord(textAfterCursor.toString()),
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
 * them); the native session calls the Rust decoder in-process.
 *
 * Case is matched and ignored as Locale.US does, the only locale the keyboard suggests in.
 */
class LatinSuggester(
    private val decoder: LatinDictionaryEngine,
    /** How readily to auto-correct, read on every strip (the setting can change while typing). */
    private val strength: () -> AutoCorrectStrength = { AutoCorrectStrength.NORMAL },
    /** The user's personal dictionaries as they stand now (read on every call, so it can be swapped). */
    private val personalWords: () -> PersonalWords = { PersonalWords.NONE },
    /** What the user has typed before ([LearnedWords]), or null if not learning. */
    private val history: () -> LearnedWords? = { null },
) {
    /** The candidate strip for [state]: at most [maxCount] words, auto-correction first. */
    fun suggest(state: TypingState, maxCount: Int, blockOffensiveWords: Boolean): List<ArrangedWord> {
        val previous = state.previousWords
        val strip = decoder.session.suggest(
            personalWords().store, history()?.store, state.typed, previous.words, previous.isBeginningOfSentence,
            state.isCursorMidWord, blockOffensiveWords, state.taps.packed(), maxCount, strength().ordinal,
        )
        return strip.words.mapIndexed { index, word ->
            ArrangedWord(word, isAutoCorrection = index == strip.autoCorrection, isTypedWord = index == strip.typedWord)
        }
    }

    /** Spell-checks [word] with no context, as confident as the default strength whatever the setting. */
    fun spell(word: String, maxCount: Int, blockOffensiveWords: Boolean): SpellCheck {
        val threshold = SuggestRules.modestThreshold
        val verdict = decoder.session.spell(personalWords().store, history()?.store, word, blockOffensiveWords, maxCount, threshold)
        return SpellCheck(verdict.isValid, verdict.suggestions, verdict.isHighConfidence)
    }
}
