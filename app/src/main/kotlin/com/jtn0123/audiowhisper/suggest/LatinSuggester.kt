package com.jtn0123.audiowhisper.suggest

import java.util.Locale

/** What surrounds the cursor when suggestions are requested. */
data class TypingState(
    /** The word being typed, left of the cursor; empty for next-word prediction. */
    val typed: String,
    val previousWords: PreviousWords,
    /** The cursor sits inside a word (letters follow it); AOSP never auto-corrects then. */
    val isCursorMidWord: Boolean,
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
 * of our own (see [isAllowedToAutoCorrect]).
 */
class LatinSuggester(
    private val decoder: TypingDecoder,
    private val locale: Locale,
    private val threshold: Float = AutoCorrection.MODEST_THRESHOLD,
) {
    /** A decoder result and how it is shown, after matching the typed word's case. */
    private data class Ranked(val raw: RawSuggestion, val display: String)

    /** The candidate strip for [state]: at most [maxCount] words, auto-correction first. */
    fun suggest(state: TypingState, maxCount: Int, blockOffensiveWords: Boolean): List<ArrangedWord> {
        if (maxCount <= 0) return emptyList()
        if (state.typed.isEmpty()) return predictions(state.previousWords, maxCount, blockOffensiveWords)
        val input = WordCase.decoderInput(state.typed, locale)
        val ranked = rank(state.typed, decoder.decode(input, state.previousWords, blockOffensiveWords))
        val corrections = ranked.filter { it.display != state.typed }
        val autoCorrection = corrections.firstOrNull()
            ?.takeIf { shouldAutoCorrect(state, input, it, isValid(state.typed, input, ranked)) }
        return buildList {
            if (autoCorrection != null) add(ArrangedWord(autoCorrection.display, isAutoCorrection = true))
            add(ArrangedWord(state.typed, isTypedWord = true))
            for (correction in corrections) {
                if (correction !== autoCorrection) add(ArrangedWord(correction.display))
            }
        }.take(maxCount)
    }

    /** Spell-checks [word] with no context. */
    fun spell(word: String, maxCount: Int, blockOffensiveWords: Boolean): SpellCheck {
        val input = WordCase.decoderInput(word, locale)
        val ranked = rank(word, decoder.decode(input, PreviousWords.NONE, blockOffensiveWords))
        if (isValid(word, input, ranked)) return SpellCheck(true, emptyList(), false)
        val corrections = ranked.filter { it.display != word }
        val state = TypingState(word, PreviousWords.NONE, isCursorMidWord = false)
        val confident = corrections.firstOrNull()?.let { shouldAutoCorrect(state, input, it, isTypedValid = false) } == true
        return SpellCheck(false, corrections.take(maxCount).map { it.display }, confident)
    }

    private fun predictions(previous: PreviousWords, maxCount: Int, blockOffensiveWords: Boolean): List<ArrangedWord> {
        if (previous.slotCount == 0) return emptyList()
        return decoder.decode("", previous, blockOffensiveWords)
            .map { it.word }
            .distinct()
            .take(maxCount)
            .map { ArrangedWord(it) }
    }

    /** Case-matched, de-duplicated results, best first. */
    private fun rank(typed: String, results: List<RawSuggestion>): List<Ranked> {
        val case = WordCase.of(typed)
        return results.asSequence()
            .filter { it.word.isNotEmpty() }
            .map { Ranked(it, case.apply(it.word, locale)) }
            .distinctBy { it.display }
            .toList()
    }

    /** The typed word is a dictionary word (in the case the user typed, or with the first letter lowered). */
    private fun isValid(typed: String, input: String, ranked: List<Ranked>): Boolean =
        ranked.any { it.display == typed } ||
            decoder.probability(typed) != NOT_A_PROBABILITY ||
            (input != typed && decoder.probability(input) != NOT_A_PROBABILITY)

    /** AOSP's gates (no correction mid-word, with digits, in mostly-caps words, to predictions or shortcuts). */
    private fun shouldAutoCorrect(state: TypingState, input: String, top: Ranked, isTypedValid: Boolean): Boolean {
        val typed = state.typed
        if (state.isCursorMidWord || typed.any(Char::isDigit) || WordCase.isMostlyCaps(typed)) return false
        if (top.raw.isPrediction || top.raw.isShortcut) return false
        return isAllowedToAutoCorrect(state, input, top, isTypedValid) &&
            AutoCorrection.suggestionExceedsThreshold(top.raw, input, threshold)
    }

    /**
     * AOSP `allowsToBeAutoCorrected`: a whitelisted top word, or a typed word of two or more
     * letters that is not in the dictionary. Our additions: a lower-case word is never upper-cased
     * into an acronym ("ok" stays, not "OK"), and a valid word is still corrected when it
     * [isMuchRarer] than the suggestion in this context.
     */
    private fun isAllowedToAutoCorrect(state: TypingState, input: String, top: Ranked, isTypedValid: Boolean): Boolean {
        if (top.raw.isWhitelist) return true
        val typed = state.typed
        if (typed.codePointCount(0, typed.length) <= 1) return false
        if (isCaseOnlyAcronym(typed, top.display)) return false
        return !isTypedValid || isMuchRarer(input, top.raw, state.previousWords)
    }

    /**
     * A valid word that is a likely slip: the dictionary knows the suggestion after the previous
     * word but not the typed word, and the suggestion is at least [RARER_BY] steps more frequent
     * on the 0-255 log scale ("came fro" -> "came from", "a lit" -> "a lot").
     */
    private fun isMuchRarer(input: String, suggestion: RawSuggestion, previous: PreviousWords): Boolean {
        if (previous.words.isEmpty()) return false
        if (decoder.ngramProbability(previous, input) != NOT_A_PROBABILITY) return false
        if (decoder.ngramProbability(previous, suggestion.word) == NOT_A_PROBABILITY) return false
        val typedProbability = decoder.probability(input).coerceAtLeast(0)
        return decoder.probability(suggestion.word) - typedProbability >= RARER_BY
    }

    private fun isCaseOnlyAcronym(typed: String, display: String): Boolean =
        display.equals(typed, ignoreCase = true) && display.none(Char::isLowerCase) && typed.any(Char::isLowerCase)

    companion object {
        /** How much more frequent (0-255 log scale) a suggestion must be to replace a valid word. */
        const val RARER_BY = 50
    }
}
