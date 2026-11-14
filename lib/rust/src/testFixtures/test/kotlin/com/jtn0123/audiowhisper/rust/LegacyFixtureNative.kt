package com.jtn0123.audiowhisper.rust

internal object LegacyFixtureNative {
    init { RustLibrary.load() }

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

    @JvmStatic external fun join(before: String, lastPhrase: String?, phrase: String, names: DictationNames): String
}
