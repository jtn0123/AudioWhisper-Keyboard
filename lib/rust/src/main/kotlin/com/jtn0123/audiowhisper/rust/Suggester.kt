package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

class Learner(private val store: LearnedWordStore) {
    private val handle = SuggesterNative.newLearner()

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** Production engine exchange. No Rust → Kotlin dictionary callback. */
    fun learnNative(word: String, textBefore: String, start: Int, swiped: Boolean, dictionary: LatinSession?): Boolean {
        fun call(session: Long) = NativeLearnerNative.call(handle, store.handle, session, word, textBefore, start, swiped)
        return (dictionary?.withHandle(::call) ?: call(-1L)).also { Reference.reachabilityFence(this) }
    }

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

internal object NativeLearnerNative {
    init { RustLibrary.load() }
    @JvmStatic external fun call(learner: Long, store: Long, session: Long, word: String,
        textBefore: String, start: Int, swiped: Boolean): Boolean
}
