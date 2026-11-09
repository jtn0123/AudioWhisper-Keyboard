package com.jtn0123.audiowhisper.suggest

import java.util.Locale

/**
 * The suggester and the word learner as they were in Kotlin before moving to Rust
 * (lib/rust/crates/suggest), copied unchanged from commit f89496da: LatinSuggester.kt's
 * LatinSuggester (its TypingState, ArrangedWord and SpellCheck are the app's) and WordLearner.kt.
 * The constants they read from PersonalWords and TypingHistory are copied in too, as Rust holds
 * those now. They are what [LatinRulesMatchTest] holds the Rust to; don't change them.
 * [TypingDecoder], [PersonalWords], [LearnedWords], [WordCase], [AutoCorrection] and
 * [PreviousWords] are the app's.
 */
object ReferenceLatin {
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
            val input = WordCase.decoderInput(state.typed)
            val ranked = rank(state.typed, decoder.decode(input, state.previousWords, blockOffensiveWords, state.taps))
            val corrections = ranked.filter { it.display != state.typed }
            val ownSpelling = personal.spellingOf(state.typed) ?: learnedSpellingOf(history, state.typed)
            val autoCorrection = autoCorrectionFor(state, input, ranked, ownSpelling)
            val case = WordCase.of(state.typed)
            val ownWords = ownCompletions(personal, history, state.typed).map { case.apply(it) }
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
            val listed = personal.completions(typed, MAX_IN_STRIP)
            val learned = history.completions(typed, MAX_IN_STRIP, accept = ::isUnknown)
            return (listed + learned).distinctBy { it.lowercase(locale) }.take(MAX_IN_STRIP)
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
            val input = WordCase.decoderInput(word)
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
            val habits = history.followers(previous, KNOWN_AFTER, maxCount)
            val dictionary = decoder.decode("", previous, blockOffensiveWords).map { it.word }
            val once = history.followers(previous, minCount = 1, maxCount)
            return (habits + dictionary + once).distinctBy { it.lowercase(locale) }.take(maxCount).map { ArrangedWord(it) }
        }

        /** Case-matched, de-duplicated results, best first. */
        private fun rank(typed: String, results: List<RawSuggestion>): List<Ranked> {
            val case = WordCase.of(typed)
            val ranked = results.asSequence()
                .filter { it.word.isNotEmpty() }
                .map { Ranked(it, case.apply(it.word)) }
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

            /** PersonalWords.MAX_IN_STRIP and TypingHistory.KNOWN_AFTER, which Rust now holds. */
            const val MAX_IN_STRIP = 2
            const val KNOWN_AFTER = 2
        }
    }

    /**
     * Feeds [history] the words the user finishes. A typed word is finished by a space, punctuation
     * or Enter (after any auto-correction), or by tapping a candidate for it. A swiped word waits for
     * what comes next: the next word or a separator keeps it, a candidate tap or a backspace means it
     * was wrong. Finishing a different word where the last one began (the user went back and changed
     * it) takes the last one back first, as does undoing its auto-correction.
     *
     * Call from the main thread only. Never logs a word.
     */
    class WordLearner(
        private val history: LearnedWords,
        private val lookup: () -> WordLookup?,
        private val locale: Locale = Locale.US,
    ) {
        /** What one [learn] stored, so it can be taken back. [word] is as committed, [spelling] as stored. */
        private data class Learned(val word: String, val spelling: String, val previous: String?, val start: Int)

        private class Swiped(val word: String, val textBefore: String)

        private var last: Learned? = null
        private var swiped: Swiped? = null

        /**
         * [word], which starts at [start] in the editor right after [textBefore], is finished.
         * Returns true if [history] changed.
         */
        fun finished(word: String, textBefore: CharSequence, start: Int): Boolean {
            val pending = swiped
            swiped = null
            // A different word after a swiped one: the swipe was kept and typing went on.
            var changed = pending != null && pending.word != word && learn(pending.word, pending.textBefore, NO_POSITION)
            val previous = last
            if (previous != null && start >= 0 && previous.start == start) {
                if (previous.word == word) return changed
                takeBack(previous)
                changed = true
            }
            return learn(word, textBefore, start) || changed
        }

        /** A swiped [word] went in after [textBefore]; a swiped word still waiting before it is kept. */
        fun swiped(word: String, textBefore: CharSequence): Boolean {
            val pending = swiped
            swiped = Swiped(word, textBefore.toString())
            return pending != null && learn(pending.word, pending.textBefore, NO_POSITION)
        }

        /** The swiped word waiting to be kept was replaced or deleted. */
        fun dropSwiped() {
            swiped = null
        }

        /** The auto-correction that starts at [start] was undone. Returns true if [history] changed. */
        fun undo(start: Int): Boolean {
            val learned = last?.takeIf { start >= 0 && it.start == start } ?: return false
            takeBack(learned)
            return true
        }

        /** Another field: positions there say nothing about what was learned here. */
        fun reset() {
            last = null
            swiped = null
        }

        private fun learn(word: String, textBefore: CharSequence, start: Int): Boolean {
            if (!LearnedWords.isStorable(word)) return false
            val context = PreviousWords.from(textBefore)
            val previous = LearnedWords.previousKey(context)
            val startsSentence = context.words.isEmpty() && context.isBeginningOfSentence
            val spelling = spellingToLearn(word, startsSentence)
            // A capital the sentence start may have put there doesn't replace a spelling seen mid-sentence.
            val keepSpelling = startsSentence && spelling == word && WordCase.of(word) == WordCase.CAPITALIZED
            history.add(spelling, previous, keepSpelling)
            last = Learned(word, spelling, previous, start)
            return true
        }

        /**
         * The spelling to keep for [word]: a dictionary word capitalised only because it starts the
         * sentence ("The"), or typed in caps lock ("HELLO"), is learned in lower case. Words the
         * dictionary knows as written ("John", "USA") and words it doesn't know keep their capitals.
         */
        private fun spellingToLearn(word: String, startsSentence: Boolean): String {
            val lookup = lookup() ?: return word
            val lower = word.lowercase(locale)
            if (lower == word || lookup.probability(word) != NOT_A_PROBABILITY) return word
            val case = WordCase.of(word)
            val capitalOnlyForSentence = case == WordCase.CAPITALIZED && startsSentence
            if (!capitalOnlyForSentence && case != WordCase.ALL_CAPS) return word
            return if (lookup.probability(lower) != NOT_A_PROBABILITY) lower else word
        }

        private fun takeBack(learned: Learned) {
            history.remove(learned.spelling, learned.previous)
            last = null
        }

        private companion object {
            /** Where a swiped word starts is not known: the space before it may be added later. */
            const val NO_POSITION = -1
        }
    }
}
