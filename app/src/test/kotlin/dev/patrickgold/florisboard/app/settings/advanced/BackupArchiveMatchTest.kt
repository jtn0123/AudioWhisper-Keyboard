package dev.patrickgold.florisboard.app.settings.advanced

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.BackupPolicy
import dev.patrickgold.florisboard.lib.io.ReferenceZipUtils
import dev.patrickgold.florisboard.lib.io.ZipUtils
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileNotFoundException
import java.security.MessageDigest
import java.util.zip.CRC32
import org.florisboard.lib.kotlin.io.readJson
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

class BackupArchiveMatchTest {
    private fun outcome(block: () -> Any?): Any? = try { block() } catch (e: Exception) {
        e.javaClass.name to e.message?.replace("/reference/", "/destination/")?.replace("/actual/", "/destination/")
    }
    private fun tree(root: File) = root.walkTopDown().filter { it.isFile }.associate {
        it.relativeTo(root).path to MessageDigest.getInstance("SHA-256").digest(it.readBytes()).toList()
    }
    @Test fun generatedArchivesMatchLegacyExtractionMetadataAndRestorePlans() {
        val root = File("build/stage2-backup-utils/archives").apply { deleteRecursively(); mkdirs() }
        val metadata = """{"package":"dev.patrickgold.florisboard","versionCode":100,"versionName":"1.2.3","timestamp":123}"""
        val base = listOf("backup_metadata.json" to metadata,
            "files/" to "", "files/ime/" to "", "files/ime/theme/" to "", "files/ime/theme/sample" to "fixture",
            "clipboard/" to "", "clipboard/clipboard_text_items.json" to "[]")
        val fixtures = listOf(base, base.map { (n, c) -> n to c.replace(":100", ":64") },
            base.map { (n, c) -> n to c.replace(":100", ":999") },
            base.map { (n, c) -> n to c.replace(":100", ":63") },
            base.filterNot { it.first == "backup_metadata.json" },
            listOf("backup_metadata.json" to "{"), listOf("backup_metadata.json" to "{}"),
            base + listOf("../escape" to "traversal", "a/../../escape" to "traversal"),
            base + listOf("same" to "first", "same" to "last"),
            base + listOf("backup_metadata.json" to metadata.replace(":100", ":999")),
            base + listOf("missing/parent/file" to "fixture"),
            base + listOf("x".repeat(256) to "skipped"),
            base.map { (n, c) -> n to if (n.endsWith("items.json")) "corrupt" else c })
        val tape = GoldenTape("backup-archives")
        for ((index, entries) in fixtures.withIndex()) {
            val parent = File(root, "$index").apply { mkdirs() }
            val archive = File(parent, "generated.zip").apply { writeBytes(zip(entries)) }
            val reference = File(parent, "reference").apply { mkdirs() }
            val actual = File(parent, "actual").apply { mkdirs() }
            assertEquals(outcome { ReferenceZipUtils.unzip(archive, reference) },
                outcome { ZipUtils.unzip(archive, actual, BackupPolicy::archive) })
            assertEquals(tree(reference), tree(actual))
            assertFalse(File(parent, "escape").exists())
            val inspected = outcome {
                val data = try { File(actual, "backup_metadata.json").readJson<Backup.Metadata>() }
                catch (e: FileNotFoundException) { error(BackupPolicy.missingMetadata) }
                BackupPolicy.metadata(data.packageName, data.versionCode, 100)
            }
            tape.answer("inspect $index", inspected)
            for (path in BackupPolicy.layout("jetpref/preferences.json").drop(3)) {
                val left = File(reference, path)
                val right = File(actual, path)
                if (left.exists()) assertEquals(outcome { left.readJson<List<ClipboardItem>>().size },
                    outcome { right.readJson<List<ClipboardItem>>().size })
            }
            val paths = BackupPolicy.layout("jetpref/preferences.json")
            for (mask in 0..63) for (erase in listOf(false, true)) {
                val present = paths.map { File(actual, it).exists() }
                tape.answer("restore $index $mask $erase", BackupPolicy.restore(mask, erase, paths[0], present))
            }
        }
        tape.finish()
        for ((i, bytes) in listOf(byteArrayOf(1, 2, 3), zip(base).copyOf(25)).withIndex()) {
            val parent = File(root, "corrupt-$i").apply { mkdirs() }
            val archive = File(parent, "generated.zip").apply { writeBytes(bytes) }
            assertEquals(outcome { ReferenceZipUtils.unzip(archive, File(parent, "reference")) },
                outcome { ZipUtils.unzip(archive, File(parent, "actual"), BackupPolicy::archive) })
        }
    }
    /** Stored ZIP records permit duplicate names which ZipOutputStream refuses to generate. */
    private fun zip(entries: List<Pair<String, String>>): ByteArray {
        val local = ByteArrayOutputStream()
        val central = ByteArrayOutputStream()
        fun ByteArrayOutputStream.u16(n: Int) { write(n and 255); write((n ushr 8) and 255) }
        fun ByteArrayOutputStream.u32(n: Int) { u16(n and 65535); u16(n ushr 16) }
        for ((name, content) in entries) {
            val filename = name.toByteArray(Charsets.UTF_8)
            val data = content.toByteArray(Charsets.UTF_8)
            val crc = CRC32().also { it.update(data) }.value.toInt()
            val offset = local.size()
            local.u32(0x04034b50); local.u16(20); local.u16(0x800); local.u16(0); local.u16(0); local.u16(0)
            local.u32(crc); local.u32(data.size); local.u32(data.size); local.u16(filename.size); local.u16(0)
            local.write(filename); local.write(data)
            central.u32(0x02014b50); central.u16(20); central.u16(20); central.u16(0x800); central.u16(0); central.u16(0); central.u16(0)
            central.u32(crc); central.u32(data.size); central.u32(data.size); central.u16(filename.size); central.u16(0); central.u16(0)
            central.u16(0); central.u16(0); central.u32(0); central.u32(offset); central.write(filename)
        }
        val offset = local.size()
        local.write(central.toByteArray())
        local.u32(0x06054b50); local.u16(0); local.u16(0); local.u16(entries.size); local.u16(entries.size)
        local.u32(central.size()); local.u32(offset); local.u16(0)
        return local.toByteArray()
    }
}
