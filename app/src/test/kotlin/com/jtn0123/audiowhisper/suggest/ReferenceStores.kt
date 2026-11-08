package com.jtn0123.audiowhisper.suggest

import java.io.File
import java.io.IOException
import java.util.Locale

/**
 * The word stores as they were in Kotlin before moving to Rust (lib/rust/crates/suggest), copied
 * unchanged from commit dd0543de: LearnedWords.kt, KeptWords.kt and PersonalWords.kt, with the
 * TypingHistory.kt whose number they count by. They are what [WordStoresMatchTest] holds the Rust
 * to; don't change them. [PreviousWords] is the app's.
 */
object ReferenceStores {
    /** The words and word pairs the user has typed, as suggestions read them. Kept by [LearnedWords]. */
    interface TypingHistory {
        /** How the user spells [word] (ignoring case), once they have used it [KNOWN_AFTER] times. */
        fun spellingOf(word: String): String?

        /**
         * Up to [maxCount] words used [KNOWN_AFTER] times that start with [typed] (ignoring case) and
         * are longer, most used first, keeping only those [accept] lets through.
         */
        fun completions(typed: String, maxCount: Int, accept: (String) -> Boolean = { true }): List<String>

        /** Up to [maxCount] words typed right after [previous] at least [minCount] times, most used first. */
        fun followers(previous: PreviousWords, minCount: Int, maxCount: Int): List<String>

        /** How many times [word] was typed right after [previous]. */
        fun pairCount(previous: PreviousWords, word: String): Int

        companion object {
            /** A word typed this many times is one of the user's words. */
            const val KNOWN_AFTER = 2

            val NONE: TypingHistory = object : TypingHistory {
                override fun spellingOf(word: String): String? = null
                override fun completions(typed: String, maxCount: Int, accept: (String) -> Boolean) = emptyList<String>()
                override fun followers(previous: PreviousWords, minCount: Int, maxCount: Int) = emptyList<String>()
                override fun pairCount(previous: PreviousWords, word: String) = 0
            }
        }
    }

    /**
     * What the user has typed: how often each word was used and which word came before it. The
     * bundled dictionary knows at most three next words for any word and none of the user's names
     * or slang, so this is where the keyboard picks up their habits. Kept in [file] on the phone
     * only, at most [wordCapacity] words and [pairCapacity] pairs; the least recently used go first.
     *
     * Thread-safe: words are learned on the main thread and read while suggesting. Never logged.
     */
    class LearnedWords(
        private val file: File?,
        private val wordCapacity: Int = WORD_CAPACITY,
        private val pairCapacity: Int = PAIR_CAPACITY,
    ) : TypingHistory {
        private class Entry(var spelling: String, var count: Int)

        /** Lower-cased word -> its spelling and use count, least recently used first. */
        private val words = LinkedHashMap<String, Entry>()

        /** "previous\tword", lower-cased -> use count, least recently used first. */
        private val pairs = LinkedHashMap<String, Int>()

        /** Two saves at once would write the same temporary file. */
        private val saveLock = Any()

        /**
         * Counts one use of [word] after [previous]: a word, [SENTENCE_START], or null when what came
         * before is unknown. With [keepSpelling] an earlier spelling stands (a capital that may only
         * be there because the word starts a sentence).
         */
        @Synchronized
        fun add(word: String, previous: String?, keepSpelling: Boolean = false) {
            if (!isStorable(word)) return
            val key = key(word)
            val entry = words.remove(key)?.also {
                it.count++
                if (!keepSpelling) it.spelling = word
            } ?: Entry(word, 1)
            words[key] = entry
            trim(words, wordCapacity)
            if (previous != null && (previous == SENTENCE_START || isStorable(previous))) {
                val pair = pairKey(previous, word)
                pairs[pair] = (pairs.remove(pair) ?: 0) + 1
                trim(pairs, pairCapacity)
            }
        }

        /** Takes back one [add] of [word] after [previous]. */
        @Synchronized
        fun remove(word: String, previous: String?) {
            val key = key(word)
            words[key]?.let { if (--it.count <= 0) words.remove(key) }
            if (previous == null) return
            val pair = pairKey(previous, word)
            val count = pairs[pair] ?: return
            if (count <= 1) pairs.remove(pair) else pairs[pair] = count - 1
        }

        @Synchronized
        fun clear() {
            words.clear()
            pairs.clear()
        }

        /** How the user spells each word they have used [TypingHistory.KNOWN_AFTER] times. */
        @Synchronized
        fun knownSpellings(): List<String> =
            words.values.filter { it.count >= TypingHistory.KNOWN_AFTER }.map { it.spelling }

        @Synchronized
        override fun spellingOf(word: String): String? =
            words[key(word)]?.takeIf { it.count >= TypingHistory.KNOWN_AFTER }?.spelling

        override fun completions(typed: String, maxCount: Int, accept: (String) -> Boolean): List<String> {
            if (typed.isEmpty() || maxCount <= 0) return emptyList()
            val prefix = key(typed)
            val matching = synchronized(this) {
                words.entries.filter { (key, entry) ->
                    entry.count >= TypingHistory.KNOWN_AFTER && key.length > prefix.length && key.startsWith(prefix)
                }.map { it.value.spelling to it.value.count }
            }
            // Most recent first, so a stable sort by count breaks ties in their favour.
            return matching.asReversed().sortedByDescending { it.second }.map { it.first }.filter(accept).take(maxCount)
        }

        @Synchronized
        override fun followers(previous: PreviousWords, minCount: Int, maxCount: Int): List<String> {
            val prefix = previousKey(previous)?.let { "${key(it)}\t" } ?: return emptyList()
            return pairs.entries.filter { it.value >= minCount && it.key.startsWith(prefix) }
                .asReversed()
                .sortedByDescending { it.value }
                .take(maxCount)
                .map { pair -> pair.key.substring(prefix.length).let { words[it]?.spelling ?: it } }
        }

        @Synchronized
        override fun pairCount(previous: PreviousWords, word: String): Int {
            val previousWord = previousKey(previous) ?: return 0
            return pairs[pairKey(previousWord, word)] ?: 0
        }

        /** Reads the saved words, merging in any learned before it finished. Blocking. */
        fun load() {
            val savedWords = LinkedHashMap<String, Entry>()
            val savedPairs = LinkedHashMap<String, Int>()
            readLines().forEach { parse(it, savedWords, savedPairs) }
            synchronized(this) {
                for ((key, entry) in words) {
                    savedWords.remove(key)?.let { entry.count += it.count }
                    savedWords[key] = entry
                }
                for ((key, count) in pairs) savedPairs[key] = (savedPairs.remove(key) ?: 0) + count
                words.clear()
                words.putAll(savedWords)
                pairs.clear()
                pairs.putAll(savedPairs)
                trim(words, wordCapacity)
                trim(pairs, pairCapacity)
            }
        }

        /** Writes everything learned, replacing the file only once the new one is complete. Blocking. */
        fun save(): Boolean = synchronized(saveLock) {
            val file = file ?: return false
            val text = synchronized(this) {
                buildString {
                    append(HEADER).append('\n')
                    for (entry in words.values) append("w\t${entry.count}\t${entry.spelling}\n")
                    for ((pair, count) in pairs) append("p\t$count\t$pair\n")
                }
            }
            val partial = File(file.path + ".tmp")
            try {
                partial.writeText(text, Charsets.UTF_8)
                partial.renameTo(file)
            } catch (_: IOException) {
                false
            }
        }

        private fun readLines(): List<String> = try {
            file?.takeIf { it.isFile }?.readLines(Charsets.UTF_8).orEmpty()
        } catch (_: IOException) {
            emptyList()
        }

        /** One saved line: "w, count, spelling" or "p, count, previous, word", tab-separated. */
        private fun parse(line: String, savedWords: MutableMap<String, Entry>, savedPairs: MutableMap<String, Int>) {
            val fields = line.split('\t')
            val count = fields.getOrNull(1)?.toIntOrNull()?.takeIf { it > 0 } ?: return
            when {
                fields[0] == "w" && fields.size == 3 && isStorable(fields[2]) ->
                    savedWords[key(fields[2])] = Entry(fields[2], count)
                fields[0] == "p" && fields.size == 4 && isStorable(fields[3]) &&
                    (fields[2] == SENTENCE_START || isStorable(fields[2])) ->
                    savedPairs[pairKey(fields[2], fields[3])] = count
            }
        }

        private fun <V> trim(map: LinkedHashMap<String, V>, capacity: Int) {
            val iterator = map.entries.iterator()
            while (map.size > capacity && iterator.hasNext()) {
                iterator.next()
                iterator.remove()
            }
        }

        companion object {
            /** The previous "word" of the first word in a sentence. */
            const val SENTENCE_START = ""

            /** Enough for everyday vocabulary; the oldest go first. */
            const val WORD_CAPACITY = 5000
            const val PAIR_CAPACITY = 20_000

            /** Longer than any real word; longer runs of letters are not worth keeping. */
            const val MAX_WORD_LENGTH = 48

            private const val HEADER = "# AudioWhisper learned words, v1"
            private val CONNECTORS = setOf('\'', '’', '-')

            /** Letters, with apostrophes or hyphens inside: no digits, spaces, tabs or symbols. */
            fun isStorable(word: String): Boolean =
                word.length in 1..MAX_WORD_LENGTH && word.any(Char::isLetter) &&
                    word.all { it.isLetter() || it in CONNECTORS }

            /** The word before the next one in [previous], or [SENTENCE_START], or null when unknown. */
            fun previousKey(previous: PreviousWords): String? =
                previous.words.firstOrNull() ?: SENTENCE_START.takeIf { previous.isBeginningOfSentence }

            private fun key(word: String) = word.lowercase(Locale.ROOT)

            private fun pairKey(previous: String, word: String) = "${key(previous)}\t${key(word)}"
        }
    }

    /**
     * Words whose auto-correction the user undid, so they are not corrected again. Kept in [file],
     * one lower-cased word per line, newest last, at most [capacity] of them. A word kept with
     * remember = false (incognito) lasts only as long as this object and is never written.
     *
     * Thread-safe: words are added on the main thread and read while suggesting.
     */
    class KeptWords(private val file: File?, private val capacity: Int = CAPACITY) {
        private val remembered = LinkedHashSet<String>()
        private val sessionOnly = HashSet<String>()

        /** Two saves at once would write the same temporary file. */
        private val saveLock = Any()

        @Synchronized
        operator fun contains(word: String): Boolean = key(word).let { it in remembered || it in sessionOnly }

        /** Keeps [word]; true when the change needs [save]. */
        @Synchronized
        fun add(word: String, remember: Boolean): Boolean {
            val key = key(word)
            if (key.isBlank() || '\n' in key) return false
            if (!remember) {
                sessionOnly.add(key)
                return false
            }
            sessionOnly.remove(key)
            remembered.remove(key)
            remembered.add(key)
            trim()
            return true
        }

        /** Reads the saved words, ahead of any added before it finished. Blocking. */
        fun load() {
            val saved = try {
                file?.takeIf { it.isFile }?.readLines(Charsets.UTF_8).orEmpty()
            } catch (_: IOException) {
                emptyList()
            }
            synchronized(this) {
                val added = remembered.toList()
                remembered.clear()
                saved.map(String::trim).filter(String::isNotEmpty).forEach { remembered.add(key(it)) }
                remembered.removeAll(added.toSet())
                remembered.addAll(added)
                trim()
            }
        }

        /** Writes the remembered words, replacing the file only once the new one is complete. Blocking. */
        fun save(): Boolean = synchronized(saveLock) {
            val file = file ?: return false
            val lines = synchronized(this) { remembered.joinToString("\n", postfix = "\n") }
            val partial = File(file.path + ".tmp")
            try {
                partial.writeText(lines, Charsets.UTF_8)
                partial.renameTo(file)
            } catch (_: IOException) {
                false
            }
        }

        private fun trim() {
            val iterator = remembered.iterator()
            while (remembered.size > capacity && iterator.hasNext()) {
                iterator.next()
                iterator.remove()
            }
        }

        private fun key(word: String) = word.lowercase(Locale.ROOT)

        companion object {
            /** Enough for years of undos; the oldest go first. */
            const val CAPACITY = 1000
        }
    }

    /**
     * The words in the user's personal dictionaries (Floris's own and the system's): never
     * auto-corrected away, and offered as completions. An immutable snapshot, small enough to scan.
     */
    class PersonalWords(entries: List<Entry>, private val locale: Locale = Locale.US) {
        /** A personal dictionary row: [frequency] is 0-255, [shortcut] types [word] when tapped. */
        data class Entry(val word: String, val frequency: Int, val shortcut: String? = null)

        private val byFrequency = entries.filter { it.word.isNotBlank() }.sortedByDescending { it.frequency }
        private val spellings: Set<String> = byFrequency.mapTo(HashSet()) { it.word }
        private val byLowerCase: Map<String, String> = byFrequency.groupBy { it.word.lowercase(locale) }
            .mapValues { (_, same) -> same.first().word }

        val isEmpty get() = byFrequency.isEmpty()

        /** Every entry with a word, most frequent first. */
        val entries: List<Entry> get() = byFrequency

        /** How the user spells [word]: [word] itself if they wrote it that way, else their spelling ignoring case, else null. */
        fun spellingOf(word: String): String? =
            if (word in spellings) word else byLowerCase[word.lowercase(locale)]

        /**
         * At most [maxCount] words for [typed], most frequent first: those whose shortcut it is, then
         * those that start with it (ignoring case) and are longer. [typed] itself is never one.
         */
        fun completions(typed: String, maxCount: Int): List<String> {
            if (typed.isEmpty() || maxCount <= 0) return emptyList()
            val expansions = byFrequency.asSequence().filter { it.shortcut?.equals(typed, ignoreCase = true) == true }
            val longer = byFrequency.asSequence()
                .filter { it.word.length > typed.length && it.word.startsWith(typed, ignoreCase = true) }
            return (expansions + longer).map { it.word }.filter { it != typed }.distinct().take(maxCount).toList()
        }

        companion object {
            val NONE = PersonalWords(emptyList())

            /** How many personal words the strip shows at most, so dictionary words still get room. */
            const val MAX_IN_STRIP = 2

            /** Whether a row tagged [tag] ("en", "en_US", null for every language) applies to [language]. */
            fun appliesTo(tag: String?, language: String): Boolean =
                tag.isNullOrEmpty() || tag.substringBefore('_').substringBefore('-').equals(language, ignoreCase = true)
        }
    }
}
