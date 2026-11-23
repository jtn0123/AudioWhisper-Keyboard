package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * The GIF panel's session, previews and send steps through JNI, as the keyboard calls them:
 * plans in and out as strings, rows of GIFs, generations, and bad input thrown. What each
 * decides is tested in Rust and, against main's Kotlin, by the app's GifSearchTraceTest.
 */
class GifSessionPlanTest {
    private fun row(id: String) = arrayOf(id, "title", "p/$id", 1.5f.toRawBits().toString(), "s/$id", "u/$id")

    private fun Array<String>.effect(code: String): Int = indexOf(code).also { assertTrue(it >= GifPlan.SIZE, code) }

    @Test
    fun aSearchIsFetchedOnceAndKept() {
        GifSessionPlan().use { plan ->
            plan.call(GifOp.SOURCE, 0, WireFlag.encode(true), "KEY")
            val load = plan.call(GifOp.LOAD, 0, "cats")
            assertEquals("cats", load[GifPlan.SHOWN])
            val fetch = load.effect(GifEffect.FETCH)
            assertTrue(load[fetch + 2].startsWith("https://api.giphy.com/v1/gifs/search?q=cats&"), "search URL")
            val request = GifSessionPlan.request(GifMedia.API).map(Int::toString)
            assertEquals(request, load.slice(fetch + 3 until fetch + 3 + GifRequest.SIZE))
            val fetched = plan.call(GifOp.FETCHED, 1, load[fetch + 1], GifReply.OK, "200", """{"data":[]}""")
            assertEquals(GifResults.LOADED.toString(), fetched[fetched.effect(GifEffect.RESULTS) + 1])
            plan.call(GifOp.LOAD, 2, "dogs")
            val again = plan.call(GifOp.LOAD, 3, "Cats ")
            assertFalse(GifEffect.FETCH in again, "kept under the same key")
            assertEquals(GifResults.LOADED.toString(), again[again.effect(GifEffect.RESULTS) + 1])
        }
    }

    @Test
    fun recentsAreReadMergedAndSaved() {
        GifSessionPlan().use { plan ->
            val attach = plan.call(GifOp.ATTACH, 0, "files/gif-recents.json")
            val read = attach.effect(GifEffect.READ)
            val generation = attach[read + 1].toInt()
            assertEquals("files/gif-recents.json", attach[read + 2])
            plan.read(generation, true, row("a"))
            val merged = plan.call(GifOp.MERGED, 0)
            assertEquals(GifRules.recents, merged[merged.effect(GifEffect.CHIPS) + 2], "Recents leads the chips")
            val sent = plan.call(GifOp.SENT, 0, WireFlag.encode(false), *row("b"))
            val rows = plan.rows(sent[sent.effect(GifEffect.SAVE) + 1].toInt())
            assertEquals("files/gif-recents.json", rows[0])
            assertEquals(listOf("b", "a"), listOf(rows[1 + GifRow.ID], rows[1 + GifRow.SIZE + GifRow.ID]))
            assertEquals(1.5f, Float.fromBits(rows[1 + GifRow.RATIO].toInt()))
            assertContentEquals(emptyArray(), plan.rows(generation + 1), "a stale generation saves nothing")
            assertFailsWith<IllegalArgumentException> { plan.read(generation, true, arrayOf("half a row")) }
        }
    }

    @Test
    fun aClosedSessionThrows() {
        val plan = GifSessionPlan()
        plan.close()
        assertFailsWith<IllegalArgumentException> { plan.call(GifOp.SNAPSHOT, 0) }
    }

    @Test
    fun sendingIsStepped() {
        assertEquals(listOf(GifSendAction.PRUNE), GifSessionPlan.sendNext(GifSendStep.START, true, true).toList())
        assertEquals(listOf(GifSendAction.TYPE_LINK), GifSessionPlan.sendNext(GifSendStep.START, false, true).toList())
        assertEquals(listOf(GifSendAction.FAILED), GifSessionPlan.sendNext(GifSendStep.START, false, false).toList())
        assertEquals(listOf(GifSendAction.TOUCH, GifSendAction.COMMIT), GifSessionPlan.sendNext(GifSendStep.CHECKED, true, true).toList())
        assertEquals(listOf("abc.gif", "abc.gif.part"), GifSessionPlan.sendFiles("a/b-c").toList())
        val names = arrayOf("new.gif", "old.gif", "cut.gif.part")
        assertContentEquals(intArrayOf(1, 2), GifSessionPlan.sendStale(names, longArrayOf(1_000, 0, 0), 86_400_001))
        assertFailsWith<IllegalArgumentException> { GifSessionPlan.sendStale(names, longArrayOf(0), 0) }
        assertFailsWith<IllegalArgumentException> { GifSessionPlan.request(3) }
    }

    @Test
    fun previewsKeepTheMostRecentlyUsed() {
        val keys = GifPreviewKeys()
        assertEquals(emptyList(), keys.put("a", 10 * MB))
        assertEquals(emptyList(), keys.put("b", 10 * MB))
        assertTrue(keys.get("a"))
        assertEquals(listOf("b"), keys.put("c", 10 * MB))
        assertFalse(keys.get("b"))
        assertEquals(listOf("a", "c", "d"), keys.put("d", 25 * MB), "one past the limit drops even itself")
        keys.put("e", 1)
        keys.clear()
        assertFalse(keys.get("e"))
        assertFailsWith<IllegalArgumentException> { keys.put("f", -1) }
    }

    private companion object {
        const val MB = 1024 * 1024
    }
}
