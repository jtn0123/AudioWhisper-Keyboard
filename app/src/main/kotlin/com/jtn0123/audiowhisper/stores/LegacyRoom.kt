package com.jtn0123.audiowhisper.stores

import android.content.Context
import android.database.Cursor
import android.database.DatabaseErrorHandler
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteDatabaseCorruptException
import android.database.sqlite.SQLiteException
import android.util.Log
import com.jtn0123.audiowhisper.rust.ClipFlag
import com.jtn0123.audiowhisper.rust.ClipboardStore
import com.jtn0123.audiowhisper.rust.StoreImport
import com.jtn0123.audiowhisper.rust.StoreOpened
import com.jtn0123.audiowhisper.rust.UserWordStore
import java.io.File
import java.io.IOException

/**
 * The Room databases main kept, read once without Room so their rows move into the Rust stores (crates/stores,
 * docs/phase5-stores.md). Each is opened read-only and read by column name, so every schema main shipped reads:
 * the history at versions 2 to 4, its media files' records at 1 and 2, and the dictionary at 1. Rust takes the
 * rows exactly once and deletes the files only after reading its store back, each database's main file first;
 * until then a crash leaves them to be read again. Only counts are logged, never a row.
 */
internal object LegacyRoom {
    const val HISTORY = "clipboard_history"
    const val FILES = "clipboard_files"
    const val DICTIONARY = "floris_user_dictionary"
    private const val TAG = "AwStores"
    private val SIDECARS = listOf("-wal", "-shm", "-journal")

    /** Leaves a damaged database for this reader to report, rather than deleting it as the framework would. */
    private val KEEP = DatabaseErrorHandler { }

    /** Moves the clipboard databases' rows into [store], if they are still there; what Rust made of them. */
    fun importClipboard(context: Context, store: ClipboardStore): StoreImport? {
        val history = context.getDatabasePath(HISTORY)
        val files = context.getDatabasePath(FILES)
        val paths = listOf(history, files) + sidecars(history) + sidecars(files)
        if (paths.none(File::exists)) return null
        if (store.opened.readOnly) return StoreImport.KEPT
        val clips = read(history, ClipRows()) { clipRows(it) } ?: return null
        val records = read(files, FileRows()) { fileRows(it) } ?: return null
        return saved("clipboard: ${clips.ids.size} clips, ${records.ids.size} files") {
            store.import(clips.ids.toLongArray(), clips.kinds.toIntArray(), clips.created.toLongArray(),
                clips.flags.toIntArray(), clips.texts.toTypedArray(), clips.uris.toTypedArray(),
                clips.mimeTypes.toTypedArray(), records.ids.toLongArray(), records.names.toTypedArray(),
                records.sizes.toLongArray(), records.orientations.toIntArray(), records.mimeTypes.toTypedArray(),
                clips.sequence, paths.map(File::getPath).toTypedArray())
        }
    }

    /** Moves the dictionary database's words into [store], if it is still there; what Rust made of them. */
    fun importDictionary(context: Context, store: UserWordStore): StoreImport? {
        val database = context.getDatabasePath(DICTIONARY)
        val paths = listOf(database) + sidecars(database)
        if (paths.none(File::exists)) return null
        if (store.opened.readOnly) return StoreImport.KEPT
        val words = read(database, WordRows()) { wordRows(it) } ?: return null
        return saved("dictionary: ${words.ids.size} words") {
            store.import(words.ids.toLongArray(), words.words.toTypedArray(), words.frequencies.toIntArray(),
                words.locales.toTypedArray(), words.shortcuts.toTypedArray(), words.sequence,
                paths.map(File::getPath).toTypedArray())
        }
    }

    private fun sidecars(database: File) = SIDECARS.map { File(database.path + it) }

    /**
     * A store this build can't change (a newer build wrote it, or it can't be read) takes no rows, so its databases
     * aren't even opened: an open of a WAL database makes files beside it, read-only or not. Rust refuses the import
     * the same way, so this only spares the disk.
     */
    private val StoreOpened.readOnly get() = this == StoreOpened.NEWER || this == StoreOpened.UNREADABLE

    /**
     * [read] of the database at [file]: [empty] if it isn't there or is damaged (Room would have deleted it), null
     * if it can't be read now, so it stays for the next start. Read-only first; if that fails, read-write, which
     * recovers what an interrupted write left as Room's own open would.
     */
    private fun <T> read(file: File, empty: T, read: (SQLiteDatabase) -> T): T? {
        if (!file.isFile) return empty
        for (mode in listOf(SQLiteDatabase.OPEN_READONLY, SQLiteDatabase.OPEN_READWRITE)) {
            try {
                return SQLiteDatabase.openDatabase(file.path, null, mode or SQLiteDatabase.NO_LOCALIZED_COLLATORS, KEEP)
                    .use(read)
            } catch (e: SQLiteDatabaseCorruptException) {
                Log.w(TAG, "${file.name} is damaged: imported as empty")
                return empty
            } catch (e: SQLiteException) {
                Log.w(TAG, "${file.name} not read: ${e.javaClass.simpleName}")
            } catch (e: IllegalArgumentException) {
                Log.w(TAG, "${file.name} lacks a column: kept")
                return null
            }
        }
        return null
    }

    private fun <T> saved(what: String, import: () -> T): T? = try {
        import().also { Log.i(TAG, "$what: $it") }
    } catch (e: IOException) {
        Log.w(TAG, "$what not imported: ${e.javaClass.simpleName}")
        null
    }

    private fun hasTable(db: SQLiteDatabase, table: String) =
        db.rawQuery("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?", arrayOf(table)).use { it.moveToFirst() }

    /** The table's `AUTOINCREMENT` counter: the largest id it ever handed out. */
    private fun sequence(db: SQLiteDatabase, table: String): Long = if (!hasTable(db, "sqlite_sequence")) 0 else
        db.rawQuery("SELECT seq FROM sqlite_sequence WHERE name = ?", arrayOf(table)).use { if (it.moveToFirst()) it.getLong(0) else 0 }

    /** Every row of [table] in id order, as Room's `SELECT *` gave them, or none if it has no such table. */
    private fun rows(db: SQLiteDatabase, table: String, row: (Cursor) -> Unit) {
        if (!hasTable(db, table)) return
        db.rawQuery("SELECT * FROM $table ORDER BY _id", null).use { cursor -> while (cursor.moveToNext()) row(cursor) }
    }

    private fun Cursor.long(name: String) = getLong(getColumnIndexOrThrow(name))
    private fun Cursor.int(name: String) = getInt(getColumnIndexOrThrow(name))
    private fun Cursor.text(name: String): String = getString(getColumnIndexOrThrow(name))
    private fun Cursor.textOrNull(name: String): String? = getColumnIndexOrThrow(name).let { if (isNull(it)) null else getString(it) }

    /** A flag column under any of the [names] it had across versions; false where a version had none. */
    private fun Cursor.flag(vararg names: String) = names.map(::getColumnIndex).firstOrNull { it >= 0 }?.let { getInt(it) != 0 } == true

    private class ClipRows {
        val ids = ArrayList<Long>(); val kinds = ArrayList<Int>(); val created = ArrayList<Long>(); val flags = ArrayList<Int>()
        val texts = ArrayList<String?>(); val uris = ArrayList<String?>(); val mimeTypes = ArrayList<String>()
        var sequence = 0L
    }

    private fun clipRows(db: SQLiteDatabase) = ClipRows().apply {
        sequence = sequence(db, HISTORY)
        rows(db, HISTORY) { row ->
            ids += row.long("_id"); kinds += row.int("type"); created += row.long("creationTimestampMs")
            val bit = { on: Boolean, flag: ClipFlag -> if (on) flag.code else 0 }
            flags += bit(row.int("isPinned") != 0, ClipFlag.PINNED) or
                bit(row.flag("is_sensitive", "isSensitive"), ClipFlag.SENSITIVE) or
                bit(row.flag("is_remote_device", "isRemoteDevice"), ClipFlag.REMOTE)
            texts += row.textOrNull("text"); uris += row.textOrNull("uri"); mimeTypes += row.text("mimeTypes")
        }
    }

    private class FileRows {
        val ids = ArrayList<Long>(); val names = ArrayList<String>(); val sizes = ArrayList<Long>()
        val orientations = ArrayList<Int>(); val mimeTypes = ArrayList<String>()
    }

    /** The media files' records; version 1 had no orientation, which reads as 0, as an image without one. */
    private fun fileRows(db: SQLiteDatabase) = FileRows().apply {
        rows(db, FILES) { row ->
            ids += row.long("_id"); names += row.text("_display_name"); sizes += row.long("_size")
            orientations += row.getColumnIndex("orientation").let { if (it >= 0) row.getInt(it) else 0 }
            mimeTypes += row.text("mimeTypes")
        }
    }

    private class WordRows {
        val ids = ArrayList<Long>(); val words = ArrayList<String>(); val frequencies = ArrayList<Int>()
        val locales = ArrayList<String?>(); val shortcuts = ArrayList<String?>()
        var sequence = 0L
    }

    private fun wordRows(db: SQLiteDatabase) = WordRows().apply {
        sequence = sequence(db, "words")
        rows(db, "words") { row ->
            ids += row.long("_id"); words += row.text("word"); frequencies += row.int("frequency")
            locales += row.textOrNull("locale"); shortcuts += row.textOrNull("shortcut")
        }
    }
}
