package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.settings.advanced.Backup
import dev.patrickgold.florisboard.app.settings.advanced.BackupFiles
import dev.patrickgold.florisboard.lib.io.ReferenceZipUtils
import kotlinx.serialization.json.*
import java.io.File
import org.florisboard.lib.kotlin.io.writeJson
import kotlin.test.*

/**
 * Back up and Restore through the real native settings, with the real strings: a backup Compose
 * made is picked, read by [BackupFiles], shown, confirmed and restored, and the restore Rust asks
 * for is the plan Compose ran for the same choices. Then what TalkBack is told.
 */
class SettingsBackupFlowTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private val root = File("build/settings-backup-flow").apply { deleteRecursively(); mkdirs() }
    private val preferences = "jetpref_datastore/florisboard-app-prefs.jetpref"
    private val paths = BackupPolicy.layout(preferences)
    private val strings: Map<String, String> by lazy {
        val pattern = Regex("""<string name="([^"]+)"[^>]*>(.*?)</string>""", RegexOption.DOT_MATCHES_ALL)
        listOf("strings.xml", "strings_dont_translate.xml", "strings_settings_renderer.xml").flatMap { name ->
            pattern.findAll(File("src/main/res/values/$name").readText()).map { it.groupValues[1] to it.groupValues[2] }
        }.toMap()
    }
    private fun s(key: String) = strings.getValue(key).replace("\\'", "'")
    private fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) =
        Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    private fun node(s: JsonObject, label: String) = nodes(s).single { it.text("label") == label }
    private fun tap(n: KeyboardRenderer, s: JsonObject, label: String) =
        state(n, SettingsUiOp.ACTIVATE, node(s, label).text("id"))
    private fun prepare(n: KeyboardRenderer) {
        n.settings(SettingsUiOp.RESOURCES, n.settings(SettingsUiOp.STRING_KEYS)[2].split("\n").filter { it in strings }
            .flatMap { listOf(it, s(it)) }.toTypedArray())
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true")).toTypedArray())
        n.settings(SettingsUiOp.FACTS, arrayOf("version_name", "1.2", "version_code", "120", "package", PACKAGE))
    }
    /** A backup of [mask] as Compose's BackupScreen made it (see BackupFilesTest). */
    private fun composeBackup(mask: Int): File {
        val input = File(root, "in$mask").apply { mkdirs() }
        for ((group, path) in paths.withIndex()) if ((mask and (1 shl group)) != 0) {
            val file = if (group in 1..2) File(input, "$path/x/extension.json") else File(input, path)
            file.parentFile.mkdirs()
            file.writeText(if (group >= 3) "[]" else "{}")
        }
        val metadata = Backup.Metadata(PACKAGE, 120, "1.2", 1_763_719_200_000)
        File(input, BackupPolicy.metadataName).writeJson(metadata)
        return File(root, "backup$mask.zip").also { ReferenceZipUtils.zip(input, it) }
    }
    private fun work(e: List<List<String>>) = e.single { it[0] == SettingsUiFileEffect.WORK }

    @Test fun aComposeBackupIsRestoredThroughTheRustScreensAsComposeRestoredIt() {
        // The plans the screens chose, as Compose chose them for the same ticks.
        val tape = GoldenTape("backup-flow-plans")
        for (mask in listOf(0b111111, 0b000111, 0b001111, 0b100001)) for (erase in listOf(false, true)) {
            val archive = composeBackup(mask)
            KeyboardRenderer(arrayOf(font)).use { n ->
                prepare(n)
                val other = state(n, SettingsUiOp.NAVIGATE, "settings/other")
                var s = tap(n, other, s("backup_and_restore__restore__title"))
                assertEquals("settings/other/restore", s.text("route"))
                s = tap(n, s, s("action__select_file"))
                assertEquals(listOf(SettingsUiFileEffect.PICK, SettingsUiPickMode.OPEN, "*/*", ""), effects(s).single())
                s = state(n, SettingsUiOp.FILES, *pickedReport(SettingsUiPickOutcome.PICKED, listOf("0" to "backup.zip")))
                val read = work(effects(s))
                assertEquals(listOf(SettingsUiJob.RESTORE_READ, "0"), listOf(read[SettingsUiWork.JOB], read[SettingsUiWork.DOCUMENT]))
                val out = File(root, "out$mask$erase").apply { mkdirs() }
                s = state(n, SettingsUiOp.FILES, *BackupFiles.inspect(archive, out, preferences) { "21 Nov 2026" })
                for (shown in listOf("backup.zip", "21 Nov 2026", "1.2 (120)", PACKAGE)) {
                    assertTrue(nodes(s).any { it.text("label") == shown || it.text("state") == shown }, shown)
                }
                // Erasing, the Compose user ticks just the groups the backup holds; merging keeps the defaults.
                // (Compose also let a group the backup lacks be ticked, and erasing then emptied it: BackupFilesTest.)
                if (erase) {
                    s = tap(n, s, s("backup_and_restore__restore__mode_erase_and_overwrite"))
                    if ((mask and BackupPolicyMasks.CLIPS) != 0) s = tap(n, s, s("backup_and_restore__back_up__files_clipboard_history"))
                }
                s = tap(n, s, s("action__restore"))
                assertTrue(effects(s).isEmpty(), "asks first")
                assertTrue(nodes(s).any { it.text("role") == "text" && "21 Nov 2026" in it.text("label") })
                s = state(n, SettingsUiOp.ACTIVATE, "7")
                val apply = work(effects(s))
                val chosen = apply[SettingsUiWork.MASK].toInt()
                assertEquals(WireFlag.encode(erase), apply[SettingsUiWork.ERASE])
                val present = paths.map { File(out, it).exists() }
                tape.answer("$mask $erase", BackupPolicy.restore(chosen, erase, preferences, present))
                s = state(n, SettingsUiOp.FILES, *BackupFiles.done(SettingsUiJob.RESTORE_APPLY, SettingsUiJobResult.OK))
                assertEquals(listOf(SettingsUiFileEffect.TOAST, s("backup_and_restore__restore__success")), effects(s).first())
                assertEquals("settings/other", s.text("route"))
            }
        }
        tape.finish()
    }

    @Test fun talkBackHearsTheChoicesThePartlyTickedClipboardAndTheNotes() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n)
            var s = state(n, SettingsUiOp.NAVIGATE, "settings/other/backup")
            val toFile = node(s, s("backup_and_restore__back_up__destination_file_sys"))
            assertEquals(listOf("radio", "true"), listOf(toFile.text("role"), toFile.text("checked")))
            val clipboard = s("backup_and_restore__back_up__files_clipboard_history")
            assertEquals(listOf("checkbox", "false"), listOf(node(s, clipboard).text("role"), node(s, clipboard).text("checked")))
            s = tap(n, s, s("backup_and_restore__back_up__files_clipboard_history__clipboard_text_items"))
            assertEquals("partial", node(s, clipboard).text("checked"))
            s = tap(n, s, s("action__back_up"))
            val note = node(s, s("settings__backup__preparing"))
            assertEquals(listOf("status", "true"), listOf(note.text("role"), note.text("live")))
            // While the backup is made only Cancel can be pressed.
            assertEquals("false", node(s, s("action__back_up")).text("enabled"))
            assertEquals("true", node(s, s("action__cancel")).text("enabled"))
            s = state(n, SettingsUiOp.FILES, *BackupFiles.done(SettingsUiJob.BACKUP_PREPARE, SettingsUiJobResult.FAILED))
            assertEquals("true", node(s, s("settings__backup__prepare_failed")).text("live"))
            assertEquals("true", node(s, s("action__back_up")).text("enabled"))
        }
    }

    private object BackupPolicyMasks { const val CLIPS = 0b111000 }
    private companion object { const val PACKAGE = "com.jtn0123.audiowhisper.keyboard" }
}
