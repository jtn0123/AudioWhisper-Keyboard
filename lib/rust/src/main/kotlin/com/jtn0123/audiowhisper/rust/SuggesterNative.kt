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

    @JvmStatic external fun suggest(
        personal: Long,
        learned: Long,
        dictionary: SuggestDictionary,
        typed: String,
        previousWords: Array<String>,
        isBeginningOfSentence: Boolean,
        isCursorMidWord: Boolean,
        maxCount: Int,
        threshold: Float,
        marks: IntArray,
    ): Array<String>

    @JvmStatic external fun spell(
        personal: Long,
        learned: Long,
        dictionary: SuggestDictionary,
        word: String,
        maxCount: Int,
        threshold: Float,
        verdict: IntArray,
    ): Array<String>

    @JvmStatic external fun newLearner(): Long

    @JvmStatic external fun freeLearner(learner: Long)

    @JvmStatic external fun learnerFinished(
        learner: Long,
        store: Long,
        dictionary: WordProbabilities?,
        word: String,
        textBefore: String,
        start: Int,
    ): Boolean

    @JvmStatic external fun learnerSwiped(
        learner: Long,
        store: Long,
        dictionary: WordProbabilities?,
        word: String,
        textBefore: String,
    ): Boolean

    @JvmStatic external fun learnerDropSwiped(learner: Long)

    @JvmStatic external fun learnerUndo(learner: Long, store: Long, start: Int): Boolean

    @JvmStatic external fun learnerReset(learner: Long)
}
