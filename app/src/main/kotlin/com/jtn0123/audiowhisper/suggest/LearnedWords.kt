package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.LearnedWordStore
import com.jtn0123.audiowhisper.rust.WordStoreRules
import java.io.File
import java.io.IOException

/**
 * What the user has typed: how often each word was used and which word came before it. The
 * bundled dictionary knows at most three next words for any word and none of the user's names
 * or slang, so this is where the keyboard picks up their habits. Kept in [file] on the phone
 * only, at most [wordCapacity] words and [pairCapacity] pairs; the least recently used go first.
 * The words live in Rust ([LearnedWordStore], lib/rust/crates/suggest/src/learned_words.rs); this
 * reads and writes the file.
 *
 * Thread-safe: words are learned on the main thread and read while suggesting. Never logged.
 */
class LearnedWords(
    private val file: File?,
    wordCapacity: Int = WORD_CAPACITY,
    pairCapacity: Int = PAIR_CAPACITY,
) : TypingHistory {
    internal val store = LearnedWordStore(wordCapacity, pairCapacity)

    /** Two saves at once would write the same temporary file. */
    private val saveLock = Any()

    /**
     * Counts one use of [word] after [previous]: a word, [SENTENCE_START], or null when what came
     * before is unknown. With [keepSpelling] an earlier spelling stands (a capital that may only
     * be there because the word starts a sentence).
     */
    fun add(word: String, previous: String?, keepSpelling: Boolean = false) = store.add(word, previous, keepSpelling)

    /** Takes back one [add] of [word] after [previous]. */
    fun remove(word: String, previous: String?) = store.remove(word, previous)

    fun clear() = store.clear()

    /** Goes up with every change to the words, however made (see [LearnedWordStore.changes]). */
    fun changes(): Long = store.changes()

    /** How the user spells each word they have used [TypingHistory.KNOWN_AFTER] times. */
    fun knownSpellings(): List<String> = store.knownSpellings()

    /** How many steps swipe typing raises each of [words] for the user's use of it (see SwipeWords). */
    fun swipeBoosts(words: List<String>): IntArray = store.swipeBoosts(words)

    override fun spellingOf(word: String): String? = store.spellingOf(word)

    override fun completions(typed: String, maxCount: Int, accept: (String) -> Boolean): List<String> {
        // Rust ranks them all; which it wants is the caller's to say.
        val ranked = store.completions(typed, maxCount)
        return if (ranked.isEmpty()) ranked else ranked.asSequence().filter(accept).take(maxCount).toList()
    }

    override fun followers(previous: PreviousWords, minCount: Int, maxCount: Int): List<String> =
        store.followers(previous.words.firstOrNull(), previous.isBeginningOfSentence, minCount, maxCount)

    override fun pairCount(previous: PreviousWords, word: String): Int =
        store.pairCount(previous.words.firstOrNull(), previous.isBeginningOfSentence, word)

    /** Reads the saved words, merging in any learned before it finished. Blocking. */
    fun load() = store.merge(readText())

    /** Writes everything learned, replacing the file only once the new one is complete. Blocking. */
    fun save(): Boolean = synchronized(saveLock) {
        val file = file ?: return false
        val text = store.savedText()
        val partial = File(file.path + ".tmp")
        try {
            partial.writeText(text, Charsets.UTF_8)
            partial.renameTo(file)
        } catch (_: IOException) {
            false
        }
    }

    private fun readText(): String = try {
        file?.takeIf { it.isFile }?.readText(Charsets.UTF_8).orEmpty()
    } catch (_: IOException) {
        ""
    }

    companion object {
        /** The previous "word" of the first word in a sentence. */
        val SENTENCE_START: String = WordStoreRules.sentenceStart

        /** Enough for everyday vocabulary; the oldest go first. */
        val WORD_CAPACITY: Int = WordStoreRules.learnedWordCapacity
        val PAIR_CAPACITY: Int = WordStoreRules.learnedPairCapacity

        /** Longer than any real word; longer runs of letters are not worth keeping. */
        val MAX_WORD_LENGTH: Int = WordStoreRules.maxWordLength

        /** Letters, with apostrophes or hyphens inside: no digits, spaces, tabs or symbols. */
        fun isStorable(word: String): Boolean = WordStoreRules.isStorable(word)

        /** The word before the next one in [previous], or [SENTENCE_START], or null when unknown. */
        fun previousKey(previous: PreviousWords): String? =
            WordStoreRules.previousKey(previous.words.firstOrNull(), previous.isBeginningOfSentence)
    }
}
