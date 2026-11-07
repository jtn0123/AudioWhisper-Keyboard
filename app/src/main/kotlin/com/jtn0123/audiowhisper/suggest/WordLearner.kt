package com.jtn0123.audiowhisper.suggest

import java.util.Locale

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
