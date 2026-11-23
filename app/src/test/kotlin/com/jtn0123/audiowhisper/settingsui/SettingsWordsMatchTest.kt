package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.settings.advanced.BackupFiles
import dev.patrickgold.florisboard.ime.dictionary.RecordingUserDictionaryDao
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryEntry
import kotlinx.serialization.json.*
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/** Made-up words in both dictionaries' tests: two in American English spelt two ways, one for all languages, one in
 * German, and one under `all`, which the Floris dictionary reads as all languages. */
internal fun sampleWords() = listOf(
    UserDictionaryEntry(1, "zorblat", 200, "en_US", null), UserDictionaryEntry(2, "Quexor", 120, "en-us", "qx"),
    UserDictionaryEntry(3, "blivet", 255, null, null), UserDictionaryEntry(4, "frobnic", 90, "de_DE", "fb"),
    UserDictionaryEntry(5, "snarf", 10, "all", null))

/** What exportCombinedList wrote for [sampleWords] into document `document:12` at [now], frozen: in Room's row order,
 * or with [system] in the provider's, by frequency. */
internal fun frozenExport(system: Boolean, now: Long = 1_800_000_000_000) =
    "dictionary=document:12;date=$now;generated-by=com.jtn0123.audiowhisper.keyboard;version=1\n" + if (system) {
        " w=blivet;f=255;l=null\n w=zorblat;f=200;l=en_US\n w=Quexor;f=120;l=en-us;s=qx\n w=frobnic;f=90;l=de_DE;s=fb\n" +
            " w=snarf;f=10;l=all\n"
    } else {
        " w=zorblat;f=200;l=en_US\n w=Quexor;f=120;l=en-us;s=qx\n w=blivet;f=255;l=null\n w=frobnic;f=90;l=de_DE;s=fb\n" +
            " w=snarf;f=10;l=all\n"
    }

/** The user dictionary routes Rust draws, in the order it lists them: each dictionary's first screen, its words, and
 * its editor for a new word and for a stored one. */
internal val wordRoutes = listOf("FLORIS", "SYSTEM").flatMap { type ->
    listOf("", "/words", "/word/add", "/word/edit").map { "settings/dictionary/user-dictionary/$type$it" }
}

/**
 * The user dictionary screens through the real native settings, each change carried out by SettingsWords' own
 * [wordAnswer] on a recording dictionary: the DAO calls, and so the Room rows and provider values, that the
 * UserDictionaryScreen and UserDictionary code it replaced made, checked against golden digests of that code's answers
 * (docs/phase6-kotlin-references.md). Every word is made up, and none is printed.
 */
class SettingsWordsMatchTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private val root = "settings/dictionary/user-dictionary/"
    private val pkg = "com.jtn0123.audiowhisper.keyboard"
    private val wordEffects = setOf(SettingsUiWordEffect.LOAD, SettingsUiWordEffect.ADD, SettingsUiWordEffect.CHANGE,
        SettingsUiWordEffect.DELETE, SettingsUiWordEffect.RESTORE, SettingsUiWordEffect.IMPORT, SettingsUiWordEffect.MANAGER)
    private val strings: Map<String, String> by lazy {
        val pattern = Regex("""<string name="([^"]+)"[^>]*>(.*?)</string>""", RegexOption.DOT_MATCHES_ALL)
        listOf("strings.xml", "strings_settings_renderer.xml").flatMap { name ->
            pattern.findAll(File("src/main/res/values/$name").readText()).map { it.groupValues[1] to it.groupValues[2].replace("\\'", "'") }
        }.toMap()
    }
    private fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) =
        Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    /** The one element, failing as an assertion, so a mutation that loses it counts as caught. */
    private fun <T> List<T>.one(what: String) = also { assertEquals(1, size, "one $what") }[0]
    private fun dao(type: String, rows: List<UserDictionaryEntry> = sampleWords()) =
        RecordingUserDictionaryDao(rows, system = type == SettingsUiDictionary.SYSTEM)

    /** Carries out each user dictionary effect of [s] as SettingsWords does, answering with the report; the last state. */
    private fun answer(n: KeyboardRenderer, dao: RecordingUserDictionaryDao, s: JsonObject): JsonObject =
        effects(s).filter { it[0] in wordEffects }.fold(s) { _, e -> state(n, SettingsUiOp.WORDS, *wordAnswer(e, dao)) }

    /** Native settings on dictionary [type]'s first screen, its report in, then at each of [routes]. */
    private fun prepare(n: KeyboardRenderer, type: String, dao: RecordingUserDictionaryDao, vararg routes: String): JsonObject {
        n.settings(SettingsUiOp.RESOURCES, n.settings(SettingsUiOp.STRING_KEYS)[2].split("\n").filter { it in strings }
            .flatMap { listOf(it, strings.getValue(it)) }.toTypedArray())
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true")).toTypedArray())
        n.settings(SettingsUiOp.FACTS, arrayOf("package", pkg))
        val s = state(n, SettingsUiOp.NAVIGATE, root + type)
        assertEquals(listOf(listOf(SettingsUiWordEffect.LOAD, type)), effects(s))
        return routes.fold(answer(n, dao, s)) { _, r -> answer(n, dao, state(n, SettingsUiOp.NAVIGATE, root + type + r)) }
    }

    private fun row(e: UserDictionaryEntry) = buildJsonArray {
        listOf("", "${e.id}", e.word, "${e.freq}", e.locale.orEmpty(), WireFlag.encode(e.locale != null), e.shortcut.orEmpty(),
            WireFlag.encode(e.shortcut != null), "", "").forEach { add(it) }
    }
    private fun savedWords(fields: List<String>, stored: UserDictionaryEntry?) = buildJsonObject {
        put("draft", buildJsonArray { add(buildJsonArray { fields.forEach { add(it) } }); add(stored?.let(::row) ?: JsonNull); add(false) })
        put("search", ""); put("deleted", JsonNull); put("transfer", JsonNull)
    }.toString()

    @Test fun addingAndEditingWriteTheEntryTheDialogWrote() {
        val words = listOf("zorblat", "Quexor", " blivet ", "", "  ", "two words", "semi;colon", "com,ma", " snarf ",
            "😀", "novel", "frobnic", "a b")
        val freqs = listOf("255", "1", "0", "256", "", " ", "12", " 12", "12 ", "+7", "-1", "many", "2147483648", "007")
        val shortcuts = listOf("", " ", "qx", "q x", "a;b", " zz ", "fb")
        val locales = listOf("", " ", "en_US", "en-us", " de-DE ", "all", "null", "iw", "zz_ZZ", "x", "de", "EN_us")
        val random = Random(1729)
        val counts = mutableMapOf<String, Int>()
        GoldenTape.check("words-drafts") { tape ->
            for (type in listOf(SettingsUiDictionary.FLORIS, SettingsUiDictionary.SYSTEM)) KeyboardRenderer(arrayOf(font)).use { n ->
                prepare(n, type, dao(type), "/word/add")
                val add = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
                state(n, SettingsUiOp.NAVIGATE, "$root$type/word/edit/2")
                val edit = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
                repeat(2500) {
                    val dao = dao(type)
                    val stored = if (random.nextBoolean()) sampleWords().random(random) else null
                    val (word, freq, shortcut, locale) = listOf(words, freqs, shortcuts, locales).map { it.random(random) }
                    val saved = (if (stored == null) add else edit).copyOf()
                    saved[17] = savedWords(listOf(word, freq, shortcut, locale), stored)
                    answer(n, dao, state(n, SettingsUiOp.RESTORE, *saved))
                    answer(n, dao, state(n, SettingsUiOp.ACTIVATE, "1005"))
                    tape.answer("calls", dao.calls)
                    // The dialog's freq.toInt(10) threw on spaces around a valid frequency; the native editor trims it.
                    // The dialog also stored a word its language had already, where the native editor refuses it.
                    val outcome = when {
                        dao.calls.isEmpty() -> "refused"
                        freq.toIntOrNull(10) == null -> "trimmed"
                        else -> "written"
                    }
                    counts[outcome] = (counts[outcome] ?: 0) + 1
                }
            }
        }
        // The reference split the refused drafts into 3,902 invalid and 143 its language had already.
        for (outcome in listOf("refused", "trimmed", "written")) assertTrue((counts[outcome] ?: 0) >= 50, "$outcome: $counts")
    }

    @Test fun deleteRemovesTheStoredRowAsTheDialogDidAndUndoStoresItAgain() {
        for (type in listOf(SettingsUiDictionary.FLORIS, SettingsUiDictionary.SYSTEM)) KeyboardRenderer(arrayOf(font)).use { n ->
            val dao = dao(type)
            prepare(n, type, dao, "/words/en_US")
            // Quexor is first alphabetically, so slot 0: its Delete is 8003.
            answer(n, dao, state(n, SettingsUiOp.ACTIVATE, "8003"))
            val quexor = sampleWords()[1]
            assertEquals(listOf("delete" to quexor), dao.calls)
            assertEquals(sampleWords() - quexor, dao.rows)
            answer(n, dao, state(n, SettingsUiOp.ACTIVATE, "7901"))
            assertEquals(listOf("delete" to quexor, "insert" to quexor), dao.calls)
            // Room keeps the id, so the word keeps its place in an export; the provider gives it a new one.
            val back = if (type == SettingsUiDictionary.FLORIS) quexor else quexor.copy(id = 6)
            assertEquals((sampleWords() - quexor + back).sortedBy { it.id }, dao.rows.sortedBy { it.id })
        }
    }

    /** Import through the picker: Import, the picked document, its text read; the words' effect carried out. */
    private fun import(type: String, dao: RecordingUserDictionaryDao, file: String): JsonObject = KeyboardRenderer(arrayOf(font)).use { n ->
        prepare(n, type, dao)
        assertEquals(SettingsUiFileEffect.PICK, effects(state(n, SettingsUiOp.ACTIVATE, "1002")).one("pick")[0])
        val work = effects(state(n, SettingsUiOp.FILES, *pickedReport(SettingsUiPickOutcome.PICKED, listOf("0" to "words.clb")))).one("read")
        assertEquals(SettingsUiJob.WORDS_READ, work[SettingsUiWork.JOB])
        answer(n, dao, state(n, SettingsUiOp.FILES, *BackupFiles.done(SettingsUiJob.WORDS_READ, SettingsUiJobResult.OK, SettingsUiDone.TEXT to file)))
    }

    @Test fun anImportMakesTheCallsImportCombinedListMadeAndAnOldExportImports() {
        val old = frozenExport(system = false)
        val varied = "header line, ignored\r\n W = zorblat ; F = 3 ; L = en-us\r\n word=novel;freq=9;locale=all;extra=x\n" +
            "w=novel;f=10;l=null;s=nv\n w=Quexor;f=255;l=en_US;shortcut=q2\n w=frobnic;f=1;l=de-DE"
        GoldenTape.check("words-import") { tape ->
            for (type in listOf(SettingsUiDictionary.FLORIS, SettingsUiDictionary.SYSTEM)) {
                for ((file, rows) in listOf(old to sampleWords(), old to emptyList(), varied to sampleWords())) {
                    val after = dao(type, rows)
                    val s = import(type, after, file)
                    tape.answer("calls", after.calls); tape.answer("rows", after.rows)
                    assertEquals(listOf(strings.getValue("settings__udm__dictionary_import_success")),
                        effects(s).filter { it[0] == SettingsUiFileEffect.TOAST }.map { it[1] })
                }
            }
        }
    }

    @Test fun anImportWithABadLineWritesNothingWhereComposeWroteTheLinesBeforeIt() {
        val file = "header\n w=zorblat;f=7;l=en_US\n w=novel;f=1\n w=broken;f=0\n w=late;f=2\n"
        // Compose wrote the two lines before the bad one, then threw (docs/phase6-kotlin-references.md).
        val after = dao(SettingsUiDictionary.FLORIS)
        val s = import(SettingsUiDictionary.FLORIS, after, file)
        assertTrue(after.calls.isEmpty())
        assertEquals(sampleWords(), after.rows)
        val message = strings.getValue("settings__udm__import_line_error").replace("{line}", "4")
        assertEquals(listOf(listOf(SettingsUiFileEffect.TOAST, message)), effects(s).filter { it[0] == SettingsUiFileEffect.TOAST })
    }

    @Test fun anExportIsByteForByteWhatExportCombinedListWrote() {
        for (type in listOf(SettingsUiDictionary.FLORIS, SettingsUiDictionary.SYSTEM)) KeyboardRenderer(arrayOf(font)).use { n ->
            val dao = dao(type)
            prepare(n, type, dao)
            val before = System.currentTimeMillis()
            val asked = effects(state(n, SettingsUiOp.ACTIVATE, "1003"))
            assertEquals(listOf(SettingsUiWordEffect.LOAD, SettingsUiFileEffect.PICK), asked.map { it[0] })
            state(n, SettingsUiOp.WORDS, *wordAnswer(asked[0], dao))
            val work = effects(state(n, SettingsUiOp.FILES, *pickedReport(SettingsUiPickOutcome.PICKED, listOf("0" to "x.clb")))).one("write")
            assertEquals(SettingsUiJob.WORDS_WRITE, work[SettingsUiWork.JOB])
            val written = exportText("document:12", work[SettingsUiWork.TEXT])
            val now = assertNotNull(Regex(";date=(\\d+);").find(written), "date").groupValues[1].toLong()
            assertTrue(now in before..System.currentTimeMillis())
            assertEquals(frozenExport(type == SettingsUiDictionary.SYSTEM, now), written, type)
            assertEquals(SettingsUiFileEffect.TOAST, effects(state(n, SettingsUiOp.FILES,
                *BackupFiles.done(SettingsUiJob.WORDS_WRITE, SettingsUiJobResult.OK))).one("toast")[0])
        }
    }
}
