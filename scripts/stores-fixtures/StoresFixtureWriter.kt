package com.jtn0123.audiowhisper.stores

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import androidx.core.net.toUri
import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.jtn0123.audiowhisper.rust.BackupPolicy
import dev.patrickgold.florisboard.ime.clipboard.clipboardPlan
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardFileInfo
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardFilesDatabase
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardHistoryDao
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardHistoryDatabase
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.ItemType
import dev.patrickgold.florisboard.ime.dictionary.FlorisUserDictionaryDatabase
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryEntry
import java.io.File
import java.security.MessageDigest
import org.florisboard.lib.kotlin.io.readJson
import org.florisboard.lib.kotlin.io.writeJson
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The stores' migration fixtures, written by main's own Room code on main's build (48b2e0018) and kept on the
 * all-rust/stores branch as test data (docs/phase5-stores.md). Every schema main shipped: the history at 2, 3
 * and 4 (4 also mid-write, its rows still in the WAL), its media files' records at 1 and 2, the dictionary at
 * 1. Beside each database, what main's DAOs read from it, as SHA-256 hashes of canonical rows, so the fixtures
 * hold synthetic text only in the databases themselves. Versions Room created are made by Room; older ones from
 * the schema JSON Room exported for them, as Room would have made them.
 */
@RunWith(AndroidJUnit4::class)
class StoresFixtureWriter {
    private val context: Context = InstrumentationRegistry.getInstrumentation().targetContext
    private val assets = InstrumentationRegistry.getInstrumentation().context.assets
    private val out = File(context.filesDir, "stores-fixtures")

    @Test fun write() {
        out.deleteRecursively()
        out.mkdirs()
        for (schema in 2..4) history(schema)
        historyWal()
        historyEmpty()
        files(2)
        files(1)
        words()
        backup()
        MatchWriter(context, out).write()
    }

    private fun history(schema: Int) {
        val file = scratch("history-v$schema")
        if (schema == 4) withHistory(file) { fill(it.clipboardItemDao()) }
        else raw(file, "ClipboardHistoryDatabase", schema) { fillRaw(it, schema) }
        settled(file)
        expectHistory("history-v$schema", file, schema)
        keep(file, "history-v$schema.db")
    }

    /** Copied while Room still has it open, its rows in the WAL, as a crash would leave it. */
    private fun historyWal() {
        val file = scratch("history-v4-wal")
        withHistory(file) { db ->
            fill(db.clipboardItemDao())
            check(File(file.path + "-wal").length() > 0) { "the WAL is empty" }
            for (suffix in listOf("", "-wal", "-shm")) File(file.path + suffix).copyTo(File(out, "history-v4-wal.db$suffix"))
        }
        delete(file)
        expectHistory("history-v4-wal", File(out, "history-v4-wal.db"), 4)
    }

    private fun historyEmpty() {
        val file = scratch("history-v4-empty")
        withHistory(file) { it.openHelper.writableDatabase }
        settled(file)
        expectHistory("history-v4-empty", file, 4)
        keep(file, "history-v4-empty.db")
    }

    private fun files(schema: Int) {
        val file = scratch("files-v$schema")
        if (schema == 2) {
            val db = Room.databaseBuilder(context, ClipboardFilesDatabase::class.java, file.name)
                .fallbackToDestructiveMigration(dropAllTables = false).build()
            try {
                val dao = db.clipboardFilesDao()
                FILES.forEach { dao.insert(it) }
                dao.delete(FILES.last().id)
            } finally { db.close() }
        } else raw(file, "ClipboardFilesDatabase", 1) { db ->
            FILES.forEach { db.insertOrThrow("clipboard_files", null, ContentValues().apply {
                put("_id", it.id); put("_display_name", it.displayName); put("_size", it.size)
                put("mimeTypes", it.mimeTypes.joinToString(","))
            }) }
            db.delete("clipboard_files", "_id = ?", arrayOf(FILES.last().id.toString()))
        }
        settled(file)
        val rows = SQLiteDatabase.openDatabase(file.path, null, SQLiteDatabase.OPEN_READONLY).use { db ->
            db.rawQuery("SELECT * FROM clipboard_files ORDER BY _id", null).use { c ->
                generateSequence { if (c.moveToNext()) c else null }.map {
                    val orientation = c.getColumnIndex("orientation").let { i -> if (i >= 0) c.getInt(i) else 0 }
                    key(c.getLong(c.getColumnIndexOrThrow("_id")), c.getString(c.getColumnIndexOrThrow("_display_name")),
                        c.getLong(c.getColumnIndexOrThrow("_size")), orientation, c.getString(c.getColumnIndexOrThrow("mimeTypes")))
                }.toList()
            }
        }
        val main = copied(file) { copy ->
            val db = Room.databaseBuilder(context, ClipboardFilesDatabase::class.java, copy.name)
                .fallbackToDestructiveMigration(dropAllTables = false).build()
            try { db.clipboardFilesDao().getAll().sortedBy { it.id }.map(::fileKey) } finally { db.close() }
        }
        record("files-v$schema", JSONObject().put("schema", schema).put("count", rows.size)
            .put("byId", digest(rows)).put("rows", JSONArray(rows.map(::sha)))
            .put("mainCount", main.size).put("mainById", digest(main)))
        keep(file, "files-v$schema.db")
    }

    private fun words() {
        val file = scratch("words-v1")
        val db = Room.databaseBuilder(context, FlorisUserDictionaryDatabase::class.java, file.name)
            .allowMainThreadQueries().build()
        try {
            val dao = db.userDictionaryDao()
            WORDS.forEach { dao.insert(it) }
            val all = dao.queryAll()
            for (gone in WORDS_GONE) dao.delete(all.first { it.word == gone })
        } finally { db.close() }
        settled(file)
        val sequence = SQLiteDatabase.openDatabase(file.path, null, SQLiteDatabase.OPEN_READONLY).use { sequence(it, "words") }
        val all = copied(file) { copy ->
            val room = Room.databaseBuilder(context, FlorisUserDictionaryDatabase::class.java, copy.name)
                .allowMainThreadQueries().build()
            try { room.userDictionaryDao().queryAll() } finally { room.close() }
        }.sortedBy { it.id }.map(::wordKey)
        record("words-v1", JSONObject().put("schema", 1).put("count", all.size).put("sequence", sequence)
            .put("byId", digest(all)).put("rows", JSONArray(all.map(::sha))))
        keep(file, "words-v1.db")
    }

    /** The clipboard backup's JSON files as main's BackupIO writes them, and what main reads back from each. */
    private fun backup() {
        val history = CLIPS.mapIndexed { i, clip -> clip.copy(id = i + 1L) }
        val expected = JSONObject()
        for (kind in listOf("TEXT", "IMAGE", "VIDEO")) {
            val items = BackupPolicy.items(kind, history.map { it.type.name }).map { history[it] }
            val file = File(out, "backup-clips-$kind.json")
            file.writeJson(items)
            val read = file.readJson<List<ClipboardItem>>().map(::clipKey)
            expected.put(kind, JSONObject().put("count", read.size).put("all", digest(read)).put("rows", JSONArray(read.map(::sha))))
        }
        record("backup-clips", expected)
    }

    private fun withHistory(file: File, block: (ClipboardHistoryDatabase) -> Unit) {
        val db = Room.databaseBuilder(context, ClipboardHistoryDatabase::class.java, file.name)
            .fallbackToDestructiveMigration(dropAllTables = false).build()
        try { block(db) } finally { db.close() }
    }

    private fun fill(dao: ClipboardHistoryDao) {
        val ids = CLIPS.map { dao.insert(it) }
        GONE.forEach { dao.delete(ids[it]) }
    }

    private fun fillRaw(db: SQLiteDatabase, schema: Int) {
        val ids = CLIPS.map { clip ->
            db.insertOrThrow("clipboard_history", null, ContentValues().apply {
                put("type", clip.type.value); put("text", clip.text); put("uri", clip.uri?.toString())
                put("creationTimestampMs", clip.creationTimestampMs); put("isPinned", clip.isPinned.bit)
                put("mimeTypes", clip.mimeTypes.joinToString(","))
                if (schema == 3) { put("isSensitive", clip.isSensitive.bit); put("isRemoteDevice", clip.isRemoteDevice.bit) }
            })
        }
        GONE.forEach { db.delete("clipboard_history", "_id = ?", arrayOf(ids[it].toString())) }
    }

    /** A database Room made at [schema], from the schema JSON it exported: its tables, indices and identity. */
    private fun raw(file: File, database: String, schema: Int, fill: (SQLiteDatabase) -> Unit) {
        val json = JSONObject(assets.open("stores-schemas/$database/$schema.json").bufferedReader().readText())
            .getJSONObject("database")
        val flags = SQLiteDatabase.CREATE_IF_NECESSARY or SQLiteDatabase.ENABLE_WRITE_AHEAD_LOGGING
        SQLiteDatabase.openDatabase(file.path, null, flags).use { db ->
            val entity = json.getJSONArray("entities").getJSONObject(0)
            val table = entity.getString("tableName")
            val sql = listOf(entity.getString("createSql")) + entity.getJSONArray("indices").let { indices ->
                (0 until indices.length()).map { indices.getJSONObject(it).getString("createSql") }
            }
            sql.forEach { db.execSQL(it.replace("\${TABLE_NAME}", table)) }
            json.getJSONArray("setupQueries").let { setup -> (0 until setup.length()).forEach { db.execSQL(setup.getString(it)) } }
            db.version = schema
            fill(db)
        }
    }

    /** What main's Room reads from the history at [source], opening a copy as main's [ClipboardHistoryDatabase.new]
     * did, migrations and all. */
    private fun expectHistory(name: String, source: File, schema: Int) {
        val (raw, room) = copied(source) { copy ->
            val sequence = SQLiteDatabase.openDatabase(copy.path, null, SQLiteDatabase.OPEN_READWRITE)
                .use { sequence(it, "clipboard_history") }
            var read: Pair<List<ClipboardItem>, Long> = emptyList<ClipboardItem>() to 0L
            withHistory(copy) { db ->
                val all = db.clipboardItemDao().getAll()
                read = all to db.openHelper.readableDatabase.query("SELECT seq FROM sqlite_sequence WHERE name = 'clipboard_history'")
                    .use { if (it.moveToFirst()) it.getLong(0) else 0L }
            }
            sequence to read
        }
        val (all, roomSequence) = room
        val byId = all.sortedBy { it.id }.map(::clipKey)
        val visible = clipboardPlan(all)[0].map(all::get).map(::clipKey)
        record(name, JSONObject().put("schema", schema).put("count", all.size).put("sequence", raw)
            .put("roomSequence", roomSequence).put("byId", digest(byId)).put("visible", digest(visible))
            .put("rows", JSONArray(byId.map(::sha))))
    }

    private fun <T> copied(source: File, read: (File) -> T): T {
        val copy = scratch("expect")
        for (suffix in SUFFIXES) File(source.path + suffix).takeIf { it.exists() }?.copyTo(File(copy.path + suffix))
        return try { read(copy) } finally { delete(copy) }
    }

    private fun sequence(db: SQLiteDatabase, table: String): Long =
        db.rawQuery("SELECT seq FROM sqlite_sequence WHERE name = ?", arrayOf(table)).use { if (it.moveToFirst()) it.getLong(0) else 0L }

    /** Closed databases are a single file: their WAL was checkpointed away. */
    private fun settled(file: File) = check(File(file.path + "-wal").let { !it.exists() || it.length() == 0L }) { "${file.name} kept a WAL" }

    private fun record(name: String, json: JSONObject) =
        File(out, "$name.expected.json").writeText(json.put("source", SOURCE).toString(2) + "\n")

    private fun scratch(name: String) = context.getDatabasePath("awfx-$name").also(::delete)
    private fun keep(file: File, name: String) { file.copyTo(File(out, name)); delete(file) }
    private fun delete(file: File) = SUFFIXES.forEach { File(file.path + it).delete() }

    companion object {
        const val SOURCE = "main 48b2e0018, StoresFixtureWriter"
        private val SUFFIXES = listOf("", "-wal", "-shm", "-journal")
        private const val T0 = 1_767_225_600_000L
        const val MEDIA = "content://com.jtn0123.audiowhisper.keyboard.provider.clipboard/clips"

        private fun text(text: String, at: Long, pinned: Boolean = false, sensitive: Boolean = false,
            remote: Boolean = false, mime: List<String> = listOf("text/plain")) =
            ClipboardItem(0, ItemType.TEXT, text, null, at, pinned, mime, sensitive, remote)

        fun media(type: ItemType, file: Long, at: Long, mime: List<String>, pinned: Boolean = false) = ClipboardItem(
            0, type, null, "$MEDIA/${if (type == ItemType.IMAGE) "images" else "videos"}/$file".toUri(), at, pinned, mime)

        /** The history every fixture holds, oldest row first; [GONE] are deleted again. */
        val CLIPS = listOf(
            text("aw-fixture-first", T0),
            text("aw-fixture-second, with a comma", T0 + 1_000, pinned = true),
            text("aw-fixture-tie-😀", T0 + 1_000),
            text("aw-fixture-line\nbreak", T0 + 5_000, sensitive = true),
            text("", T0 + 6_000),
            media(ItemType.IMAGE, 101, T0 + 7_000, listOf("image/png")),
            text("aw-fixture-gap", T0 + 7_500),
            media(ItemType.VIDEO, 102, T0 + 8_000, listOf("video/mp4")),
            media(ItemType.IMAGE, 103, T0 + 9_000, listOf("image/png", "image/jpeg"), pinned = true),
            text("aw-fixture-no-mime", T0 + 10_000, mime = emptyList()),
            text("aw-fixture-remote", T0 + 11_000, pinned = true, remote = true),
            text("aw-fixture-üñï-中文", T0 + 12_000, sensitive = true, remote = true),
            text("aw-fixture-last", T0 + 13_000),
        )
        /** A gap in the ids, and the largest id handed out gone, so the counter is past every row. */
        private val GONE = listOf(6, 12)

        private val FILES = listOf(
            ClipboardFileInfo(101, "aw-fixture-image.png", 2_048, 90, listOf("image/png")),
            ClipboardFileInfo(102, "aw-fixture-clip.mp4", 40_960, 0, listOf("video/mp4")),
            ClipboardFileInfo(103, "aw-fixture-photo.jpg", 4_096, 270, listOf("image/png", "image/jpeg")),
            ClipboardFileInfo(104, "aw-fixture-orphan.png", 10, 180, listOf("image/png")),
            ClipboardFileInfo(105, "aw-fixture-gone.png", 11, 0, listOf("image/png")),
        )

        private val WORDS = listOf(
            UserDictionaryEntry(0, "awfixtureone", 255, null, null),
            UserDictionaryEntry(0, "awfixturetwo", 1, "en-US", "awsc"),
            UserDictionaryEntry(0, "awfixtureü", 128, "de-DE", null),
            UserDictionaryEntry(0, "awfixture😀", 0, "fr", "awsc2"),
            UserDictionaryEntry(0, "awfixturegap", 10, "en-US", null),
            UserDictionaryEntry(40, "awfixtureexplicit", 50, null, "awsc3"),
            UserDictionaryEntry(0, "awfixturetwo", 2, "de-DE", null),
            UserDictionaryEntry(0, "awfixturelast", 7, null, null),
        )
        private val WORDS_GONE = listOf("awfixturegap", "awfixturelast")
    }
}

internal val Boolean.bit get() = if (this) 1 else 0

/** Fields as `length:text`, `~` for none, joined by `|`: one row, unambiguously. */
internal fun key(vararg fields: Any?) = fields.joinToString("|") { if (it == null) "~" else it.toString().let { s -> "${s.length}:$s" } }

internal fun clipKey(c: ClipboardItem) = key(c.id, c.type.value, c.text, c.uri?.toString(), c.creationTimestampMs,
    c.isPinned.bit, c.mimeTypes.joinToString(","), c.isSensitive.bit, c.isRemoteDevice.bit)

internal fun fileKey(f: ClipboardFileInfo) = key(f.id, f.displayName, f.size, f.orientation, f.mimeTypes.joinToString(","))
internal fun wordKey(w: UserDictionaryEntry) = key(w.id, w.word, w.freq, w.locale, w.shortcut)

internal fun sha(text: String): String = MessageDigest.getInstance("SHA-256").digest(text.toByteArray(Charsets.UTF_8))
    .joinToString("") { "%02x".format(it) }

internal fun digest(rows: List<String>) = sha(rows.joinToString("\n"))
