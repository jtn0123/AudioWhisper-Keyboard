package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.ime.keyboard.KeyboardExtension
import dev.patrickgold.florisboard.ime.nlp.LanguagePackExtension
import dev.patrickgold.florisboard.ime.theme.ThemeExtension
import dev.patrickgold.florisboard.lib.cache.CacheManager
import dev.patrickgold.florisboard.lib.ext.ExtensionMaintainer
import dev.patrickgold.florisboard.lib.ext.ExtensionMeta
import dev.patrickgold.florisboard.lib.ext.ReferenceExtensionPolicy
import dev.patrickgold.florisboard.lib.io.FileRegistry
import kotlinx.serialization.json.*
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/**
 * The importer through the real native settings, its report built by ExtensionFileWork's own [importRows], against what
 * the frozen ExtensionImportScreen decided for the same files: which it skipped and why, and which Import imported.
 * One change is named: a file of a type the importer does not take is shown skipped, and Import stays off when nothing
 * else can be imported, where Compose listed it as importable, left it out and still said the import worked.
 * Every name and id is made up.
 */
class SettingsExtensionImportMatchTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) =
        Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    private fun node(s: JsonObject, id: Int) = nodes(s).singleOrNull { it.text("id") == id.toString() }

    private val random = Random(2020)
    private val reasons = mapOf("unsupported" to "R:unsupported", "corrupted" to "R:corrupted", "core" to "R:core")
    private val types = listOf("EXT_ANY" to "ext-any", "EXT_KEYBOARD" to "ext-keyboard", "EXT_THEME" to "ext-theme",
        "EXT_LANGUAGEPACK" to "ext-languagepack")
    private val core = setOf("core.ext")

    private fun meta(id: String) = ExtensionMeta(id = id, version = "1.${random.nextInt(9)}", title = "Title $id",
        description = "About $id".takeIf { random.nextBoolean() }, maintainers = listOf(ExtensionMaintainer("M")), license = "mit")
    private fun ext(id: String) = when (random.nextInt(4)) {
        0 -> null
        1 -> KeyboardExtension(meta(id))
        2 -> ThemeExtension(meta(id), themes = emptyList())
        else -> LanguagePackExtension(meta(id))
    }
    private fun file(f: Int) = CacheManager.FileInfo(File("/nowhere/$f-" + listOf("a.flex", "b.zip", "c.txt", "d", "e.flex").random(random)),
        listOf(null, FileRegistry.FlexExtension.mediaType, "application/octet-stream", "text/plain").random(random),
        random.nextLong(0, 1L shl 20), ext(listOf("x$f.ext", "core.ext").random(random)))

    /** A phone-wide window, and the skip reasons as strings a test can tell apart. */
    private fun configure(n: KeyboardRenderer) {
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true")).toTypedArray())
        n.settings(SettingsUiOp.RESOURCES, arrayOf("ext__import__file_skip_unsupported", "R:unsupported",
            "ext__import__file_skip_ext_corrupted", "R:corrupted", "ext__import__file_skip_ext_core", "R:core",
            "settings__ext__skip_other_type", "R:other"))
    }
    /** The importer of [type] showing [files] as workspace `w1`. */
    private fun open(n: KeyboardRenderer, type: String, files: List<CacheManager.FileInfo>?): JsonObject {
        configure(n)
        val route = "ext/import/$type?files=w1"
        assertEquals(listOf(listOf(SettingsUiExtensionEffect.FILES, "w1")), effects(state(n, SettingsUiOp.NAVIGATE, route)))
        return state(n, SettingsUiOp.EXTENSION_FILES, *importRows("w1", files, { "$it B" }) { it in core })
    }
    /** What ExtensionImportScreen's getSkipReason said of [info]: an ExtensionSkipReason, or empty. */
    private fun composeSkip(info: CacheManager.FileInfo) = ReferenceExtensionPolicy.skip(
        FileRegistry.matchesFileFilter(info, listOf(FileRegistry.FlexExtension)), info.ext != null, info.ext?.meta?.id in core)
    private fun boxLabels(s: JsonObject, f: Int) =
        nodes(s).filter { it.text("id").toInt() in 321_000 + 1000 * f until 322_000 + 1000 * f }.map { it.text("label") }

    @Test fun eachFileShowsWhatComposeShowedAndImportImportsWhatComposeImported() {
        // What Compose showed and imported also goes on a golden tape (docs/phase6-kotlin-references.md).
        val tape = GoldenTape.ofReference("extension-import")
        repeat(40) {
            val files = List(1 + random.nextInt(5)) { file(it) }
            for ((type, id) in types) KeyboardRenderer(arrayOf(font)).use { n ->
                val s = open(n, type, files)
                val takes = files.map { f -> f.ext?.let { ReferenceExtensionPolicy.accepts(id, it.serialType()) } == true }
                for ((f, info) in files.withIndex()) {
                    val labels = boxLabels(s, f)
                    assertEquals(listOf(info.file.name, info.mediaType ?: "application/unknown"), labels.take(2))
                    val head = listOfNotNull("${info.size} B", info.ext?.meta?.id, info.ext?.meta?.version).joinToString(" • ")
                    assertEquals(head, labels[2])
                    val compose = composeSkip(info)
                    val shown = reasons[compose] ?: "R:other".takeIf { !takes[f] }
                    tape.answer("shown $type $f", shown)
                    assertEquals(shown, labels.last().takeIf { it.startsWith("R:") }, "$type $f")
                }
                val imported = files.indices.filter { composeSkip(files[it]).isEmpty() && takes[it] }
                tape.answer("imported $type", imported.joinToString(","))
                val import = node(s, 1002)!!
                assertEquals(imported.isNotEmpty().toString(), import.text("enabled"), type)
                if (imported.isEmpty()) return@use
                val work = effects(state(n, SettingsUiOp.ACTIVATE, "1002")).single()
                assertEquals(SettingsUiJob.EXT_IMPORT, work[SettingsUiWork.JOB])
                assertEquals("w1", work[SettingsUiWork.EXTENSION])
                assertEquals(imported.joinToString(","), work[SettingsUiWork.TEXT])
            }
        }
        tape.finish()
    }

    /** Compose's Import was on whenever one file passed getSkipReason, even when its type was not the importer's. */
    @Test fun aFileOfAnotherTypeIsShownSkippedAndLeavesImportOff() {
        val theme = CacheManager.FileInfo(File("/nowhere/a.flex"), FileRegistry.FlexExtension.mediaType, 9, ThemeExtension(meta("t.ext"), themes = emptyList()))
        assertEquals("", composeSkip(theme))
        KeyboardRenderer(arrayOf(font)).use { n ->
            val s = open(n, "EXT_KEYBOARD", listOf(theme))
            assertEquals("R:other", boxLabels(s, 0).last())
            assertEquals("false", node(s, 1002)!!.text("enabled"))
        }
    }

    /** A workspace Android cleared, as after the app was ended, leaves the importer offering Select files again. */
    @Test fun aClosedWorkspaceOffersSelectFilesAgain() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            val s = open(n, "EXT_ANY", null)
            assertEquals("ext/import/EXT_ANY", s.text("route"))
            assertNotNull(node(s, 320_000))
            assertEquals("false", node(s, 1002)!!.text("enabled"))
        }
    }

    /** Improvement: ExtensionExportScreen left a half-written archive in the picked document when export threw. */
    @Test fun aFailedExportDeletesWhatItWrote() {
        val calls = mutableListOf<String>()
        assertTrue(writeOrDelete({ calls += "write" }) { calls += "delete" })
        assertEquals(listOf("write"), calls)
        calls.clear()
        assertFalse(writeOrDelete({ calls += "write"; error("disk full") }) { calls += "delete" })
        assertEquals(listOf("write", "delete"), calls)
        assertFalse(writeOrDelete({ error("disk full") }) { error("gone") })
    }
}
