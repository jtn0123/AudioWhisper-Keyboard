package dev.patrickgold.florisboard.lib.ext

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.ExtensionPolicy
import dev.patrickgold.florisboard.lib.io.ReferenceZipUtils
import dev.patrickgold.florisboard.lib.io.ZipUtils
import java.io.ByteArrayOutputStream
import java.io.File
import java.security.MessageDigest
import java.util.zip.CRC32
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ExtensionArchiveMatchTest {
    private fun outcome(body: () -> Unit): Any = try { body(); "ok" } catch (e: Exception) { e.javaClass.name to e.message?.replace("/reference/", "/destination/")?.replace("/actual/", "/destination/") }
    private fun tree(root: File) = root.walkTopDown().filter { it.isFile }.associate {
        it.relativeTo(root).path to MessageDigest.getInstance("SHA-256").digest(it.readBytes()).toList()
    }
    /** [value] without the checkout's path, which a golden must not hold. */
    private fun stable(value: Any?, root: File) = value.toString().replace(root.canonicalPath, "<root>")
        .replace(root.absolutePath, "<root>").replace(root.path, "<root>")
    private fun encoded(extension: Extension) = ExtensionJsonConfig.encodeToString(Extension.serializer(), extension)
    private fun compare(zip: File, parent: File, root: File, tape: GoldenTape) {
        val reference = File(parent, "reference").apply { mkdirs() }
        val actual = File(parent, "actual").apply { mkdirs() }
        val unzipped = outcome { ReferenceZipUtils.unzip(zip, reference) }
        tape.answer("unzip", stable(unzipped, root))
        tape.answer("tree", tree(reference).toSortedMap())
        assertEquals(unzipped, outcome { ZipUtils.unzip(zip, actual) })
        assertEquals(tree(reference), tree(actual))
        assertFalse(File(parent, "escape.txt").exists())
    }

    @Test fun generatedArchivesMatchExtractionIncludingTraversalMissingParentsAndDuplicates() {
        val root = File("build/stage2-extensions/archive-tests").apply { deleteRecursively(); mkdirs() }
        val metadata = """{"$":"ime.extension.theme","meta":{"id":"local.fixture","version":"not-semver","title":"Fixture","maintainers":["Jane Doe"],"license":"MIT"},"themes":[{"id":"fixture","label":"Fixture","authors":["Jane Doe"]}]}"""
        val cohorts = listOf(
            listOf("extension.json" to metadata, "stylesheets/" to "", "stylesheets/fixture.json" to "{}"),
            listOf("extension.json" to metadata), // Referenced component files are not checked during import.
            listOf("README" to "Missing manifest"),
            listOf("../escape.txt" to "Traversal", "extension.json" to metadata),
            listOf("a/../../escape.txt" to "Traversal", "extension.json" to metadata),
            listOf("x".repeat(256) to "Too long", "😀".repeat(128) to "Too many UTF-16 units", "extension.json" to metadata),
            listOf("absent/parent.txt" to "Missing explicit parent"),
            listOf("same.txt" to "First", "same.txt" to "Last", "extension.json" to metadata),
            listOf("extension.json" to metadata, "extension.json" to metadata.replace("local.fixture", "local.last")),
            listOf("directory/" to "", "directory/../inside.txt" to "Inside", "literal\\..\\name.txt" to "Backslash"),
        )
        // The reference's extraction also goes on a golden tape (docs/phase6-kotlin-references.md):
        // each tree in path order, each message without the checkout's path, each manifest as encoded.
        val tape = GoldenTape.ofReference("extension-unzips")
        for ((i, entries) in cohorts.withIndex()) {
            val parent = File(root, i.toString()).apply { mkdirs() }
            val zip = File(parent, "fixture.flex")
            zip.writeBytes(zip(entries))
            compare(zip, parent, root, tape)
            val manifest = File(parent, "actual/extension.json")
            if (manifest.exists()) {
                val decoded = runCatching { encoded(ReferenceExtensionPolicy.decode(manifest.readText())) }
                tape.answer("manifest $i", stable(decoded.getOrElse { it.javaClass.name to it.message }, root))
                assertEquals(outcome { ReferenceExtensionPolicy.decode(manifest.readText()) }, outcome { ExtensionCodec.decode(manifest.readText()) })
            }
        }
        // The real ZIP reader reports this generated expanded size, without keeping a 100 MB byte array.
        val large = File(root, "large.flex")
        ZipOutputStream(large.outputStream()).use { out ->
            out.putNextEntry(ZipEntry("large.bin"))
            val block = ByteArray(8192)
            var remaining = 100_000_001
            while (remaining > 0) { val size = minOf(remaining, block.size); out.write(block, 0, size); remaining -= size }
            out.closeEntry()
        }
        val largeDir = File(root, "large").apply { mkdirs() }
        compare(large, largeDir, root, tape)
        tape.finish()
        assertTrue(File(largeDir, "actual/large.bin").exists())
        assertEquals(0L, File(largeDir, "actual/large.bin").length())
    }

    @Test fun seededEntryListsPreserveExactUtf16LengthContainmentSizeAndDuplicateRules() {
        val random = Random(20261113)
        val root = "/tmp/fixture"
        val names = listOf("extension.json", "../escape", "", "a/", "x".repeat(255), "x".repeat(256), "😀".repeat(128), "same", "same")
        val paths = listOf("$root/extension.json", "$root/", root, "${root}2/outside", "/tmp/escape", "$root/" + "x".repeat(1023 - root.length - 1), "$root/" + "x".repeat(1024 - root.length - 1))
        val sizes = listOf(-1L, 0L, 1L, 99_999_999L, 100_000_000L, 100_000_001L, Long.MAX_VALUE)
        repeat(10_000) {
            val entries = List(random.nextInt(12)) { ExtensionPolicy.ArchiveEntry(names.random(random), paths.random(random), sizes.random(random), random.nextBoolean()) }
            val expected = entries.map { entry ->
                when {
                    entry.name.length > 255 -> "skip"
                    entry.canonicalPath.length > 1023 -> "skip"
                    !entry.canonicalPath.startsWith("$root/") -> "skip"
                    entry.directory -> "directory"
                    entry.size > 100000000 -> "empty"
                    else -> "copy"
                }
            }
            assertEquals(entries.map { it.name.length <= 255 }, ExtensionPolicy.archiveNames(entries.map { it.name }))
            assertEquals(expected, ExtensionPolicy.archive(root, entries))
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
