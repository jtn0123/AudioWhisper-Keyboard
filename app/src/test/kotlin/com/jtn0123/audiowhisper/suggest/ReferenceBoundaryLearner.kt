package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.LegacyLearner
import com.jtn0123.audiowhisper.rust.WordProbabilities

/**
 * Feeds [history] the words the user finishes. A typed word is finished by a space, punctuation
 * or Enter (after any auto-correction), or by tapping a candidate for it. A swiped word waits for
 * what comes next: the next word or a separator keeps it, a candidate tap or a backspace means it
 * was wrong. Finishing a different word where the last one began (the user went back and changed
 * it) takes the last one back first, as does undoing its auto-correction.
 *
 * A dictionary word capitalised only because it starts the sentence ("The"), or typed in caps
 * lock ("HELLO"), is learned in lower case; other words keep their capitals. The rules are in Rust
 * (lib/rust/crates/suggest/src/word_learner.rs), which asks [lookup] how words are spelled.
 *
 * Call from the main thread only. Never logs a word.
 */
class ReferenceBoundaryLearner(history: LearnedWords, private val lookup: () -> WordLookup?) {
    private val learner = LegacyLearner(history.store)

    /** The dictionary, as the Rust asks it. */
    private class Probabilities(private val lookup: WordLookup) : WordProbabilities {
        override fun probability(word: String): Int = lookup.probability(word)
    }

    /**
     * [word], which starts at [start] in the editor right after [textBefore], is finished.
     * Returns true if [history] changed.
     */
    fun finished(word: String, textBefore: CharSequence, start: Int): Boolean =
        learner.finished(word, textBefore.toString(), start, dictionary())

    /** A swiped [word] went in after [textBefore]; a swiped word still waiting before it is kept. */
    fun swiped(word: String, textBefore: CharSequence): Boolean =
        learner.swiped(word, textBefore.toString(), dictionary())

    /** The swiped word waiting to be kept was replaced or deleted. */
    fun dropSwiped() = learner.dropSwiped()

    /** The auto-correction that starts at [start] was undone. Returns true if [history] changed. */
    fun undo(start: Int): Boolean = learner.undo(start)

    /** Another field: positions there say nothing about what was learned here. */
    fun reset() = learner.reset()

    private fun dictionary(): WordProbabilities? = lookup()?.let(::Probabilities)
}
