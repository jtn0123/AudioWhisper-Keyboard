package com.jtn0123.audiowhisper.rust

/**
 * The word stores' entry points in libawnative: lib/rust/crates/awnative/src/suggest/
 * learned_words.rs, kept_words.rs and personal_words.rs, which document each. [WordStoreRules],
 * [LearnedWordStore], [KeptWordStore] and [PersonalWordStore] wrap them.
 */
internal object WordStoresNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun sentenceStart(): String

    @JvmStatic external fun knownAfter(): Int

    @JvmStatic external fun learnedWordCapacity(): Int

    @JvmStatic external fun learnedPairCapacity(): Int

    @JvmStatic external fun maxWordLength(): Int

    @JvmStatic external fun keptCapacity(): Int

    @JvmStatic external fun isStorable(word: String): Boolean

    @JvmStatic external fun previousKey(firstWord: String?, isBeginningOfSentence: Boolean): String?

    @JvmStatic external fun appliesTo(tag: String?, language: String): Boolean

    @JvmStatic external fun newLearnedWords(wordCapacity: Int, pairCapacity: Int): Long

    @JvmStatic external fun freeLearnedWords(store: Long)

    @JvmStatic external fun learnedAdd(store: Long, word: String, previous: String?, keepSpelling: Boolean)

    @JvmStatic external fun learnedRemove(store: Long, word: String, previous: String?)

    @JvmStatic external fun learnedClear(store: Long)

    @JvmStatic external fun learnedKnownSpellings(store: Long): Array<String>

    @JvmStatic external fun learnedSpellingOf(store: Long, word: String): String?

    @JvmStatic external fun learnedSwipeBoosts(store: Long, words: Array<String>): IntArray

    @JvmStatic external fun learnedCompletions(store: Long, typed: String, maxCount: Int): Array<String>

    @JvmStatic external fun learnedFollowers(
        store: Long,
        firstWord: String?,
        isBeginningOfSentence: Boolean,
        minCount: Int,
        maxCount: Int,
    ): Array<String>

    @JvmStatic external fun learnedPairCount(store: Long, firstWord: String?, isBeginningOfSentence: Boolean, word: String): Int

    @JvmStatic external fun learnedMerge(store: Long, text: String)

    @JvmStatic external fun learnedSavedText(store: Long): String

    @JvmStatic external fun newKeptWords(capacity: Int): Long

    @JvmStatic external fun freeKeptWords(store: Long)

    @JvmStatic external fun keptContains(store: Long, word: String): Boolean

    @JvmStatic external fun keptAdd(store: Long, word: String, remember: Boolean): Boolean

    @JvmStatic external fun keptClear(store: Long)

    @JvmStatic external fun keptMerge(store: Long, text: String)

    @JvmStatic external fun keptSavedText(store: Long): String

    @JvmStatic external fun newPersonalWords(words: Array<String>, frequencies: IntArray, shortcuts: Array<String?>): Long

    @JvmStatic external fun freePersonalWords(words: Long)

    @JvmStatic external fun personalOrder(words: Long): IntArray

    @JvmStatic external fun personalSpellingOf(words: Long, word: String): String?

    @JvmStatic external fun personalCompletions(words: Long, typed: String, maxCount: Int): Array<String>
}
