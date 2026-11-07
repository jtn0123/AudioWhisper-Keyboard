package com.jtn0123.audiowhisper.suggest

import java.util.Locale

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
