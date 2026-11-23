package dev.patrickgold.florisboard.ime.dictionary

import dev.patrickgold.florisboard.lib.FlorisLocale

/**
 * A user dictionary in memory that records each change, as the Floris dictionary's Room table behaves or, with
 * [system], as Android's provider does: Room keeps an inserted id and hands out new ones past any used, the
 * provider ignores the id; `queryAll()` is Room's row order or the provider's frequency order.
 */
class RecordingUserDictionaryDao(rows: List<UserDictionaryEntry> = emptyList(), private val system: Boolean = false) : UserDictionaryDao {
    val rows = rows.toMutableList()
    val calls = mutableListOf<Pair<String, UserDictionaryEntry>>()
    private var nextId = (rows.maxOfOrNull { it.id } ?: 0) + 1
    private fun tag(locale: FlorisLocale?) = locale?.localeTag()

    override fun queryAll() = if (system) rows.sortedByDescending { it.freq } else rows.sortedBy { it.id }
    override fun queryExact(word: String, locale: FlorisLocale?) = queryAll().filter { it.word == word && it.locale == tag(locale) }
    override fun insert(entry: UserDictionaryEntry) {
        calls += "insert" to entry
        val id = if (system || entry.id == 0L) nextId else entry.id
        check(rows.none { it.id == id }) { "UNIQUE constraint failed" }
        rows += entry.copy(id = id)
        nextId = maxOf(nextId, id + 1)
    }
    override fun update(entry: UserDictionaryEntry) {
        calls += "update" to entry
        rows.replaceAll { if (it.id == entry.id) entry else it }
    }
    override fun delete(entry: UserDictionaryEntry) {
        calls += "delete" to entry
        rows.removeAll { it.id == entry.id }
    }
}
