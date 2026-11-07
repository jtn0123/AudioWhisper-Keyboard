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
    /** The user's personal dictionaries as they stand now (read on every call, so it can be swapped). */
    private val personalWords: () -> PersonalWords = { PersonalWords.NONE },
    /** What the user has typed before ([LearnedWords]), or [TypingHistory.NONE] if not learning. */
    private val history: () -> TypingHistory = { TypingHistory.NONE },
) {
    /** A decoder result and how it is shown, after matching the typed word's case. */
    private data class Ranked(val raw: RawSuggestion, val display: String)

    /** The candidate strip for [state]: at most [maxCount] words, auto-correction first. */
    fun suggest(state: TypingState, maxCount: Int, blockOffensiveWords: Boolean): List<ArrangedWord> {
        if (maxCount <= 0) return emptyList()
        if (state.typed.isEmpty()) return predictions(state.previousWords, maxCount, blockOffensiveWords)
        val personal = personalWords()
        val history = history()
        val input = WordCase.decoderInput(state.typed, locale)
        val ranked = rank(state.typed, decoder.decode(input, state.previousWords, blockOffensiveWords))
        val corrections = ranked.filter { it.display != state.typed }
        val ownSpelling = personal.spellingOf(state.typed) ?: learnedSpellingOf(history, state.typed)
        val autoCorrection = autoCorrectionFor(state, input, ranked, ownSpelling)
        val case = WordCase.of(state.typed)
        val ownWords = ownCompletions(personal, history, state.typed).map { case.apply(it, locale) }
        return buildList {
            if (autoCorrection != null) add(ArrangedWord(autoCorrection, isAutoCorrection = true))
            add(ArrangedWord(state.typed, isTypedWord = true))
            for (word in ownWords) add(ArrangedWord(word))
            for (correction in acronymAfterCompletions(state.typed, corrections)) add(ArrangedWord(correction.display))
        }.distinctBy { it.text }.take(maxCount)
    }

    /**
     * The user's own words that continue [typed]: personal dictionary entries first, then words
     * they have used that the dictionary lacks (it already offers its own, ranked by frequency).
     */
    private fun ownCompletions(personal: PersonalWords, history: TypingHistory, typed: String): List<String> {
        val listed = personal.completions(typed, PersonalWords.MAX_IN_STRIP)
        val learned = history.completions(typed, PersonalWords.MAX_IN_STRIP, accept = ::isUnknown)
        return (listed + learned).distinctBy { it.lowercase(locale) }.take(PersonalWords.MAX_IN_STRIP)
    }

    /** A learned spelling counts only for a word the dictionary lacks: it knows its own words better. */
    private fun learnedSpellingOf(history: TypingHistory, typed: String): String? =
        history.spellingOf(typed)?.takeIf(::isUnknown)

    private fun isUnknown(word: String): Boolean =
        decoder.probability(word) == NOT_A_PROBABILITY && decoder.probability(word.lowercase(locale)) == NOT_A_PROBABILITY

    /**
     * What [state]'s word is auto-corrected to, if anything. One of the user's own words is never
     * corrected away; typed in another case ("iphone") and unknown to the dictionary, it takes the
     * user's spelling ("iPhone").
     */
    private fun autoCorrectionFor(state: TypingState, input: String, ranked: List<Ranked>, ownSpelling: String?): String? {
        val isTypedValid = isValid(state.typed, input, ranked)
        if (ownSpelling != null) {
            return ownSpelling.takeIf { it != state.typed && !isTypedValid && isCorrectable(state) }
        }
        return ranked.firstOrNull { it.display != state.typed }
            ?.takeIf { shouldAutoCorrect(state, input, it, isTypedValid) }
            ?.display
    }

    /** Spell-checks [word] with no context. */
    fun spell(word: String, maxCount: Int, blockOffensiveWords: Boolean): SpellCheck {
        if (personalWords().spellingOf(word) != null || learnedSpellingOf(history(), word) != null) {
            return SpellCheck(true, emptyList(), false)
        }
        val input = WordCase.decoderInput(word, locale)
        val ranked = rank(word, decoder.decode(input, PreviousWords.NONE, blockOffensiveWords))
        if (isValid(word, input, ranked)) return SpellCheck(true, emptyList(), false)
        val corrections = ranked.filter { it.display != word }
        val state = TypingState(word, PreviousWords.NONE, isCursorMidWord = false)
        val confident = corrections.firstOrNull()?.let { shouldAutoCorrect(state, input, it, isTypedValid = false) } == true
        return SpellCheck(false, corrections.take(maxCount).map { it.display }, confident)
    }

    /**
     * Next words after [previous]: those the user has typed there more than once, then the
     * dictionary's (it knows at most three per word), then those typed there once.
     */
    private fun predictions(previous: PreviousWords, maxCount: Int, blockOffensiveWords: Boolean): List<ArrangedWord> {
        if (previous.slotCount == 0) return emptyList()
        val history = history()
        val habits = history.followers(previous, TypingHistory.KNOWN_AFTER, maxCount)
        val dictionary = decoder.decode("", previous, blockOffensiveWords).map { it.word }
        val once = history.followers(previous, minCount = 1, maxCount)
        return (habits + dictionary + once).distinctBy { it.lowercase(locale) }.take(maxCount).map { ArrangedWord(it) }
    }

    /** Case-matched, de-duplicated results, best first. */
    private fun rank(typed: String, results: List<RawSuggestion>): List<Ranked> {
        val case = WordCase.of(typed)
        val ranked = results.asSequence()
            .filter { it.word.isNotEmpty() }
            .map { Ranked(it, case.apply(it.word, locale)) }
            .distinctBy { it.display }
            .toList()
        // Capitalized or all-caps typing already gave every result the same case.
        return if (typed.any(Char::isUpperCase)) ranked else oneSpellingPerWord(typed, ranked)
    }

    /**
     * For a word typed in lower case, where results keep the dictionary's spelling: no second
     * spelling of a typed word that is itself a word ("jello" doesn't also offer "Jello"), and one
     * spelling per word, lower case preferred ("jelo" offers "jello", not both).
     */
    private fun oneSpellingPerWord(typed: String, ranked: List<Ranked>): List<Ranked> {
        val typedIsWord = ranked.any { it.display == typed } || decoder.probability(typed) != NOT_A_PROBABILITY
        val kept = ranked.filter { it.display == typed || !(typedIsWord && it.display.equals(typed, ignoreCase = true)) }
        return kept.groupBy { it.display.lowercase(locale) }.values.map { spellings ->
            spellings.firstOrNull { it.display.none(Char::isUpperCase) } ?: spellings.first()
        }
    }

    /**
     * Strip order only (auto-correction is decided first): an acronym spelled with exactly the
     * typed letters goes after the more frequent words that continue them, so "wor" shows "work"
     * and "word" before "WOR". With nothing like that ahead of it, it stays ("ok" still offers "OK").
     */
    private fun acronymAfterCompletions(typed: String, corrections: List<Ranked>): List<Ranked> {
        val acronym = corrections.firstOrNull { it.display.equals(typed, ignoreCase = true) && isAcronym(it.display) }
            ?: return corrections
        val frequency = decoder.probability(acronym.raw.word)
        val rest = corrections.filter { it !== acronym }
        val lastAhead = rest.indexOfLast {
            it.display.startsWith(typed, ignoreCase = true) && decoder.probability(it.raw.word) > frequency
        }
        // Already behind all of them (or nothing to go behind): leave the order alone.
        if (lastAhead < corrections.indexOf(acronym)) return corrections
        return rest.toMutableList().apply { add(lastAhead + 1, acronym) }
    }

    private fun isAcronym(word: String): Boolean {
        val letters = word.filter(Char::isLetter)
        return letters.length > 1 && letters.none(Char::isLowerCase)
    }

    /** The typed word is a dictionary word (in the case the user typed, or with the first letter lowered). */
    private fun isValid(typed: String, input: String, ranked: List<Ranked>): Boolean =
        ranked.any { it.display == typed } ||
            decoder.probability(typed) != NOT_A_PROBABILITY ||
            (input != typed && decoder.probability(input) != NOT_A_PROBABILITY)

    /** AOSP's gates (no correction mid-word, with digits, in mostly-caps words, to predictions or shortcuts). */
    private fun shouldAutoCorrect(state: TypingState, input: String, top: Ranked, isTypedValid: Boolean): Boolean {
        if (!isCorrectable(state)) return false
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

    /** AOSP never auto-corrects mid-word, a word with digits, or a mostly-caps word. */
    private fun isCorrectable(state: TypingState): Boolean =
        !state.isCursorMidWord && state.typed.none(Char::isDigit) && !WordCase.isMostlyCaps(state.typed)

    private fun isCaseOnlyAcronym(typed: String, display: String): Boolean =
        display.equals(typed, ignoreCase = true) && display.none(Char::isLowerCase) && typed.any(Char::isLowerCase)

    companion object {
        /** How much more frequent (0-255 log scale) a suggestion must be to replace a valid word. */
        const val RARER_BY = 50
    }
}
