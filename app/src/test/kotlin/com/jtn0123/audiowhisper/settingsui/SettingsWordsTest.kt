package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.ime.dictionary.RecordingUserDictionaryDao
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryEntry
import dev.patrickgold.florisboard.lib.FlorisLocale
import java.io.StringReader
import kotlin.test.*

/** SettingsWords' adapter on its own: the report's rows, the stored formats frozen as literals, the locale the dialog
 * stored, the undo's id, the import's transaction, and a read that stops at its limit. Every word is made up. */
class SettingsWordsTest {
    private fun fields(rows: Array<String>, at: Int) = rows.toList().subList(at * SettingsUiWord.SIZE, (at + 1) * SettingsUiWord.SIZE)

    /** The export format is frozen in [frozenExport], which SettingsWordsMatchTest holds the native export to. Kotlin
     * writes the document's segment in front, `null` when it has none, as StringBuilder.append(null) did. */
    @Test fun anExportNamesTheDocumentOrNull() {
        assertEquals("dictionary=null;date=1", exportText(null, ";date=1"))
        assertEquals(frozenExport(system = false).substringBefore(";date="), exportText("document:12", ""))
    }

    /** The adapter stores the locale the dialog stored: frozen as literals, and as a golden digest of the dialog's. */
    @Test fun addingAndChangingStoreTheLocaleTheDialogStored() {
        val stored = mapOf("en-us" to "en_US", "EN_us" to "en_US", "de-DE" to "de_DE", "fr" to "fr", "pt_BR" to "pt_BR")
        for ((typed, tag) in stored) {
            assertEquals(tag, normalized(UserDictionaryEntry(0, "w", 1, typed, null)).locale)
        }
        // The rest against a golden digest of the dialog's locales (docs/phase6-kotlin-references.md).
        GoldenTape.check("words-locales") { tape ->
            for (typed in listOf("iw", "in_ID", "zz_ZZ", "x", "all", "null", "en_US_POSIX", "sr-Latn-RS", "de_DE_1996")) {
                tape.answer("locale", normalized(UserDictionaryEntry(0, "w", 1, typed, null)).locale)
            }
        }
        val dao = RecordingUserDictionaryDao()
        applyWordEffect(dao, SettingsUiWordEffect.ADD, listOf(UserDictionaryEntry(0, "w", 1, "en-us", "s")))
        applyWordEffect(dao, SettingsUiWordEffect.CHANGE, listOf(UserDictionaryEntry(1, "w", 2, null, null)))
        assertEquals(listOf("insert" to UserDictionaryEntry(0, "w", 1, "en_US", "s"), "update" to UserDictionaryEntry(1, "w", 2, null, null)), dao.calls)
    }

    @Test fun theReportCarriesEveryWordAsStoredThenEachLocaleOnce() {
        val rows = wordReport(SettingsUiDictionary.FLORIS, RecordingUserDictionaryDao(sampleWords()), SettingsUiWordOutcome.IMPORTED)
        assertEquals((1 + 5 + 4) * SettingsUiWord.SIZE, rows.size)
        val head = fields(rows, 0)
        assertEquals(listOf(SettingsUiWordKind.DICTIONARY, SettingsUiDictionary.FLORIS, WireFlag.encode(true), SettingsUiWordOutcome.IMPORTED),
            listOf(head[SettingsUiWord.KIND], head[SettingsUiWord.ID], head[SettingsUiWord.WORD], head[SettingsUiWord.LABEL]))
        val words = (1..5).map { fields(rows, it) }
        assertTrue(words.all { it[SettingsUiWord.KIND] == SettingsUiWordKind.WORD })
        assertEquals(sampleWords(), wordEntries(words.flatten()))
        // A row as the wire carries it, frozen.
        assertEquals(listOf("word", "2", "Quexor", "120", "en-us", WireFlag.encode(true), "qx", WireFlag.encode(true), "", ""), words[1])
        val locales = (6..9).map { fields(rows, it) }
        assertEquals(listOf("en_US", "en-us", "de_DE", "all"), locales.map { it[SettingsUiWord.LOCALE] })
        for (l in locales) {
            val locale = FlorisLocale.fromTag(l[SettingsUiWord.LOCALE])
            assertEquals(listOf(SettingsUiWordKind.LOCALE, locale.displayName(), locale.displayLanguage()),
                listOf(l[SettingsUiWord.KIND], l[SettingsUiWord.LABEL], l[SettingsUiWord.LANGUAGE]))
        }
        // The system dictionary lists by frequency, highest first, as its provider queries.
        val system = wordReport(SettingsUiDictionary.SYSTEM, RecordingUserDictionaryDao(sampleWords(), system = true), SettingsUiWordOutcome.NONE)
        assertEquals(listOf("3", "1", "2", "4", "5"), (1..5).map { fields(system, it)[SettingsUiWord.ID] })
    }

    @Test fun aDictionaryThatIsOffIsReportedOffAndEveryChangeFails() {
        val load = wordAnswer(listOf(SettingsUiWordEffect.LOAD, SettingsUiDictionary.SYSTEM), null)
        assertEquals(SettingsUiWord.SIZE, load.size)
        assertEquals(listOf(WireFlag.encode(false), SettingsUiWordOutcome.NONE), listOf(load[SettingsUiWord.WORD], load[SettingsUiWord.LABEL]))
        val add = listOf(SettingsUiWordEffect.ADD, SettingsUiDictionary.SYSTEM) + fields(wordReport("", RecordingUserDictionaryDao(sampleWords()), ""), 1)
        assertEquals(SettingsUiWordOutcome.FAILED, wordAnswer(add, null)[SettingsUiWord.LABEL])
        // A refused change is reported as failed, with the dictionary as it is.
        val dao = RecordingUserDictionaryDao(sampleWords())
        val restore = listOf(SettingsUiWordEffect.RESTORE, SettingsUiDictionary.FLORIS) + add.drop(2)
        assertEquals(SettingsUiWordOutcome.NONE, wordAnswer(restore, dao)[SettingsUiWord.LABEL])
        assertEquals(UserDictionaryEntry(6, "zorblat", 200, "en_US", null), dao.rows.last(), "a taken id is not reused")
    }

    @Test fun anImportRunsInOneTransactionAndAFailedOneReportsFailed() {
        var transactions = 0
        val dao = RecordingUserDictionaryDao(sampleWords())
        val words = listOf(UserDictionaryEntry(0, "novel", 3, null, null), UserDictionaryEntry(0, "zorblat", 9, "en-us", null))
        val rows = words.flatMap { fields(wordReport("", RecordingUserDictionaryDao(listOf(it)), ""), 1) }
        val p = listOf(SettingsUiWordEffect.IMPORT, SettingsUiDictionary.FLORIS) + rows
        val report = wordAnswer(p, dao) { block -> transactions++; block() }
        assertEquals(1, transactions)
        assertEquals(SettingsUiWordOutcome.IMPORTED, report[SettingsUiWord.LABEL])
        // What importCombinedList made of "h\n w=novel;f=3\n w=zorblat;f=9;l=en-us\n".
        assertEquals(listOf("insert" to UserDictionaryEntry(0, "novel", 3, null, null),
            "update" to UserDictionaryEntry(1, "zorblat", 9, "en-us", null)), dao.calls)
        // A transaction that throws, as Room's rolls back, is reported failed.
        val failed = wordAnswer(p, RecordingUserDictionaryDao(sampleWords())) { error("rolled back") }
        assertEquals(SettingsUiWordOutcome.FAILED, failed[SettingsUiWord.LABEL])
    }

    @Test fun aReadStopsPastItsLimitWithoutKeepingTheRest() {
        assertEquals("abc", readCapped(StringReader("abc"), 3))
        assertNull(readCapped(StringReader("abcd"), 3))
        assertEquals("", readCapped(StringReader(""), 0))
        val long = "x".repeat(20_000)
        assertEquals(long, readCapped(StringReader(long), 20_000))
        assertNull(readCapped(StringReader(long), 19_999))
        assertEquals(4 shl 20, WordFileWork.LIMIT)
    }
}
