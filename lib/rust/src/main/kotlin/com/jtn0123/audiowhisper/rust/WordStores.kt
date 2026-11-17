package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * The rules and numbers of the words the keyboard keeps for the user, in Rust
 * (lib/rust/crates/suggest): what it learns from typing, the words kept as typed, and the
 * personal dictionaries. The keyboard's `suggest` package wraps these and documents them.
 */
object WordStoreRules {
    /** The previous "word" of a sentence's first word. */
    val sentenceStart: String = WordStoresNative.sentenceStart()

    /** A word typed this many times is one of the user's words. */
    val knownAfter: Int = WordStoresNative.knownAfter()

    /** How many words a [LearnedWordStore] keeps unless told otherwise. */
    val learnedWordCapacity: Int = WordStoresNative.learnedWordCapacity()

    /** How many word pairs a [LearnedWordStore] keeps unless told otherwise. */
    val learnedPairCapacity: Int = WordStoresNative.learnedPairCapacity()

    /** The longest word a [LearnedWordStore] keeps, in UTF-16 units. */
    val maxWordLength: Int = WordStoresNative.maxWordLength()

    /** How many words a [KeptWordStore] remembers unless told otherwise. */
    val keptCapacity: Int = WordStoresNative.keptCapacity()

    /** Whether [word] can be learned: letters, with apostrophes or hyphens inside. */
    fun isStorable(word: String): Boolean = WordStoresNative.isStorable(word)

    /**
     * The word before the next one: [firstWord], the most recent word before it, else
     * [sentenceStart] if [isBeginningOfSentence], else null, as what came before is unknown.
     */
    fun previousKey(firstWord: String?, isBeginningOfSentence: Boolean): String? =
        WordStoresNative.previousKey(firstWord, isBeginningOfSentence)

    /** Whether a personal dictionary row tagged [tag] (null for every language) applies to [language]. */
    fun appliesTo(tag: String?, language: String): Boolean = WordStoresNative.appliesTo(tag, language)
}

/**
 * The words and word pairs the user has typed, with how often, in Rust. Safe to use from any
 * thread. Its native memory is freed once nothing can reach it. The previous word of a pair is
 * passed as the context's most recent word and whether a sentence starts there (see
 * [WordStoreRules.previousKey]).
 */
class LearnedWordStore(
    wordCapacity: Int = WordStoreRules.learnedWordCapacity,
    pairCapacity: Int = WordStoreRules.learnedPairCapacity,
) {
    internal val handle = WordStoresNative.newLearnedWords(wordCapacity, pairCapacity)

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** One use of [word] after [previous]: a word, [WordStoreRules.sentenceStart], or null when unknown. */
    fun add(word: String, previous: String?, keepSpelling: Boolean) {
        WordStoresNative.learnedAdd(handle, word, previous, keepSpelling)
        Reference.reachabilityFence(this)
    }

    /** Takes back one [add] of [word] after [previous]. */
    fun remove(word: String, previous: String?) {
        WordStoresNative.learnedRemove(handle, word, previous)
        Reference.reachabilityFence(this)
    }

    fun clear() {
        WordStoresNative.learnedClear(handle)
        Reference.reachabilityFence(this)
    }

    /** How the user spells each of their words, least recently used first. */
    fun knownSpellings(): List<String> =
        WordStoresNative.learnedKnownSpellings(handle).asList().also { Reference.reachabilityFence(this) }

    /**
     * How many steps swipe typing raises each of [words] for the user's use of it
     * ([SwipeRules.tunedFrequency]): more each time its uses double, up to a limit, and none for a
     * word that isn't theirs as spelt. Cleared with the words.
     */
    fun swipeBoosts(words: List<String>): IntArray =
        WordStoresNative.learnedSwipeBoosts(handle, words.toTypedArray()).also { Reference.reachabilityFence(this) }

    /** How the user spells [word], once it is one of theirs, or null. */
    fun spellingOf(word: String): String? =
        WordStoresNative.learnedSpellingOf(handle, word).also { Reference.reachabilityFence(this) }

    /**
     * Every completion of [typed], best first: none if [typed] is empty or [maxCount] isn't
     * positive. The caller keeps the first [maxCount] it wants.
     */
    fun completions(typed: String, maxCount: Int): List<String> =
        WordStoresNative.learnedCompletions(handle, typed, maxCount).asList().also { Reference.reachabilityFence(this) }

    /** Up to [maxCount] words typed after the previous word at least [minCount] times, best first. */
    fun followers(firstWord: String?, isBeginningOfSentence: Boolean, minCount: Int, maxCount: Int): List<String> =
        WordStoresNative.learnedFollowers(handle, firstWord, isBeginningOfSentence, minCount, maxCount).asList()
            .also { Reference.reachabilityFence(this) }

    /** How many times [word] was typed after the previous word. */
    fun pairCount(firstWord: String?, isBeginningOfSentence: Boolean, word: String): Int =
        WordStoresNative.learnedPairCount(handle, firstWord, isBeginningOfSentence, word)
            .also { Reference.reachabilityFence(this) }

    /**
     * How many times the words have changed (learned, taken back, cleared or loaded). Equal
     * counts mean nothing changed in between; only equality means anything.
     */
    fun changes(): Long = WordStoresNative.learnedChanges(handle).also { Reference.reachabilityFence(this) }

    /** Takes in a saved file's [text], keeping what was learned before it was read. */
    fun merge(text: String) {
        WordStoresNative.learnedMerge(handle, text)
        Reference.reachabilityFence(this)
    }

    /** The text to save, for [merge] to read back. */
    fun savedText(): String = WordStoresNative.learnedSavedText(handle).also { Reference.reachabilityFence(this) }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = WordStoresNative.freeLearnedWords(handle)
    }
}

/**
 * The words whose auto-correction the user undid, in Rust. Safe to use from any thread. Its
 * native memory is freed once nothing can reach it.
 */
class KeptWordStore(capacity: Int = WordStoreRules.keptCapacity) {
    private val handle = WordStoresNative.newKeptWords(capacity)

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** Whether [word] is kept, in any case. */
    operator fun contains(word: String): Boolean =
        WordStoresNative.keptContains(handle, word).also { Reference.reachabilityFence(this) }

    /** Keeps [word], saving it if [remember]; true when [savedText] changed. */
    fun add(word: String, remember: Boolean): Boolean =
        WordStoresNative.keptAdd(handle, word, remember).also { Reference.reachabilityFence(this) }

    /** Forgets every kept word, remembered or not. */
    fun clear() {
        WordStoresNative.keptClear(handle)
        Reference.reachabilityFence(this)
    }

    /** Takes in a saved file's [text], ahead of the words remembered before it was read. */
    fun merge(text: String) {
        WordStoresNative.keptMerge(handle, text)
        Reference.reachabilityFence(this)
    }

    /** The text to save, for [merge] to read back. */
    fun savedText(): String = WordStoresNative.keptSavedText(handle).also { Reference.reachabilityFence(this) }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = WordStoresNative.freeKeptWords(handle)
    }
}

/**
 * A snapshot of the personal dictionaries' rows, in Rust: row i is [words] i, [frequencies] i and
 * [shortcuts] i. Never changes, so safe to use from any thread. Its native memory is freed once
 * nothing can reach it.
 */
class PersonalWordStore(words: List<String>, frequencies: IntArray, shortcuts: List<String?>) {
    internal val handle = WordStoresNative.newPersonalWords(words.toTypedArray(), frequencies, shortcuts.toTypedArray())

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** The rows kept, most frequent first, by their index in the lists this was made from. */
    val order: IntArray = WordStoresNative.personalOrder(handle)

    /** How the user spells [word]: [word] if they wrote it so, else their spelling ignoring case, else null. */
    fun spellingOf(word: String): String? =
        WordStoresNative.personalSpellingOf(handle, word).also { Reference.reachabilityFence(this) }

    /** At most [maxCount] words for [typed], best first. */
    fun completions(typed: String, maxCount: Int): List<String> =
        WordStoresNative.personalCompletions(handle, typed, maxCount).asList().also { Reference.reachabilityFence(this) }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = WordStoresNative.freePersonalWords(handle)
    }
}
