package com.jtn0123.audiowhisper.stores

import android.content.Context
import android.database.sqlite.SQLiteConstraintException
import androidx.room.Room
import com.jtn0123.audiowhisper.rust.BackupPolicy
import dev.patrickgold.florisboard.ime.clipboard.clipboardPlan
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardHistoryDao
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardHistoryDatabase
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.ItemType
import dev.patrickgold.florisboard.ime.dictionary.FlorisUserDictionaryDatabase
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryDao
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryEntry
import dev.patrickgold.florisboard.lib.FlorisLocale
import java.io.File
import java.util.Random
import org.json.JSONArray
import org.json.JSONObject

/**
 * The match goldens: seeded operation sequences run on main's real Room DAOs, driven as main's ClipboardManager
 * and settings screen drove them, with the visible history (or every word) hashed after each step. The branch
 * replays each sequence on the Rust stores and must reach every hash (docs/phase5-stores.md).
 */
internal class MatchWriter(private val context: Context, private val out: File) {
    fun write() {
        File(out, "clipboard-match.json").writeText(JSONObject().put("source", StoresFixtureWriter.SOURCE)
            .put("sequences", JSONArray((1..CLIP_SEQUENCES).map(::clipboard))).toString() + "\n")
        File(out, "dictionary-match.json").writeText(JSONObject().put("source", StoresFixtureWriter.SOURCE)
            .put("sequences", JSONArray((1..WORD_SEQUENCES).map(::dictionary))).toString() + "\n")
    }

    private fun clipboard(seed: Int): JSONObject {
        val random = Random(seed.toLong())
        val file = context.getDatabasePath("awfx-match").also { gone(it) }
        val db = Room.databaseBuilder(context, ClipboardHistoryDatabase::class.java, file.name)
            .fallbackToDestructiveMigration(dropAllTables = false).build()
        try {
            val main = MainClipboard(db.clipboardItemDao(), random.nextInt(4) != 0, 2 + random.nextInt(7))
            val sequence = JSONObject().put("seed", seed).put("limitEnabled", main.limitEnabled).put("limit", main.limit)
            var now = 1_767_225_600_000L + seed * 10_000_000L
            val steps = JSONArray()
            repeat(CLIP_STEPS) {
                now += random.nextInt(90_000)
                val step = main.step(random, now)
                steps.put(step.put("rows", main.visible.size).put("hash", digest(main.visible.map(::clipKey))))
            }
            return sequence.put("steps", steps)
        } finally { db.close(); gone(file) }
    }

    private fun dictionary(seed: Int): JSONObject {
        val random = Random(1_000L + seed)
        val file = context.getDatabasePath("awfx-words-match").also { gone(it) }
        val db = Room.databaseBuilder(context, FlorisUserDictionaryDatabase::class.java, file.name)
            .allowMainThreadQueries().build()
        try {
            val main = MainWords(db, db.userDictionaryDao())
            val steps = JSONArray()
            repeat(WORD_STEPS) {
                val step = main.step(random)
                val all = db.userDictionaryDao().queryAll()
                steps.put(step.put("rows", all.size).put("hash", digest(all.map(::wordKey))))
            }
            return JSONObject().put("seed", seed).put("steps", steps)
        } finally { db.close(); gone(file) }
    }

    private fun gone(file: File) = listOf("", "-wal", "-shm", "-journal").forEach { File(file.path + it).delete() }

    companion object {
        const val CLIP_SEQUENCES = 40
        const val CLIP_STEPS = 30
        const val WORD_SEQUENCES = 24
        const val WORD_STEPS = 25
    }
}

/** Main's ClipboardManager over its Room DAO, every coroutine it launched run to the end before the next step. */
private class MainClipboard(val dao: ClipboardHistoryDao, var limitEnabled: Boolean, var limit: Int) {
    var visible: List<ClipboardItem> = emptyList()

    /** What main's LiveData observer did on every change: sort, then delete past the limit, until it holds. */
    fun settle(now: Long) {
        while (true) {
            val all = dao.getAll()
            visible = clipboardPlan(all, now = now)[0].map(all::get)
            val drop = clipboardPlan(visible, now = now, limitEnabled = limitEnabled, limit = limit)[4]
            if (drop.isEmpty()) return
            dao.delete(drop.map(visible::get))
        }
    }

    fun step(random: Random, now: Long): JSONObject {
        val op = JSONObject().put("now", now)
        when (random.nextInt(100)) {
            in 0..44 -> add(op, item(random, now))
            in 45..55 -> pin(op, random)
            in 56..67 -> deleteOne(op, random)
            in 68..72 -> clearExact(op, random)
            in 73..76 -> clear(op, random.nextInt(4) != 0)
            in 77..85 -> restore(op, random, now)
            in 86..95 -> expire(op, random, now)
            else -> {
                limitEnabled = random.nextInt(4) != 0
                limit = 2 + random.nextInt(7)
                op.put("op", "limit").put("enabled", limitEnabled).put("limit", limit)
            }
        }
        settle(now)
        return op
    }

    /** insertOrMoveBeginning. */
    private fun add(op: JSONObject, item: ClipboardItem) {
        op.put("op", "add").put("item", json(item))
        val plan = clipboardPlan(visible, now = op.getLong("now"), newItem = item)[6]
        val previous = visible.getOrNull(plan[0])
        if (previous == null) dao.insert(item)
        else {
            dao.delete(previous.id)
            dao.insert(item.copy(isPinned = plan[1] != 0))
        }
    }

    private fun target(random: Random): Long? =
        if (visible.isEmpty() || random.nextInt(10) == 0) null else visible[random.nextInt(visible.size)].id

    /** pinClip and unpinClip. */
    private fun pin(op: JSONObject, random: Random) {
        val id = target(random) ?: MISSING
        val pinned = visible.firstOrNull { it.id == id }?.let { random.nextInt(10) < 7 != it.isPinned } ?: random.nextBoolean()
        op.put("op", "pin").put("id", id).put("pinned", pinned)
        visible.firstOrNull { it.id == id }?.let { dao.update(it.copy(isPinned = pinned)) }
    }

    /** deleteClip. */
    private fun deleteOne(op: JSONObject, random: Random) {
        val id = target(random) ?: MISSING
        val onlyIfUnpinned = random.nextBoolean()
        op.put("op", "delete").put("ids", JSONArray(listOf(id))).put("onlyIfUnpinned", onlyIfUnpinned)
        if (onlyIfUnpinned) dao.deleteIfUnpinned(id) else dao.delete(id)
    }

    /** clearExactHistory. */
    private fun clearExact(op: JSONObject, random: Random) {
        val items = visible.shuffled(random).take(1 + random.nextInt(3))
        op.put("op", "delete").put("ids", JSONArray(items.map { it.id })).put("onlyIfUnpinned", false)
        dao.delete(items)
    }

    /** clearHistory and clearFullHistory. */
    private fun clear(op: JSONObject, unpinnedOnly: Boolean) {
        op.put("op", "clear").put("unpinnedOnly", unpinnedOnly)
        if (unpinnedOnly) dao.deleteAllUnpinned() else dao.deleteAll()
    }

    /** restoreHistory. */
    private fun restore(op: JSONObject, random: Random, now: Long) {
        val items = ArrayList<ClipboardItem>()
        repeat(1 + random.nextInt(4)) {
            items += when {
                visible.isNotEmpty() && random.nextInt(3) == 0 -> visible[random.nextInt(visible.size)].copy(id = 77)
                items.isNotEmpty() && random.nextInt(6) == 0 -> items[random.nextInt(items.size)]
                else -> item(random, now)
            }
        }
        op.put("op", "restore").put("items", JSONArray(items.map(::json)))
        val existing = visible.map { it.copy(id = 0) }.toSet()
        val indices = BackupPolicy.incoming(items.map { it.copy(id = 0) in existing })
        for (index in indices) dao.insert(items[index].copy(id = 0))
    }

    /** enforceExpiryDate. */
    private fun expire(op: JSONObject, random: Random, now: Long) {
        val old = random.nextBoolean()
        val minutes = 1 + random.nextInt(20)
        val sensitive = random.nextBoolean()
        val seconds = 5 + random.nextInt(600)
        op.put("op", "expire").put("old", old).put("minutes", minutes).put("sensitive", sensitive).put("seconds", seconds)
        val indices = clipboardPlan(visible, now = now, old = old, minutes = minutes, sensitive = sensitive, seconds = seconds)[5]
        if (indices.isNotEmpty()) dao.delete(indices.map(visible::get))
    }

    private fun item(random: Random, now: Long): ClipboardItem {
        val created = when (random.nextInt(8)) {
            0 -> now
            1 -> now - 299_999
            2 -> now - 300_000
            3 -> now - 300_001
            4 -> visible.randomOrNull(random)?.creationTimestampMs ?: now
            else -> now - random.nextInt(1_500_000)
        }
        val flags = Triple(random.nextInt(10) == 0, random.nextInt(5) == 0, random.nextInt(10) == 0)
        val kind = random.nextInt(20)
        if (kind >= 15) {
            val type = if (kind < 18) ItemType.IMAGE else ItemType.VIDEO
            val mime = listOf(if (type == ItemType.IMAGE) "image/png" else "video/mp4")
            return StoresFixtureWriter.media(type, 1L + random.nextInt(6), created, mime, flags.first)
                .copy(isSensitive = flags.second, isRemoteDevice = flags.third)
        }
        val text = "aw-m-${random.nextInt(8)}" + if (random.nextInt(6) == 0) "-😀" else ""
        val mime = if (random.nextInt(5) == 0) listOf("text/html", "text/plain") else listOf("text/plain")
        return ClipboardItem(0, ItemType.TEXT, text, null, created, flags.first, mime, flags.second, flags.third)
    }

    private fun <T> List<T>.randomOrNull(random: Random) = if (isEmpty()) null else this[random.nextInt(size)]

    private fun json(item: ClipboardItem) = JSONObject().put("type", item.type.value)
        .put("text", item.text ?: JSONObject.NULL).put("uri", item.uri?.toString() ?: JSONObject.NULL)
        .put("created", item.creationTimestampMs).put("pinned", item.isPinned).put("sensitive", item.isSensitive)
        .put("remote", item.isRemoteDevice).put("mime", item.mimeTypes.joinToString(","))

    companion object {
        const val MISSING = 9_999L
    }
}

/** Main's settings screen over the Room dictionary: inserts, updates, deletes, exact lookups, transactions. */
private class MainWords(val db: FlorisUserDictionaryDatabase, val dao: UserDictionaryDao) {
    private class Abort : RuntimeException()

    fun step(random: Random): JSONObject = when (random.nextInt(100)) {
        in 0..69 -> change(random, JSONObject()).let { (op, run) -> op.put("outcome", attempt(run)) }
        in 70..84 -> exact(random)
        else -> transaction(random)
    }

    private fun attempt(run: () -> Unit): String = try { run(); "ok" } catch (e: SQLiteConstraintException) { "taken" }

    /** One change, recorded before it runs. */
    private fun change(random: Random, op: JSONObject): Pair<JSONObject, () -> Unit> {
        val ids = dao.queryAll().map { it.id }
        fun known() = if (ids.isEmpty() || random.nextInt(8) == 0) 999L else ids[random.nextInt(ids.size)]
        return when (random.nextInt(10)) {
            in 0..4 -> {
                val id = if (random.nextInt(5) < 3) 0L else if (ids.isNotEmpty() && random.nextInt(3) == 0) known() else 1L + random.nextInt(30)
                val entry = entry(random, id)
                op.put("op", "insert").put("entry", json(entry)) to { dao.insert(entry) }
            }
            in 5..7 -> {
                val entry = entry(random, known())
                op.put("op", "update").put("entry", json(entry)) to { dao.update(entry) }
            }
            else -> {
                val entry = entry(random, known())
                op.put("op", "delete").put("entry", json(entry)) to { dao.delete(entry) }
            }
        }
    }

    private fun exact(random: Random): JSONObject {
        val word = "aw-w-${random.nextInt(6)}"
        val locale = LOCALES[random.nextInt(LOCALES.size)]
        val found = dao.queryExact(word, locale?.let(FlorisLocale::fromTag))
        return JSONObject().put("op", "exact").put("word", word).put("locale", locale ?: JSONObject.NULL)
            .put("found", found.size).put("result", digest(found.map(::wordKey)))
    }

    /** runInTransaction around a few changes, the last throwing if [fail], or a change that does. */
    private fun transaction(random: Random): JSONObject {
        val fail = random.nextInt(3) == 0
        val changes = JSONArray()
        val outcome = try {
            db.runInTransaction {
                repeat(1 + random.nextInt(3)) {
                    val (op, run) = change(random, JSONObject())
                    changes.put(op)
                    run()
                }
                if (fail) throw Abort()
            }
            "committed"
        } catch (e: Abort) { "rolled back" } catch (e: SQLiteConstraintException) { "rolled back" }
        return JSONObject().put("op", "transaction").put("changes", changes).put("fail", fail).put("outcome", outcome)
    }

    private fun entry(random: Random, id: Long) = UserDictionaryEntry(id, "aw-w-${random.nextInt(6)}",
        random.nextInt(256), LOCALES[random.nextInt(LOCALES.size)], SHORTCUTS[random.nextInt(SHORTCUTS.size)])

    private fun json(entry: UserDictionaryEntry) = JSONObject().put("id", entry.id).put("word", entry.word)
        .put("freq", entry.freq).put("locale", entry.locale ?: JSONObject.NULL).put("shortcut", entry.shortcut ?: JSONObject.NULL)

    companion object {
        val LOCALES = listOf(null, "en-US", "de-DE", "fr")
        val SHORTCUTS = listOf(null, null, "awsc0", "awsc1")
    }
}
