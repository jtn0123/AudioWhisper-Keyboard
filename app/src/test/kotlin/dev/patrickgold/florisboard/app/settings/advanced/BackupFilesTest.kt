package dev.patrickgold.florisboard.app.settings.advanced

import com.jtn0123.audiowhisper.rust.BackupPolicy
import com.jtn0123.audiowhisper.rust.SettingsUiDone
import com.jtn0123.audiowhisper.rust.SettingsUiFiles
import com.jtn0123.audiowhisper.rust.SettingsUiJob
import com.jtn0123.audiowhisper.rust.SettingsUiJobResult
import dev.patrickgold.florisboard.lib.io.ReferenceZipUtils
import java.io.File
import java.security.MessageDigest
import java.util.zip.ZipInputStream
import org.florisboard.lib.kotlin.io.writeJson
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The archives Rust's Back up and Restore ask for, against what Compose's BackupScreen wrote and
 * RestoreScreen read. Compose's steps are copied below as [composeArchive]; the zip they used is
 * [ReferenceZipUtils], FlorisBoard's own, so no code under test makes the fixtures.
 */
class BackupFilesTest {
    private val root = File("build/backup-files-test").apply { deleteRecursively(); mkdirs() }
    private val preferences = BackupIO.preferences
    private val paths = BackupPolicy.layout(preferences)
    private val metadata = Backup.Metadata("com.jtn0123.audiowhisper.keyboard", 120, "1.2", 1_763_719_200_000)
    private val textClip = """[{"id":1,"type":"TEXT","text":"hello","uri":null,"creationTimestampMs":5,"isPinned":false,"mimeTypes":["text/plain"]}]"""

    private fun dir(name: String) = File(root, name).apply { mkdirs() }
    /** What BackupIO.backup leaves in [input] for the groups in [mask]: the same files at the same paths. */
    private fun filled(input: File, mask: Int): File {
        for ((group, path) in paths.withIndex()) {
            if ((mask and (1 shl group)) == 0) continue
            val file = File(input, path)
            when (group) {
                0 -> file.apply { parentFile.mkdirs() }.writeText("""{"type":"jetpref-datastore","entries":["b;keyboard__x;true"]}""")
                1, 2 -> File(file, "org.example.$group/extension.json").apply { parentFile.mkdirs() }.writeText("{\"id\":$group}")
                3 -> file.apply { parentFile.mkdirs() }.writeText(textClip)
                else -> file.apply { parentFile.mkdirs() }.writeText("[]")
            }
        }
        return input
    }
    /** BackupScreen's prepareBackup after BackupIO.backup, as Compose wrote it. */
    private fun composeArchive(input: File, output: File, metadata: Backup.Metadata): File {
        File(input, BackupPolicy.metadataName).writeJson(metadata)
        val zip = File(output, BackupPolicy.fileName(metadata.packageName, metadata.versionCode, metadata.timestamp))
        ReferenceZipUtils.zip(input, zip)
        return zip
    }
    private fun sha(bytes: ByteArray) = MessageDigest.getInstance("SHA-256").digest(bytes).toList()
    private fun entries(zip: File): Map<String, List<Byte>> = ZipInputStream(zip.inputStream()).use { input ->
        generateSequence { input.nextEntry }.associate { it.name to sha(input.readBytes()) }
    }
    private fun tree(dir: File) = dir.walkTopDown().filter { it.isFile }.associate { it.relativeTo(dir).path to sha(it.readBytes()) }
    private fun inspect(archive: File, name: String) = BackupFiles.inspect(archive, dir(name), preferences) { "made $it" }
    /** A ZIP of [entries], each after its directories' entries, as ZipUtils.zip writes them. */
    private fun archiveOf(name: String, entries: List<Pair<String, String>>) = File(dir(name), "in.zip").also { zip ->
        val folders = entries.flatMap { (path, _) -> path.split('/').dropLast(1).runningReduce { a, b -> "$a/$b" } }.distinct()
        java.util.zip.ZipOutputStream(zip.outputStream()).use { out ->
            for (folder in folders) { out.putNextEntry(java.util.zip.ZipEntry("$folder/")); out.closeEntry() }
            for ((path, content) in entries) { out.putNextEntry(java.util.zip.ZipEntry(path)); out.write(content.toByteArray()); out.closeEntry() }
        }
    }

    @Test fun aBackupIsTheArchiveComposeMadeUnderTheNameItGave() {
        for (mask in listOf(0b111111, 0b000111, 0b000001, 0b000110, 0b111000)) {
            val compose = composeArchive(filled(dir("c$mask/in"), mask), dir("c$mask/out"), metadata)
            val ours = BackupFiles.pack(filled(dir("o$mask/in"), mask), dir("o$mask/out"), metadata)
            assertEquals(compose.name, ours.name)
            assertEquals(entries(compose), entries(ours), "mask $mask")
        }
    }

    /** Every selection Compose could back up reads back whole, unpacks to the tree Compose's
     * restore unpacked, and leads to the restore plan Compose ran for the groups the backup holds. */
    @Test fun everyComposeBackupReadsAsComposeReadItAndRestoresTheSame() {
        for (mask in 1..63) {
            val archive = composeArchive(filled(dir("m$mask/in"), mask), dir("m$mask/out"), metadata)
            val report = inspect(archive, "m$mask/read")
            assertEquals(SettingsUiFiles.DONE, report[SettingsUiDone.REPORT])
            assertEquals(SettingsUiJob.RESTORE_READ, report[SettingsUiDone.JOB])
            assertEquals(SettingsUiJobResult.OK, report[SettingsUiDone.RESULT])
            assertEquals(listOf(metadata.packageName, "120", "1.2", "made ${metadata.timestamp}", "$mask", "0"),
                listOf(SettingsUiDone.PACKAGE, SettingsUiDone.VERSION_CODE, SettingsUiDone.VERSION_NAME,
                    SettingsUiDone.DATE, SettingsUiDone.PRESENT, SettingsUiDone.DAMAGED).map { report[it] })
            val compose = dir("m$mask/compose").also { ReferenceZipUtils.unzip(archive, it) }
            assertEquals(tree(compose), tree(File(root, "m$mask/read")))
            val present = paths.map { File(compose, it).exists() }
            assertEquals((0 until 6).map { (mask and (1 shl it)) != 0 }, present)
            // The screen offers only the groups a backup holds, so its mask is the selection's part of them.
            for (selection in 0..63) for (erase in listOf(false, true)) {
                val composePlan = ReferenceBackupPolicy.restore(selection, erase, preferences, present)
                val ours = BackupPolicy.restore(selection and mask, erase, preferences, present)
                // Erasing a group the backup lacks emptied it in Compose; now it cannot be chosen.
                val lacking = (1..2).filter { (selection and (1 shl it)) != 0 && !present[it] }.map { paths[it] }
                assertEquals(composePlan.filterNot { it[0] == "clear-directory" && it[1] in lacking }, ours,
                    "mask $mask selection $selection erase $erase")
                if ((selection and mask.inv()) == 0 || !erase) assertEquals(composePlan, ours)
            }
        }
    }

    @Test fun filesThatAreNotUsableBackupsSayWhy() {
        val base = listOf(BackupPolicy.metadataName to """{"package":"com.jtn0123.audiowhisper.keyboard","versionCode":120,"versionName":"1.2","timestamp":5}""",
            paths[0] to "{}", paths[3] to textClip)
        val cases = listOf(
            "photo" to listOf("DCIM/photo.jpg" to "jpeg"),
            "brace" to listOf(BackupPolicy.metadataName to "{"),
            "empty" to listOf(BackupPolicy.metadataName to "{}"),
            "words" to listOf(BackupPolicy.metadataName to """{"package":"x","versionCode":"many","versionName":"","timestamp":1}"""),
        )
        val expected = listOf(SettingsUiJobResult.NOT_BACKUP, SettingsUiJobResult.BAD_METADATA,
            SettingsUiJobResult.BAD_METADATA, SettingsUiJobResult.BAD_METADATA)
        for ((case, result) in cases.zip(expected)) {
            assertEquals(result, inspect(archiveOf(case.first, case.second), "${case.first}/read")[SettingsUiDone.RESULT], case.first)
        }
        val whole = archiveOf("whole", base)
        // Bytes that are no ZIP at all, an empty file and a ZIP cut short cannot be unpacked.
        val broken = listOf(byteArrayOf(1, 2, 3), ByteArray(0), whole.readBytes().copyOf(40), whole.readBytes().copyOf(200))
        assertEquals(List(4) { SettingsUiJobResult.UNREADABLE }, broken.mapIndexed { i, bytes ->
            inspect(File(dir("broken$i"), "in.zip").apply { writeBytes(bytes) }, "broken$i/read")[SettingsUiDone.RESULT]
        })
        assertEquals(SettingsUiJobResult.UNREADABLE, inspect(File(root, "missing.zip"), "missing/read")[SettingsUiDone.RESULT])
        // A damaged clipboard file is held, and marked, so Restore can leave it out before changing anything.
        val damaged = inspect(archiveOf("damaged", base.map { (p, c) -> p to if (p == paths[3]) "corrupt" else c }), "damaged/read")
        assertEquals(listOf(SettingsUiJobResult.OK, "9", "8"),
            listOf(damaged[SettingsUiDone.RESULT], damaged[SettingsUiDone.PRESENT], damaged[SettingsUiDone.DAMAGED]))
        // A newer backup reads; Rust says it is newer.
        val newer = inspect(archiveOf("newer", base.map { (p, c) -> p to c.replace(":120", ":999") }), "newer/read")
        assertEquals(listOf(SettingsUiJobResult.OK, "999"), listOf(newer[SettingsUiDone.RESULT], newer[SettingsUiDone.VERSION_CODE]))
    }

    @Test fun aReportFillsOnlyWhatItIsGiven() {
        val report = BackupFiles.done(SettingsUiJob.BACKUP_PREPARE, SettingsUiJobResult.OK, SettingsUiDone.NAME to "b.zip")
        assertEquals(SettingsUiDone.SIZE, report.size)
        assertEquals(listOf(SettingsUiFiles.DONE, SettingsUiJob.BACKUP_PREPARE, SettingsUiJobResult.OK, "b.zip"), report.take(4))
        assertTrue(report.drop(4).all { it.isEmpty() })
    }
}
