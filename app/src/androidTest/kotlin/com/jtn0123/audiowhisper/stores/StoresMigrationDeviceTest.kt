package com.jtn0123.audiowhisper.stores

import android.content.Context
import android.content.ContextWrapper
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.jtn0123.audiowhisper.rust.ClipboardStore
import com.jtn0123.audiowhisper.rust.StoreImport
import com.jtn0123.audiowhisper.rust.StoreOpened
import com.jtn0123.audiowhisper.rust.UserWordStore
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.zip.CRC32
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Main's Room databases into the Rust stores, on Android's own SQLite: every schema main shipped, from fixtures main's
 * build wrote (assets/stores, scripts/stores-fixtures), must import to what main's DAOs read from them, and leave no
 * database file behind. Then the state a crash leaves at each step of the import: before any file went, between
 * the two clipboard databases, with only their WAL files left, mid-save of the store. None may lose a row or import
 * one twice. A damaged database imports as empty, as Room would have reset it; a store from a newer build keeps the
 * databases. Rows are compared by hash only.
 */
@RunWith(AndroidJUnit4::class)
class StoresMigrationDeviceTest {
    private val target = InstrumentationRegistry.getInstrumentation().targetContext
    private val assets = InstrumentationRegistry.getInstrumentation().context.assets
    private val dir = File(target.cacheDir, "stores-migration")
    private val databases = File(dir, "databases")
    private val app: Context = object : ContextWrapper(target) {
        override fun getDatabasePath(name: String) = File(databases, name)
    }

    @Before fun setUp() = reset()

    @After fun tearDown() {
        dir.deleteRecursively()
    }

    private fun reset() {
        dir.deleteRecursively()
        databases.mkdirs()
    }

    @Test fun everyHistorySchemaImportsAsMainReadIt() {
        val pairs = listOf("history-v4" to "files-v2", "history-v3" to "files-v2", "history-v2" to "files-v1", "history-v4-wal" to null)
        for ((history, files) in pairs) {
            reset()
            place(history, LegacyRoom.HISTORY)
            files?.let { place(it, LegacyRoom.FILES) }
            assertEquals(history, StoreImport.ROWS, LegacyRoom.importClipboard(app, clips()))
            assertEquals("$history: database files left", emptyList<String>(), left())
            val reopened = clips()
            checkHistory(reopened, history)
            files?.let { checkFiles(reopened, it) }
            val next = reopened.add(1, 0, 0, "aw-next", null, "text/plain", 0, false, 0)
            assertEquals("$history: the next id", expected(history).getLong("sequence") + 1, next)
        }
    }

    @Test fun anEmptyHistoryImportsNothingAndIsDeleted() {
        place("history-v4-empty", LegacyRoom.HISTORY)
        val store = clips()
        assertEquals(StoreImport.NOTHING, LegacyRoom.importClipboard(app, store))
        assertEquals(0, store.items().size)
        assertEquals(emptyList<String>(), left())
        assertEquals(null, LegacyRoom.importClipboard(app, store))
    }

    @Test fun theDictionaryImportsAsMainReadIt() {
        place("words-v1", LegacyRoom.DICTIONARY)
        assertEquals(StoreImport.ROWS, LegacyRoom.importDictionary(app, words()))
        assertEquals(emptyList<String>(), left())
        val reopened = words()
        val expected = expected("words-v1")
        val all = reopened.all().map(::wordKey)
        assertEquals(expected.getInt("count"), all.size)
        assertEquals(expected.getString("byId"), digest(all))
        assertEquals(expected.getLong("sequence") + 1, reopened.insert(0, "awnext", 1, null, null))
    }

    /** Died after the store was saved, before a database was deleted: the next start finds them all again. */
    @Test fun aCrashBeforeAnyDatabaseWentImportsNothingTwice() {
        placeClipboard()
        assertEquals(StoreImport.ROWS, LegacyRoom.importClipboard(app, clips()))
        placeClipboard()
        val again = clips()
        assertEquals(StoreImport.ALREADY, LegacyRoom.importClipboard(app, again))
        assertEquals(emptyList<String>(), left())
        checkHistory(clips(), "history-v4")
        checkFiles(clips(), "files-v2")

        place("words-v1", LegacyRoom.DICTIONARY)
        assertEquals(StoreImport.ROWS, LegacyRoom.importDictionary(app, words()))
        place("words-v1", LegacyRoom.DICTIONARY)
        assertEquals(StoreImport.ALREADY, LegacyRoom.importDictionary(app, words()))
        assertEquals(expected("words-v1").getString("byId"), digest(words().all().map(::wordKey)))
    }

    /** Died after the history's main file went: the media records and the history's WAL are left. */
    @Test fun aCrashBetweenTheDatabasesLeavesNoDuplicate() {
        placeClipboard()
        assertEquals(StoreImport.ROWS, LegacyRoom.importClipboard(app, clips()))
        place("files-v2", LegacyRoom.FILES)
        sidecars("history-v4-wal", LegacyRoom.HISTORY)
        assertEquals(StoreImport.ROWS, LegacyRoom.importClipboard(app, clips()))
        assertEquals(emptyList<String>(), left())
        checkHistory(clips(), "history-v4")
        checkFiles(clips(), "files-v2")
    }

    /** Died after both main files went: only WAL files are left, which hold nothing without them. */
    @Test fun aCrashWithOnlyTheWalLeftImportsNothing() {
        placeClipboard()
        assertEquals(StoreImport.ROWS, LegacyRoom.importClipboard(app, clips()))
        sidecars("history-v4-wal", LegacyRoom.HISTORY)
        sidecars("history-v4-wal", LegacyRoom.FILES)
        assertEquals(StoreImport.NOTHING, LegacyRoom.importClipboard(app, clips()))
        assertEquals(emptyList<String>(), left())
        checkHistory(clips(), "history-v4")
    }

    /** Died while saving the store: its temporary file is left beside it, and the import runs again from the start. */
    @Test fun aCrashMidSaveImportsOnTheNextStart() {
        placeClipboard()
        val store = File(dir, "clipboard.store")
        File(store.path + ".tmp").writeBytes(ByteArray(64) { it.toByte() })
        assertEquals(StoreImport.ROWS, LegacyRoom.importClipboard(app, clips()))
        assertEquals(false, File(store.path + ".tmp").exists())
        checkHistory(clips(), "history-v4")
    }

    @Test fun aDamagedDatabaseImportsAsEmptyAndIsDeleted() {
        File(databases, LegacyRoom.HISTORY).writeBytes(ByteArray(8_192) { (it * 31 + 7).toByte() })
        place("files-v2", LegacyRoom.FILES)
        val store = clips()
        assertEquals(StoreImport.ROWS, LegacyRoom.importClipboard(app, store))
        assertEquals(0, store.items().size)
        checkFiles(store, "files-v2")
        assertEquals(emptyList<String>(), left())
    }

    @Test fun aStoreFromANewerBuildKeepsTheDatabases() {
        File(dir, "clipboard.store").writeBytes(newer("CLIP"))
        File(dir, "words.store").writeBytes(newer("WORD"))
        placeClipboard()
        place("words-v1", LegacyRoom.DICTIONARY)
        val placed = left()
        val store = clips()
        assertEquals(StoreOpened.NEWER, store.opened)
        assertEquals(StoreImport.KEPT, LegacyRoom.importClipboard(app, store))
        assertEquals(StoreImport.KEPT, LegacyRoom.importDictionary(app, words()))
        assertEquals(placed, left())
    }

    private fun clips() = ClipboardStore(File(dir, "clipboard.store").path)
    private fun words() = UserWordStore(File(dir, "words.store").path)

    private fun placeClipboard() {
        place("history-v4", LegacyRoom.HISTORY)
        place("files-v2", LegacyRoom.FILES)
    }

    /** The fixture's database and whichever of its WAL files it has, as the database [name]. */
    private fun place(fixture: String, name: String) {
        copy("$fixture.db", name)
        sidecars(fixture, name)
    }

    private fun sidecars(fixture: String, name: String) {
        for (suffix in listOf("-wal", "-shm")) if ("$fixture.db$suffix" in listing) copy("$fixture.db$suffix", name + suffix)
    }

    private fun copy(asset: String, name: String) =
        assets.open("stores/$asset").use { input -> File(databases, name).outputStream().use { input.copyTo(it) } }

    private val listing by lazy { checkNotNull(assets.list("stores")).toSet() }

    private fun left() = databases.list().orEmpty().sorted()

    private fun expected(name: String) = JSONObject(assets.open("stores/$name.expected.json").reader().readText())

    private fun checkHistory(store: ClipboardStore, fixture: String) {
        val expected = expected(fixture)
        val items = store.items()
        assertEquals("$fixture: clips", expected.getInt("count"), items.size)
        assertEquals("$fixture: clips by id", expected.getString("byId"), digest(items.sortedBy { it.id }.map(::clipKey)))
        assertEquals("$fixture: the history's order", expected.getString("visible"), digest(items.map(::clipKey)))
    }

    /** The media records, looked up by every id near the fixture's, as the content provider looks them up. */
    private fun checkFiles(store: ClipboardStore, fixture: String) {
        val expected = expected(fixture)
        val records = (95L..110L).flatMap { store.fileRecords(it) }
            .map { key(it.id, it.name, it.size, it.orientation, it.mimeTypes) }
        assertEquals("$fixture: records", expected.getInt("count"), records.size)
        assertEquals("$fixture: records by id", expected.getString("byId"), digest(records))
    }

    /** An empty store of [kind] framed as version 2, which this build can't read. */
    private fun newer(kind: String): ByteArray {
        val body = ByteBuffer.allocate(20).order(ByteOrder.LITTLE_ENDIAN)
            .put("AWST".toByteArray()).put(kind.toByteArray()).putInt(2).putLong(0).array()
        val crc = CRC32().apply { update(body) }.value.toInt()
        return body + ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN).putInt(crc).array()
    }
}
