package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

class LegacyLearner(private val store: LearnedWordStore) {
    private val handle = SuggesterNative.newLearner()

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** Legacy callback seam for reference comparisons; fixture callback entry. */
    fun finished(word: String, textBefore: String, start: Int, dictionary: WordProbabilities?): Boolean =
        LegacyFixtureNative.learnerFinished(handle, store.handle, dictionary, word, textBefore, start)
            .also { Reference.reachabilityFence(this) }

    /** Legacy callback seam for reference comparisons; fixture callback entry. */
    fun swiped(word: String, textBefore: String, dictionary: WordProbabilities?): Boolean =
        LegacyFixtureNative.learnerSwiped(handle, store.handle, dictionary, word, textBefore)
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
