package dev.patrickgold.florisboard.ime.dictionary

import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.reference.ReferenceFlorisLocale
import dev.patrickgold.florisboard.lib.reference.ReferenceUtilityRules
import java.io.Reader
import java.io.Writer

/**
 * What UserDictionaryScreen and UserDictionary did before the native screens replaced them, frozen as they were at
 * f5a6fc180: `importCombinedList` and `exportCombinedList` over a reader and writer in place of a document, and the
 * add and edit dialog's checks and the entry its `onConfirm` built. Only what touched Android is swapped: the
 * document for a reader or writer, `uri.lastPathSegment`, `context.packageName` and `System.currentTimeMillis()`
 * for parameters. The DAO calls are unchanged, so the rows they write are the old rows.
 */
object ReferenceUserDictionary {
    const val FREQUENCY_MIN = 1
    const val FREQUENCY_MAX = 255

    fun importCombinedList(dao: UserDictionaryDao, src: Reader) {
        var isFirstLine = true
        src.forEachLine { line ->
            if (isFirstLine) {
                // Ignore
                isFirstLine = false
            } else {
                var word: String? = null
                var freq: Int? = null
                var locale: String? = null
                var shortcut: String? = null
                for (property in line.split(';')) {
                    val keyValuePair = property.split('=')
                    check(keyValuePair.size == 2) { "Error at source line: Key-Value pair expected, but either only key or too many values provided" }
                    val key = keyValuePair[0].trim().lowercase()
                    val value = keyValuePair[1].trim()
                    when (key) {
                        "w", "word" -> word = value.ifBlank { null }
                        "f", "freq" -> {
                            val number = value.toIntOrNull(10)
                            checkNotNull(number) { "Error at source line: Freq is not a valid decimal number" }
                            check(number in FREQUENCY_MIN..FREQUENCY_MAX) {
                                "Error at source line: Freq not within range of $FREQUENCY_MIN and $FREQUENCY_MAX"
                            }
                            freq = number
                        }
                        "l", "locale" -> locale = when (value) {
                            "all", "null", "" -> null
                            else -> value.ifBlank { null }
                        }
                        "s", "shortcut" -> shortcut = value.ifBlank { null }
                    }
                }
                checkNotNull(word) { "Error at source line: Word cannot be empty or missing" }
                checkNotNull(freq) { "Error at source line: Freq cannot be empty or missing" }
                val alreadyExistingEntries = dao.queryExact(
                    word, locale?.let { FlorisLocale.fromTag(it) },
                )
                if (alreadyExistingEntries.isNotEmpty()) {
                    dao.update(UserDictionaryEntry(alreadyExistingEntries[0].id, word, freq, locale, shortcut))
                } else {
                    dao.insert(UserDictionaryEntry(0, word, freq, locale, shortcut))
                }
            }
        }
    }

    fun exportCombinedList(dao: UserDictionaryDao, dst: Writer, lastPathSegment: String?, packageName: String, now: Long) {
        StringBuilder().apply {
            append("dictionary=")
            append(lastPathSegment)
            append(";date=")
            append(now)
            append(";generated-by=")
            append(packageName)
            append(";version=1")
            appendLine()
            dst.write(toString())
        }
        for (entry in dao.queryAll()) {
            StringBuilder().apply {
                append(" w=")
                append(entry.word)
                append(";f=")
                append(entry.freq)
                append(";l=")
                append(entry.locale) // always append locale even if null
                if (entry.shortcut != null) {
                    append(";s=")
                    append(entry.shortcut)
                }
                appendLine()
                dst.write(toString())
            }
        }
    }

    /** Whether the dialog's `onConfirm` found the draft invalid: UserDictionaryValidation's four rules. */
    fun isInvalid(word: String, freq: String, shortcut: String, locale: String): Boolean {
        fun rule(name: String, input: String) = ReferenceUtilityRules.validate(name, input, FREQUENCY_MIN, FREQUENCY_MAX)
        val localeInvalid = !locale.isBlank() && runCatching { ReferenceFlorisLocale.fromTag(locale.trim()) }.getOrNull() == null
        return rule("word", word) != "" || rule("freq", freq) != "" || rule("shortcut", shortcut) == "invalid" || localeInvalid
    }

    /** The entry `onConfirm` built from a valid draft; `freq.toInt(10)` threw on a frequency with spaces around it. */
    fun entry(id: Long, word: String, freq: String, shortcut: String, locale: String) = UserDictionaryEntry(
        id = id,
        word = word.trim(),
        freq = freq.toInt(10),
        shortcut = shortcut.trim().takeIf { it.isNotBlank() },
        locale = locale.trim().takeIf { it.isNotBlank() }?.let {
            ReferenceFlorisLocale.fromTag(it).localeTag()
        },
    )
}

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
