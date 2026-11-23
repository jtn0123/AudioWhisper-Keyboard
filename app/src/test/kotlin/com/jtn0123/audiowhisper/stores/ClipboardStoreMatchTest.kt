package com.jtn0123.audiowhisper.stores

import com.jtn0123.audiowhisper.rust.ClipFlag
import com.jtn0123.audiowhisper.rust.ClipboardStore
import com.jtn0123.audiowhisper.rust.StoreOpened
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.long

/**
 * The clipboard store against main's Room-backed ClipboardManager: clipboard-match.json holds 40 seeded sequences of
 * 30 operations (adds with duplicate texts and tied times, pins, deletes, clears, restores, expiry and limit
 * changes) that main's StoresFixtureWriter ran on main's real Room DAO the way main's ClipboardManager drove it,
 * hashing the visible history after each. The Rust store must show the same history, ids included, after every
 * step, and so must a store reopened from its file every seventh step, as after the process died.
 */
class ClipboardStoreMatchTest {
    @Test fun everyStepShowsTheHistoryMainShowed() {
        val sequences = golden("clipboard-match.json").array("sequences")
        assertEquals(40, sequences.size)
        val steps = inTempDir { dir -> sequences.sumOf { Replay(dir, it.obj).run() } }
        assertEquals(1_200, steps)
    }
}

private class Replay(dir: File, private val sequence: JsonObject) {
    private val seed = sequence.int("seed")
    private val path = File(dir, "clips-$seed.store").path
    private var store = ClipboardStore(path)
    private var limitEnabled = sequence.bool("limitEnabled")
    private var limit = sequence.int("limit")

    fun run(): Int {
        assertEquals(StoreOpened.MISSING, store.opened)
        val steps = sequence.array("steps").map { it.obj }
        steps.forEachIndexed { i, step ->
            apply(step, step.long("now"))
            if (i % 7 == 6) store = ClipboardStore(path).also { assertEquals(StoreOpened.LOADED, it.opened) }
            val visible = store.items().map(::clipKey)
            val where = "seed $seed step $i (${step.string("op")})"
            assertEquals(step.int("rows"), visible.size, "$where: rows")
            assertEquals(step.string("hash"), digest(visible), "$where: history")
        }
        return steps.size
    }

    private fun apply(step: JsonObject, now: Long) {
        when (step.string("op")) {
            "add" -> step.obj("item").let {
                store.add(it.int("type"), it.long("created"), flags(it), it.stringOrNull("text"), it.stringOrNull("uri"),
                    it.string("mime"), now, limitEnabled, limit)
            }
            "pin" -> store.pin(step.long("id"), step.bool("pinned"), now, limitEnabled, limit)
            "delete" -> store.delete(step.array("ids").map { it.jsonPrimitive.long }.toLongArray(),
                step.bool("onlyIfUnpinned"), now, limitEnabled, limit)
            "clear" -> store.clear(step.bool("unpinnedOnly"), now, limitEnabled, limit)
            "restore" -> restore(step.array("items").map { it.obj }, now)
            "expire" -> store.expire(now, step.bool("old"), step.int("minutes"), step.bool("sensitive"),
                step.int("seconds"), limitEnabled, limit)
            "limit" -> {
                limitEnabled = step.bool("enabled")
                limit = step.int("limit")
                store.enforce(now, limitEnabled, limit)
            }
            else -> error("seed $seed: unknown operation")
        }
    }

    private fun restore(items: List<JsonObject>, now: Long) {
        store.restore(items.map { it.int("type") }.toIntArray(), items.map { it.long("created") }.toLongArray(),
            items.map(::flags).toIntArray(), items.map { it.stringOrNull("text") }.toTypedArray(),
            items.map { it.stringOrNull("uri") }.toTypedArray(), items.map { it.string("mime") }.toTypedArray(),
            now, limitEnabled, limit)
    }

    private fun flags(item: JsonObject) = (if (item.bool("pinned")) ClipFlag.PINNED.code else 0) or
        (if (item.bool("sensitive")) ClipFlag.SENSITIVE.code else 0) or (if (item.bool("remote")) ClipFlag.REMOTE.code else 0)
}
