package com.jtn0123.audiowhisper.stores

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.jtn0123.audiowhisper.rust.ClipboardStore
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.restoreItems
import dev.patrickgold.florisboard.ime.clipboard.provider.toItem
import java.io.File
import org.florisboard.lib.kotlin.io.readJson
import org.florisboard.lib.kotlin.io.writeJson
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The clipboard backup both ways. assets/stores/backup-clips-*.json are the files main's BackupIO wrote for each kind
 * (scripts/stores-fixtures), beside what main read back from them. This build must read each into the same clips
 * and write those clips back byte for byte: main's backups restore here, and since main reads these bytes as its
 * own, this build's backups restore on main. Then the restore itself: the clips go into the Rust store through the
 * mapping ClipboardManager.restoreHistory uses, and come back out as the history shows them.
 */
@RunWith(AndroidJUnit4::class)
class StoresBackupDeviceTest {
    private val target = InstrumentationRegistry.getInstrumentation().targetContext
    private val assets = InstrumentationRegistry.getInstrumentation().context.assets
    private val dir = File(target.cacheDir, "stores-backup").apply { deleteRecursively(); mkdirs() }

    @After fun tearDown() {
        dir.deleteRecursively()
    }

    @Test fun mainsBackupsReadAndWriteByteForByte() {
        val expected = JSONObject(assets.open("stores/backup-clips.expected.json").reader().readText())
        var clips = 0
        for (kind in listOf("TEXT", "IMAGE", "VIDEO")) {
            val main = File(dir, "main-$kind.json").apply { writeBytes(assets.open("stores/backup-clips-$kind.json").readBytes()) }
            val items = main.readJson<List<ClipboardItem>>()
            val read = expected.getJSONObject(kind)
            assertEquals("$kind: clips", read.getInt("count"), items.size)
            assertEquals("$kind: as main read them", read.getString("all"), digest(items.map(::itemKey)))
            val written = File(dir, "branch-$kind.json").apply { writeJson(items) }
            assertArrayEquals("$kind: bytes", main.readBytes(), written.readBytes())
            clips += items.size
        }
        assertEquals(13, clips)
    }

    @Test fun aRestoredBackupShowsEveryClipOnce() {
        val items = listOf("TEXT", "IMAGE", "VIDEO").flatMap { kind ->
            File(dir, "$kind.json").apply { writeBytes(assets.open("stores/backup-clips-$kind.json").readBytes()) }
                .readJson<List<ClipboardItem>>()
        }
        val store = ClipboardStore(File(dir, "clipboard.store").path)
        assertEquals(items.size, store.restoreItems(items, 0, false, 0))
        assertEquals(0, store.restoreItems(items, 0, false, 0))
        val shown = store.items().mapNotNull { it.toItem() }.map { itemKey(it.copy(id = 0)) }
        val expected = items.sortedByDescending { it.creationTimestampMs }.map { itemKey(it.copy(id = 0)) }
        assertEquals(digest(expected), digest(shown))
    }

    private fun itemKey(c: ClipboardItem) = key(c.id, c.type.value, c.text, c.uri?.toString(), c.creationTimestampMs,
        if (c.isPinned) 1 else 0, c.mimeTypes.joinToString(","), if (c.isSensitive) 1 else 0, if (c.isRemoteDevice) 1 else 0)
}
