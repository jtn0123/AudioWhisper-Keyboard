package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.PersonalWordStore
import com.jtn0123.audiowhisper.rust.WordStoreRules

/**
 * The words in the user's personal dictionaries (Floris's own and the system's): never
 * auto-corrected away, and offered as completions. An immutable snapshot, small enough to scan,
 * kept in Rust ([PersonalWordStore], lib/rust/crates/suggest/src/personal_words.rs). Case is
 * ignored as Locale.US ignores it.
 */
class PersonalWords(entries: List<Entry>) {
    /** A personal dictionary row: [frequency] is 0-255, [shortcut] types [word] when tapped. */
    data class Entry(val word: String, val frequency: Int, val shortcut: String? = null)

    internal val store = PersonalWordStore(
        entries.map { it.word },
        entries.map { it.frequency }.toIntArray(),
        entries.map { it.shortcut },
    )

    /** Every entry with a word, most frequent first. */
    val entries: List<Entry> = store.order.map { entries[it] }

    val isEmpty get() = entries.isEmpty()

    /** How the user spells [word]: [word] itself if they wrote it that way, else their spelling ignoring case, else null. */
    fun spellingOf(word: String): String? = store.spellingOf(word)

    /**
     * At most [maxCount] words for [typed], most frequent first: those whose shortcut it is, then
     * those that start with it (ignoring case) and are longer. [typed] itself is never one.
     */
    fun completions(typed: String, maxCount: Int): List<String> = store.completions(typed, maxCount)

    companion object {
        val NONE = PersonalWords(emptyList())

        /** Whether a row tagged [tag] ("en", "en_US", null for every language) applies to [language]. */
        fun appliesTo(tag: String?, language: String): Boolean = WordStoreRules.appliesTo(tag, language)
    }
}
