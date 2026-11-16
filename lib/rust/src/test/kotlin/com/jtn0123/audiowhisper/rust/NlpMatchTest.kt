package com.jtn0123.audiowhisper.rust

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class NlpMatchTest {
    internal class Pairing : AutoCloseable {
        val rust = NlpSession()
        val reference = ReferenceNlp()
        var checks = 0
        fun call(op: String, vararg f: String): List<String> {
            val expected = reference.call(op, *f)
            val actual = rust.call(op, *f).toList()
            assertEquals(expected, actual, "NLP $op at checkpoint $checks")
            checks++
            assertEquals(reference.call("snapshot"), rust.call("snapshot").toList(), "state after $op")
            assertEquals(reference.call("words"), rust.call("words").toList(), "Smartbar words after $op")
            return actual
        }
        override fun close() = rust.close()
    }
    @Test fun seededSessionsMatchFiftyThousandEvents() {
        var events = 0
        val reached = mutableSetOf<Int>()
        repeat(200) { seed ->
            val random = Random(20261113 + seed)
            Pairing().use { p ->
                var context = ReferenceNlp.context()
                p.call("context", *context.toTypedArray())
                p.call("previewStart", "0")
                val pending = mutableListOf<String>()
                val inflations = mutableListOf<String>()
                val clipboardRequests = mutableListOf<Pair<String, List<String>>>()
                repeat(250) { step ->
                    val kind = random.nextInt(16); reached += kind
                    when (kind) {
                        0, 1, 2 -> {
                            val word = listOf("h", "hel", "helo", "teh", "zorblax", ":smile", "a\uD800", "")[random.nextInt(8)]
                            context = context.toMutableList().also { c ->
                                c[10] = word.length.toString(); c[11] = c[10]; c[13] = c[10]; c[14] = word
                            }
                            p.call("context", *context.toTypedArray())
                            pending += p.call("begin", (step / 2).toString())[0] // equal timestamps, faster than debounce
                        }
                        3 -> if (pending.isNotEmpty()) {
                            val gen = pending.removeAt(random.nextInt(pending.size))
                            p.call("ready", gen, (step / 2 + 10).toString())
                            val values = if (random.nextInt(5) == 0) emptyArray() else arrayOf("a$step", "help", "0", "1", "b$step", "help", "0", "0", "c$step", "helo", "0", "0", "e$step", "😀", "1", "0")
                            p.call("complete", gen, *values)
                        }
                        4 -> {
                            context = context.toMutableList().also { it[10] = random.nextInt(5).toString(); it[11] = it[10] }
                            p.call("context", *context.toTypedArray())
                        }
                        5 -> {
                            context = context.toMutableList().also { it[0] = listOf(1, 0x81, 0x91, 0xe1, 0x12, 0x21, 0x11, 0x80001, 0x8021, 0x8011, 0x80b1)[random.nextInt(11)].toString() }
                            p.call("context", *context.toTypedArray()); p.call("policy", *context.toTypedArray())
                        }
                        6 -> {
                            context = context.toMutableList().also { it[7] = random.nextInt(2).toString(); it[4] = random.nextInt(2).toString(); it[5] = random.nextInt(2).toString(); it[16] = if (random.nextBoolean()) ":" else "" }
                            p.call("context", *context.toTypedArray())
                        }
                        7 -> { inflations += p.call("inlineBegin", random.nextInt(4).toString())[0] }
                        8 -> if (inflations.isNotEmpty()) p.call("inlineComplete", inflations.removeAt(random.nextInt(inflations.size)), "unpin", "0", "pin1", "1", "pin2", "1")
                        9 -> {
                            p.call("auto", listOf(-1, 32, 46, 39, 64, 0x1d7ce)[random.nextInt(6)].toString(), random.nextInt(2).toString())
                            p.call("preview", (step * 7).toString(), random.nextInt(2).toString(), random.nextInt(200).toString())
                            p.call("glidePlan", random.nextInt(9).toString(), random.nextInt(1, 9).toString(), random.nextInt(2).toString())
                        }
                        10 -> {
                            p.call("learn", random.nextInt(3).toString(), listOf(-1, 39, 46)[random.nextInt(3)].toString(), "zorblax", "a ", "2", random.nextInt(2).toString(), "a")
                            p.call("keep", "zorblax"); p.call("save", (step * 97L).toString())
                        }
                        11 -> p.call("clear")
                        12 -> p.call("direct", "direct", "zorblax", "0", "0")
                        13 -> { p.call("clipboard", "clip", "synthetic", "2", "0"); p.call("pick", "direct") }
                        14 -> clipboardRequests.add(p.call("clipboardBeginFor", *context.toTypedArray())[0] to context.toList())
                        15 -> if (clipboardRequests.isNotEmpty()) {
                            val (gen, requested) = clipboardRequests.removeAt(random.nextInt(clipboardRequests.size))
                            p.call("clipboardFor", *requested.toTypedArray(), *context.toTypedArray(), gen, "clip$step", "synthetic", "2", "0")
                        }
                    }
                    events++
                }
            }
        }
        assertEquals(50_000, events); assertEquals((0..15).toSet(), reached)
        println("NLP: 50000 events, 200 seeds, zero differences")
    }
    @Test fun outOfOrderResultsNeverApplyOrBecomePickable() {
        Pairing().use { p ->
            p.call("context", *ReferenceNlp.context("h").toTypedArray())
            val external = p.call("externalBegin")[0]
            val old = p.call("begin", "100")[0]
            p.call("context", *ReferenceNlp.context("hel").toTypedArray())
            assertEquals(listOf("0"), p.call("externalValid", external))
            val current = p.call("begin", "100")[0]
            assertEquals(listOf("0"), p.call("complete", old, "stale", "hat", "0", "1"))
            assertEquals(listOf("1"), p.call("complete", current, "fresh", "help", "0", "1"))
            assertEquals(listOf("0"), p.call("complete", old, "stale", "hat", "0", "1"))
            assertEquals(listOf("0"), p.call("pick", "stale"))
            assertEquals(listOf("1"), p.call("pick", "fresh"))
            val inline = p.call("inlineBegin", "1")[0]
            p.call("clear")
            // Cursor/field/privacy changes invalidate inline inflation too.
            p.call("context", *ReferenceNlp.context("hel", 0x81).toTypedArray())
            assertEquals(listOf("0"), p.call("inlineComplete", inline, "secret", "1"))
            assertEquals(listOf("0"), p.call("auto", "-1", "0"))
        }
    }
    @Test fun candidateOrderingTypedEntryAndSharedSmartbarRules() {
        Pairing().use { p ->
            p.call("context", *ReferenceNlp.context().toTypedArray())
            p.call("direct", "1", "help", "0", "1", "2", "helo", "0", "0", "3", "help", "0", "0", "4", "😀", "1", "0")
            assertEquals(listOf("2", "0", "1", "3", "1", "2", "4", "0"), p.call("snapshot"))
            val gen = p.call("inlineBegin", "3")[0]
            p.call("inlineComplete", gen, "a", "0", "b", "1", "c", "1")
            assertEquals(listOf("b", "c", "a"), p.call("snapshot").takeLast(3))
            p.call("inlineBegin", "0")
            assertEquals("0", p.call("snapshot")[1]) // candidates still prevent expansion
            val selection = ReferenceNlp.context().toMutableList().also { it[11] = "0" }
            p.call("context", *selection.toTypedArray())
            assertEquals("1", p.call("snapshot")[1])
        }
    }
    @Test fun platformAutofillInPasswordDoesNotEnableWordCandidates() {
        Pairing().use { p ->
            p.call("context", *ReferenceNlp.context("", 0x81).toTypedArray())
            assertEquals(listOf("0", "0", "0", "0", "0", "1"), p.call("policy", *ReferenceNlp.context("", 0x81).toTypedArray()))
            val inline = p.call("inlineBegin", "1")
            assertEquals("1", inline[1])
            p.call("inlineComplete", inline[0], "platform", "1")
            assertEquals("0", p.call("snapshot")[3])
            assertEquals("platform", p.call("snapshot").last())
        }
    }
    @Test fun rawFlagsTypingAndClipboardMatchOriginalDecisions() {
        Pairing().use { p ->
            val random = Random(16)
            repeat(5000) {
                val context = ReferenceNlp.context(input = random.nextInt()).toMutableList().also { it[3] = if (random.nextBoolean()) "en" else "fr" }
                p.call("policy", *context.toTypedArray())
                p.call("candidatePlan", *context.toTypedArray(), random.nextInt(2).toString(), "1", "0", "1")
                p.call("typingPolicy", *context.toTypedArray(), "hello", listOf("", "abc", ":smile", "😀")[random.nextInt(4)], "x")
                p.call("clipboardPlan", random.nextInt(2).toString(), "1", random.nextInt(2).toString(), if (random.nextBoolean()) "" else "a", "aw@example.test", random.nextInt(1001).toString(), "1000", random.nextInt(2).toString(), "example.test", "3", "14", "aw@example.test", "0", "14", "test", "11", "14")
            }
        }
    }
    @Test fun productionBulkMessagesCheckTheCurrentEditorBeforePublishing() {
        Pairing().use { p ->
            val old = ReferenceNlp.context("Aa")
            val newer = ReferenceNlp.context("BB")
            assertEquals("Aa".hashCode(), "BB".hashCode())
            val a = p.call("beginFor", *old.toTypedArray(), "100")[0]
            val b = p.call("beginFor", *newer.toTypedArray(), "100")[0]
            assertEquals("0", p.call("completeFor", *newer.toTypedArray(), a, "stale", "hat", "0", "1")[0])
            assertEquals("0", p.call("completeFor", *old.toTypedArray(), a, "stale-metadata", "hat", "0", "1")[0])
            assertEquals(b, p.call("snapshot")[0], "Old completion metadata must not cancel the new request")
            assertEquals("1", p.call("completeFor", *newer.toTypedArray(), b, "fresh", "book", "0", "1")[0])
            assertEquals("fresh", p.call("snapshot")[4])
        }
    }
    @Test fun freshSynchronousAutocorrectGetsAOneShotNativeIdentity() {
        Pairing().use { p ->
            val current = ReferenceNlp.context("teh")
            val changed = ReferenceNlp.context("else")
            p.call("context", *current.toTypedArray())
            assertEquals(listOf("1"), p.call("autoCandidate", *current.toTypedArray(), *current.toTypedArray(), "fresh-auto", "the", "0", "1"))
            p.call("begin", "100") // same editor, a background strip refresh cannot cancel a fresh synchronous result
            assertEquals(listOf("1"), p.call("pick", "fresh-auto"))
            assertEquals(listOf("0"), p.call("pick", "fresh-auto"))
            assertEquals(listOf("0"), p.call("autoCandidate", *current.toTypedArray(), *changed.toTypedArray(), "stale-auto", "the", "0", "1"))
            assertEquals(listOf("0"), p.call("pick", "stale-auto"))
        }
    }
    @Test fun clipboardAndSynchronousResultsCannotRollBackANewerRequest() {
        Pairing().use { p ->
            val old = ReferenceNlp.context("old")
            val current = ReferenceNlp.context("current")
            val oldClip = p.call("clipboardBeginFor", *old.toTypedArray())[0]
            p.call("beginFor", *old.toTypedArray(), "100")
            val fresh = p.call("beginFor", *current.toTypedArray(), "101")[0]
            val before = p.call("snapshot")
            assertEquals(before, p.call("clipboardFor", *old.toTypedArray(), *old.toTypedArray(), oldClip, "old-clip", "clip", "2", "0"))
            assertEquals(before, p.call("clipboardFor", *old.toTypedArray(), *current.toTypedArray(), oldClip, "changed-clip", "clip", "2", "0"))
            assertEquals(listOf("0"), p.call("autoCandidate", *old.toTypedArray(), *old.toTypedArray(), "old-auto", "older", "0", "1"))
            assertEquals(before, p.call("snapshot"), "Obsolete result metadata cannot change context or cancel pending work")
            assertEquals(listOf("1"), p.call("ready", fresh, "109"))
            assertEquals("1", p.call("completeFor", *current.toTypedArray(), fresh, "new", "current", "0", "0")[0])
            val clip1 = p.call("clipboardBeginFor", *current.toTypedArray())[0]
            val clip2 = p.call("clipboardBeginFor", *current.toTypedArray())[0]
            p.call("clipboardFor", *current.toTypedArray(), *current.toTypedArray(), clip2, "fresh-clip", "clip", "2", "0")
            val shown = p.call("snapshot")
            assertEquals(shown, p.call("clipboardFor", *current.toTypedArray(), *current.toTypedArray(), clip1, "stale-same-editor", "clip", "2", "0"))
            assertEquals("fresh-clip", p.call("snapshot")[4])
        }
    }
    @Test fun pendingRequestsHideStaleWordsWithoutFlashingExpandedActions() {
        Pairing().use { p ->
            p.call("context", *ReferenceNlp.context().toTypedArray())
            p.call("direct", "shown", "help", "0", "0")
            assertEquals("0", p.call("snapshot")[1])
            val generation = p.call("begin", "100")[0]
            assertEquals("0", p.call("snapshot")[3])
            assertEquals("-1", p.call("snapshot")[1], "Pending leaves the existing expansion binding unchanged")
            p.call("complete", generation)
            assertEquals("1", p.call("snapshot")[1], "An empty completed response expands actions")
        }
    }
    @Test fun typingHoldsTheSmartbarWordsAcrossTheRefillOnly() {
        Pairing().use { p ->
            fun words() = p.call("words").single()
            p.call("context", *ReferenceNlp.context("h").toTypedArray())
            p.call("begin", "0")
            assertEquals("0", words(), "An empty row holds nothing while its first request is in flight")
            p.call("direct", "help", "help", "0", "0")
            assertEquals("1", words())
            // A collector's configure lands before the keystroke's beginFor: typing-only change.
            p.call("context", *ReferenceNlp.context("he").toTypedArray())
            assertEquals(listOf("1", "-1"), listOf(words(), p.call("snapshot")[1]), "Carry holds the row and the expansion")
            val generation = p.call("beginFor", *ReferenceNlp.context("he").toTypedArray(), "10")[0]
            assertEquals(listOf("1", "0"), listOf(words(), p.call("snapshot")[3]), "The begin gap holds an empty list")
            p.call("complete", generation, "hello", "hello", "0", "0")
            assertEquals("1", words(), "The refill shows words")
            val empty = p.call("beginFor", *ReferenceNlp.context("hex").toTypedArray(), "20")[0]
            assertEquals("1", words())
            p.call("complete", empty)
            assertEquals(listOf("0", "1"), listOf(words(), p.call("snapshot")[1]), "An empty refill ends the hold")
            // Changes that are not typing never hold: incognito, a selection, a cleared row.
            p.call("direct", "help", "help", "0", "0")
            p.call("context", *ReferenceNlp.context("hex", 0x81).toTypedArray())
            assertEquals("0", words(), "Incognito does not hold")
            p.call("context", *ReferenceNlp.context("hex").toTypedArray()); p.call("direct", "help", "help", "0", "0")
            p.call("context", *ReferenceNlp.context("hex").toMutableList().also { it[10] = "0" }.toTypedArray())
            assertEquals("0", words(), "A selection does not hold")
            p.call("context", *ReferenceNlp.context("hex").toTypedArray()); p.call("direct", "help", "help", "0", "0")
            p.call("clear")
            assertEquals("0", words(), "Clear does not hold")
        }
    }
    @Test fun glidePreviewClockAndCommitSliceMatchTheFrozenManager() {
        Pairing().use { p ->
            p.call("previewStart", "100")
            assertEquals(listOf("0"), p.call("preview", "250", "1", "150"))
            assertEquals(listOf("1"), p.call("preview", "251", "1", "150"))
            assertEquals(listOf("0"), p.call("preview", "1000", "0", "150"))
            assertEquals(listOf("1"), p.call("preview", "1001", "1", "150"))
            assertEquals(listOf("0", "0", "-1"), p.call("glidePlan", "0", "8", "1"))
            assertEquals(listOf("1", "5", "0"), p.call("glidePlan", "5", "8", "1"))
            assertEquals(listOf("0", "1", "-1"), p.call("glidePlan", "5", "1", "0"))
        }
    }
    @Test fun malformedMessagesAndClosedHandlesThrow() {
        val n = NlpSession()
        assertFailsWith<IllegalArgumentException> { n.call("context", "short") }
        assertFailsWith<IllegalArgumentException> { n.call("complete", "1", "partial") }
        assertFailsWith<IllegalArgumentException> { n.call("inlineComplete") }
        assertFailsWith<IllegalArgumentException> { n.call("unknown") }
        n.close(); n.close()
        assertFailsWith<IllegalStateException> { n.call("snapshot") }
    }
    @Test fun legacyTimestampCounterexampleIsIntentionallyRejected() {
        // Legacy ReferenceNlpManager compares last COMPLETED time, not latest REQUEST time.
        val lastCompleted = 10L
        val oldRequest = 11L
        val newestRequest = 12L
        assertTrue(oldRequest > lastCompleted && newestRequest > oldRequest)
        // Old result arriving first passes oldRequest > lastCompleted despite newest being in flight.
        outOfOrderResultsNeverApplyOrBecomePickable()
    }
}
