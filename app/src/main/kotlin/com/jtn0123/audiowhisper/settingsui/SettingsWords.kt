package com.jtn0123.audiowhisper.settingsui

import android.content.Context
import android.net.Uri
import androidx.activity.ComponentActivity
import androidx.lifecycle.lifecycleScope
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.settings.advanced.BackupFiles
import dev.patrickgold.florisboard.ime.dictionary.DictionaryManager
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryDao
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryEntry
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.util.launchActivity
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.florisboard.lib.android.readText
import org.florisboard.lib.android.writeText

/** The system's own user dictionary settings, which UserDictionaryScreen's overflow menu opened. */
internal const val SYSTEM_DICTIONARY_SETTINGS = "android.settings.USER_DICTIONARY_SETTINGS"

/** What the native user dictionary screens need from the two dictionaries: one's words as a `WORDS` report, and
 * each change Rust asks for, made as UserDictionaryScreen made it, the Floris dictionary in Room and the system's
 * through its provider. Every effect is answered with a fresh report. No word, shortcut or locale is ever logged. */
internal class SettingsWords(private val activity: ComponentActivity, private val report: (Array<String>) -> Unit) {
    /** One change at a time and in order: a delete and its undo reach the database as they were asked. */
    private val serial = Dispatchers.IO.limitedParallelism(1)
    /** A `SettingsUiWordEffect`. */
    fun execute(p: List<String>) {
        if (p[0] == SettingsUiWordEffect.MANAGER) activity.launchActivity { it.action = SYSTEM_DICTIONARY_SETTINGS }
        activity.lifecycleScope.launch { report(withContext(serial) { answer(p) }) }
    }
    private fun answer(p: List<String>): Array<String> {
        val manager = DictionaryManager.default()
        manager.loadUserDictionariesIfNecessary()
        val floris = p[1] == SettingsUiDictionary.FLORIS
        val dao = if (floris) manager.florisUserDictionaryDao() else manager.systemUserDictionaryDao()
        val database = manager.florisUserDictionaryDatabase().takeIf { floris }
        // An import is all or nothing in the Floris dictionary; the system's provider has no transaction.
        return wordAnswer(p, dao) { block -> if (database != null) database.runInTransaction { block() } else block() }
    }
}

/** The report answering effect [p] on [dao], which is null for a dictionary that is off: [p] carried out, and
 * whether the database refused it. */
internal fun wordAnswer(p: List<String>, dao: UserDictionaryDao?, transaction: (() -> Unit) -> Unit = { it() }): Array<String> {
    val outcome = when {
        p[0] == SettingsUiWordEffect.LOAD || p[0] == SettingsUiWordEffect.MANAGER -> SettingsUiWordOutcome.NONE
        dao == null -> SettingsUiWordOutcome.FAILED
        else -> runCatching { applyWordEffect(dao, p[0], wordEntries(p.drop(2)), transaction) }.getOrDefault(SettingsUiWordOutcome.FAILED)
    }
    return wordReport(p[1], dao, outcome)
}

/** The words of an effect's rows. */
internal fun wordEntries(rows: List<String>): List<UserDictionaryEntry> = rows.chunked(SettingsUiWord.SIZE).map { f ->
    UserDictionaryEntry(id = f[SettingsUiWord.ID].toLong(), word = f[SettingsUiWord.WORD], freq = f[SettingsUiWord.FREQ].toInt(),
        locale = f[SettingsUiWord.LOCALE].takeIf { WireFlag.decode(f[SettingsUiWord.HAS_LOCALE]) },
        shortcut = f[SettingsUiWord.SHORTCUT].takeIf { WireFlag.decode(f[SettingsUiWord.HAS_SHORTCUT]) })
}

/** Carries out [kind] on [dao] with [words]; the `SettingsUiWordOutcome`. Adding and changing store the locale
 * as the dialog did, normalised by `FlorisLocale`; an import upserts each word in order inside [transaction]. */
internal fun applyWordEffect(dao: UserDictionaryDao, kind: String, words: List<UserDictionaryEntry>,
    transaction: (() -> Unit) -> Unit = { it() }): String {
    when (kind) {
        SettingsUiWordEffect.ADD -> dao.insert(normalized(words.single()))
        SettingsUiWordEffect.CHANGE -> dao.update(normalized(words.single()))
        SettingsUiWordEffect.DELETE -> dao.delete(words.single())
        SettingsUiWordEffect.RESTORE -> restoreWord(dao, words.single())
        SettingsUiWordEffect.IMPORT -> { transaction { words.forEach { importWord(dao, it) } }; return SettingsUiWordOutcome.IMPORTED }
    }
    return SettingsUiWordOutcome.NONE
}

/** [entry] with its locale as the dialog stored it: `FlorisLocale.fromTag(it).localeTag()`. */
internal fun normalized(entry: UserDictionaryEntry) = entry.copy(locale = entry.locale?.let { FlorisLocale.fromTag(it).localeTag() })

/** A deleted word stored again. In Room it keeps its id, which AUTOINCREMENT never hands out again, so it keeps
 * its place in an export; were the id taken, it gets a new one. The system's provider always gives a new one. */
internal fun restoreWord(dao: UserDictionaryDao, entry: UserDictionaryEntry) {
    val taken = dao.queryAll().any { it.id == entry.id }
    dao.insert(if (taken) entry.copy(id = 0) else entry)
}

/** One imported word, as `importCombinedList` wrote it: the first word stored with that text and locale updated,
 * or a new one inserted, the locale kept as the file spelled it. */
internal fun importWord(dao: UserDictionaryDao, entry: UserDictionaryEntry) {
    val existing = dao.queryExact(entry.word, entry.locale?.let { FlorisLocale.fromTag(it) })
    if (existing.isNotEmpty()) dao.update(entry.copy(id = existing[0].id)) else dao.insert(entry.copy(id = 0))
}

/** The `WORDS` report: the dictionary, whether it is on and [outcome], its words in `queryAll()` order, then each
 * distinct locale they are stored under, named as Java names it. */
internal fun wordReport(type: String, dao: UserDictionaryDao?, outcome: String): Array<String> {
    val words = dao?.queryAll().orEmpty()
    val out = mutableListOf<String>()
    out += wordRow(SettingsUiWordKind.DICTIONARY, id = type, word = WireFlag.encode(dao != null), label = outcome)
    for (w in words) out += SettingsUiWord.row(SettingsUiWord.KIND to SettingsUiWordKind.WORD, SettingsUiWord.ID to "${w.id}",
        SettingsUiWord.WORD to w.word, SettingsUiWord.FREQ to "${w.freq}", SettingsUiWord.LOCALE to w.locale.orEmpty(),
        SettingsUiWord.HAS_LOCALE to WireFlag.encode(w.locale != null), SettingsUiWord.SHORTCUT to w.shortcut.orEmpty(),
        SettingsUiWord.HAS_SHORTCUT to WireFlag.encode(w.shortcut != null), SettingsUiWord.LABEL to "", SettingsUiWord.LANGUAGE to "")
    for (stored in words.mapNotNull { it.locale }.distinct()) {
        val locale = FlorisLocale.fromTag(stored)
        out += wordRow(SettingsUiWordKind.LOCALE, locale = stored, label = locale.displayName(), language = locale.displayLanguage())
    }
    return out.toTypedArray()
}

private fun wordRow(kind: String, id: String = "", word: String = "", locale: String = "", label: String = "", language: String = "") =
    SettingsUiWord.row(SettingsUiWord.KIND to kind, SettingsUiWord.ID to id, SettingsUiWord.WORD to word, SettingsUiWord.FREQ to "",
        SettingsUiWord.LOCALE to locale, SettingsUiWord.HAS_LOCALE to "", SettingsUiWord.SHORTCUT to "", SettingsUiWord.HAS_SHORTCUT to "",
        SettingsUiWord.LABEL to label, SettingsUiWord.LANGUAGE to language)

/** Import's read and export's write, as Rust asks for them. A read hands Rust the whole document, up to [LIMIT]
 * characters; a write puts Rust's text after `dictionary=` and the document's last path segment, as
 * `exportCombinedList` began, which only Kotlin sees. */
internal object WordFileWork {
    val JOBS = setOf(SettingsUiJob.WORDS_READ, SettingsUiJob.WORDS_WRITE)
    /** More than any dictionary needs; a larger document is not one, and is not read into memory. */
    const val LIMIT = 4 shl 20

    suspend fun run(context: Context, p: List<String>, document: (String) -> Uri?): Array<String> {
        val job = p[SettingsUiWork.JOB]
        val uri = document(p[SettingsUiWork.DOCUMENT]) ?: return BackupFiles.done(job, SettingsUiJobResult.GONE)
        val resolver = context.contentResolver
        return withContext(Dispatchers.IO) {
            if (job == SettingsUiJob.WORDS_READ) {
                var text: String? = null
                runCatching { resolver.readText(uri) { text = readCapped(it, LIMIT) } }
                text?.let { BackupFiles.done(job, SettingsUiJobResult.OK, SettingsUiDone.TEXT to it) }
                    ?: BackupFiles.done(job, SettingsUiJobResult.UNREADABLE)
            } else {
                val written = runCatching { resolver.writeText(uri) { it.write(exportText(uri.lastPathSegment, p[SettingsUiWork.TEXT])) } }
                BackupFiles.done(job, if (written.isSuccess) SettingsUiJobResult.OK else SettingsUiJobResult.UNWRITABLE)
            }
        }
    }
}

/** What an export writes: `exportCombinedList`'s `dictionary=` and [segment], `null` when there is none, then [text]. */
internal fun exportText(segment: String?, text: String) = "dictionary=$segment$text"

/** All of [reader], or null past [limit] characters. */
internal fun readCapped(reader: java.io.Reader, limit: Int): String? {
    val out = StringBuilder()
    val buffer = CharArray(8192)
    while (true) {
        val n = reader.read(buffer)
        if (n < 0) return out.toString()
        if (out.length + n > limit) return null
        out.appendRange(buffer, 0, n)
    }
}
