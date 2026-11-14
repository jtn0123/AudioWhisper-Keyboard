package dev.patrickgold.florisboard.app.settings.advanced

import com.jtn0123.audiowhisper.rust.BackupPolicy
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

class BackupPlanMatchTest {
    private val prefs = "jetpref/florisboard-app-prefs.json"
    @Test fun allSelectionsModesAndIncomingExistingCombinations() {
        for (mask in 0..63) {
            assertEquals(ReferenceBackupPolicy.manifest(mask, prefs), BackupPolicy.manifest(mask, prefs))
            assertEquals(ReferenceBackupPolicy.selection(mask), BackupPolicy.selection(mask))
            for (bits in 0..63) {
                val present = List(6) { bits and (1 shl it) != 0 }
                for (erase in listOf(false, true)) assertEquals(
                    ReferenceBackupPolicy.restore(mask, erase, prefs, present), BackupPolicy.restore(mask, erase, prefs, present))
            }
        }
        // Existing clipboard matches compare all fields except the database id; repeated incoming
        // entries are compared only with the original snapshot, exactly as ClipboardManager did.
        for (bits in 0..255) {
            val existing = List(8) { bits and (1 shl it) != 0 }
            assertEquals(ReferenceBackupPolicy.incoming(existing), BackupPolicy.incoming(existing))
        }
        val random = Random(1819)
        repeat(10_000) {
            val existing = List(random.nextInt(40)) { random.nextBoolean() }
            assertEquals(ReferenceBackupPolicy.incoming(existing), BackupPolicy.incoming(existing))
            val types = List(random.nextInt(40)) { listOf("TEXT", "IMAGE", "VIDEO", "OTHER").random(random) }
            for (type in listOf("TEXT", "IMAGE", "VIDEO")) assertEquals(
                types.indices.filter { types[it] == type }, BackupPolicy.items(type, types))
        }
    }
    @Test fun versionVendorMessagesAndNamesMatchIncludingBoundaryVersions() {
        val versions = listOf(Int.MIN_VALUE, 0, 63, 64, 65, 100, Int.MAX_VALUE)
        val packages = listOf("", " ", "\u00a0", "dev.patrickgold.florisboard", "dev.patrickgold.florisboard.debug", "com.jtn0123.audiowhisper.keyboard")
        for (code in versions) for (current in versions) for (pkg in packages) {
            assertEquals(ReferenceBackupPolicy.metadata(pkg, code, current), BackupPolicy.metadata(pkg, code, current))
            for (time in listOf(Long.MIN_VALUE, 0, Long.MAX_VALUE)) assertEquals(
                ReferenceBackupPolicy.fileName(pkg, code, time), BackupPolicy.fileName(pkg, code, time))
        }
        assertEquals("Invalid archive: either backup_metadata.json is missing or file is not a ZIP archive.", BackupPolicy.missingMetadata)
    }
}
