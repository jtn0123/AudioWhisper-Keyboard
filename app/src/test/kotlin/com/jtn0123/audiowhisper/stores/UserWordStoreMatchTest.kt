package com.jtn0123.audiowhisper.stores

import dev.patrickgold.florisboard.ime.dictionary.FlorisUserDictionaryDatabase
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryDao
import dev.patrickgold.florisboard.ime.dictionary.UserDictionaryEntry
import dev.patrickgold.florisboard.lib.FlorisLocale
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlinx.serialization.json.JsonObject

/**
 * The keyboard's own dictionary against main's Room database: dictionary-match.json holds 24 seeded sequences of 25
 * operations (inserts with new, chosen and taken ids, updates and deletes of present and missing words, exact
 * lookups, and transactions that commit, fail on a taken id or are aborted) that main's StoresFixtureWriter ran on
 * main's real Room DAO, hashing every word after each. [FlorisUserDictionaryDatabase] over the Rust store must give
 * the same words, ids included, the same outcomes and lookups, and so must one reopened from its file every seventh
 * step.
 */
class UserWordStoreMatchTest {
    @Test fun everyStepLeavesTheWordsMainLeft() {
        val sequences = golden("dictionary-match.json").array("sequences")
        assertEquals(24, sequences.size)
        val steps = inTempDir { dir -> sequences.sumOf { WordReplay(dir, it.obj).run() } }
        assertEquals(600, steps)
    }
}

private class WordReplay(dir: File, private val sequence: JsonObject) {
    private class Abort : RuntimeException()

    private val seed = sequence.int("seed")
    private val path = File(dir, "words-$seed.store").path
    private var db = FlorisUserDictionaryDatabase.at(path)
    private val dao: UserDictionaryDao get() = db.userDictionaryDao()

    fun run(): Int {
        val steps = sequence.array("steps").map { it.obj }
        steps.forEachIndexed { i, step ->
            val where = "seed $seed step $i (${step.string("op")})"
            apply(step, where)
            if (i % 7 == 6) db = FlorisUserDictionaryDatabase.at(path)
            val all = dao.queryAll().map(::wordKey)
            assertEquals(step.int("rows"), all.size, "$where: rows")
            assertEquals(step.string("hash"), digest(all), "$where: words")
        }
        return steps.size
    }

    private fun apply(step: JsonObject, where: String) {
        when (step.string("op")) {
            "exact" -> {
                val found = dao.queryExact(step.string("word"), step.stringOrNull("locale")?.let(FlorisLocale::fromTag))
                assertEquals(step.int("found"), found.size, "$where: found")
                assertEquals(step.string("result"), digest(found.map(::wordKey)), "$where: lookup")
            }
            "transaction" -> assertEquals(step.string("outcome"), transaction(step), where)
            else -> assertEquals(step.string("outcome"), attempt { change(step) }, where)
        }
    }

    private fun transaction(step: JsonObject): String = try {
        db.runInTransaction {
            step.array("changes").forEach { change(it.obj) }
            if (step.bool("fail")) throw Abort()
        }
        "committed"
    } catch (e: Abort) {
        "rolled back"
    } catch (e: IllegalStateException) {
        "rolled back"
    }

    private fun attempt(run: () -> Unit): String = try {
        run()
        "ok"
    } catch (e: IllegalStateException) {
        "taken"
    }

    private fun change(op: JsonObject) {
        val entry = op.obj("entry").let {
            UserDictionaryEntry(it.long("id"), it.string("word"), it.int("freq"), it.stringOrNull("locale"), it.stringOrNull("shortcut"))
        }
        when (op.string("op")) {
            "insert" -> dao.insert(entry)
            "update" -> dao.update(entry)
            "delete" -> dao.delete(entry)
            else -> error("seed $seed: unknown change")
        }
    }

    private fun wordKey(w: UserDictionaryEntry) = key(w.id, w.word, w.freq, w.locale, w.shortcut)
}
