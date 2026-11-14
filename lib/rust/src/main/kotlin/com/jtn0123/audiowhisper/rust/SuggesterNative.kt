package com.jtn0123.audiowhisper.rust

/**
 * The candidate strip's, the spell checker's and the word learner's entry points in libawnative:
 * lib/rust/crates/awnative/src/suggest/suggester.rs and word_learner.rs, which document each.
 * [Suggester] and [Learner] wrap them.
 */
internal object SuggesterNative {
    init {
        RustLibrary.load()
    }



    @JvmStatic external fun newLearner(): Long

    @JvmStatic external fun freeLearner(learner: Long)



    @JvmStatic external fun learnerDropSwiped(learner: Long)

    @JvmStatic external fun learnerUndo(learner: Long, store: Long, start: Int): Boolean

    @JvmStatic external fun learnerReset(learner: Long)
}
