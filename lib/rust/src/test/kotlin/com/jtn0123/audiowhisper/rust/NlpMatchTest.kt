package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/**
 * The NLP session call for call as the frozen NlpManager, LatinLanguageProvider, inline and clipboard
 * decisions answered (ReferenceNlp, deleted for golden digests of its answers;
 * docs/phase6-kotlin-references.md). It kept the deliberate safety changes: generations replace the
 * legacy timestamps, repeated provider entries are dropped, and password and settings eligibility
 * covers every provider; and AudioWhisper's own rules: the Smartbar holds words typing emptied until
 * the refill, a filter, URL or e-mail field is corrected only when it asks, and only prose is learned
 * (docs/decisions.md).
 */
class NlpMatchTest {
    /** A session whose every answer, and its snapshot and Smartbar words after each call, go on [tape]. */
    internal class Pairing(private val tape: GoldenTape) : AutoCloseable {
        val rust = NlpSession()
        fun call(op: NlpOp, vararg f: String): List<String> {
            val actual = rust.call(op, *f).toList()
            tape.answer(op.name, actual)
            tape.answer("snapshot", rust.call(NlpOp.SNAPSHOT).toList())
            tape.answer("words", rust.call(NlpOp.WORDS).toList())
            return actual
        }
        override fun close() = rust.close()
    }
    @Test fun seededSessionsMatchFiftyThousandEvents() {
        var events = 0
        val reached = mutableSetOf<Int>()
        val tape = GoldenTape("nlp-sessions")
        repeat(200) { seed ->
            val random = Random(20261113 + seed)
            Pairing(tape).use { p ->
                var context = context()
                p.call(NlpOp.CONTEXT, *context.toTypedArray())
                p.call(NlpOp.PREVIEW_START, "0")
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
                            p.call(NlpOp.CONTEXT, *context.toTypedArray())
                            pending += p.call(NlpOp.BEGIN, (step / 2).toString())[0] // equal timestamps, faster than debounce
                        }
                        3 -> if (pending.isNotEmpty()) {
                            val gen = pending.removeAt(random.nextInt(pending.size))
                            p.call(NlpOp.READY, gen, (step / 2 + 10).toString())
                            val values = if (random.nextInt(5) == 0) emptyArray() else arrayOf("a$step", "help", "0", "1", "b$step", "help", "0", "0", "c$step", "helo", "0", "0", "e$step", "😀", "1", "0")
                            p.call(NlpOp.COMPLETE, gen, *values)
                        }
                        4 -> {
                            context = context.toMutableList().also { it[10] = random.nextInt(5).toString(); it[11] = it[10] }
                            p.call(NlpOp.CONTEXT, *context.toTypedArray())
                        }
                        5 -> {
                            context = context.toMutableList().also { it[0] = listOf(1, 0x81, 0x91, 0xe1, 0x12, 0x21, 0x11, 0x80001, 0x8021, 0x8011, 0x80b1)[random.nextInt(11)].toString() }
                            p.call(NlpOp.CONTEXT, *context.toTypedArray()); p.call(NlpOp.POLICY, *context.toTypedArray())
                        }
                        6 -> {
                            context = context.toMutableList().also { it[7] = random.nextInt(2).toString(); it[4] = random.nextInt(2).toString(); it[5] = random.nextInt(2).toString(); it[16] = if (random.nextBoolean()) ":" else "" }
                            p.call(NlpOp.CONTEXT, *context.toTypedArray())
                        }
                        7 -> { inflations += p.call(NlpOp.INLINE_BEGIN, random.nextInt(4).toString())[0] }
                        8 -> if (inflations.isNotEmpty()) p.call(NlpOp.INLINE_COMPLETE, inflations.removeAt(random.nextInt(inflations.size)), "unpin", "0", "pin1", "1", "pin2", "1")
                        9 -> {
                            p.call(NlpOp.AUTO, listOf(-1, 32, 46, 39, 64, 0x1d7ce)[random.nextInt(6)].toString(), random.nextInt(2).toString())
                            p.call(NlpOp.PREVIEW, (step * 7).toString(), random.nextInt(2).toString(), random.nextInt(200).toString())
                            p.call(NlpOp.GLIDE_PLAN, random.nextInt(9).toString(), random.nextInt(1, 9).toString(), random.nextInt(2).toString())
                        }
                        10 -> {
                            p.call(NlpOp.LEARN, random.nextInt(3).toString(), listOf(-1, 39, 46)[random.nextInt(3)].toString(), "zorblax", "a ", "2", random.nextInt(2).toString(), "a")
                            p.call(NlpOp.KEEP, "zorblax"); p.call(NlpOp.SAVE, (step * 97L).toString())
                        }
                        11 -> p.call(NlpOp.CLEAR)
                        12 -> p.call(NlpOp.DIRECT, "direct", "zorblax", "0", "0")
                        13 -> { p.call(NlpOp.CLIPBOARD, "clip", "synthetic", "2", "0"); p.call(NlpOp.PICK, "direct") }
                        14 -> clipboardRequests.add(p.call(NlpOp.CLIPBOARD_BEGIN_FOR, *context.toTypedArray())[0] to context.toList())
                        15 -> if (clipboardRequests.isNotEmpty()) {
                            val (gen, requested) = clipboardRequests.removeAt(random.nextInt(clipboardRequests.size))
                            p.call(NlpOp.CLIPBOARD_FOR, *requested.toTypedArray(), *context.toTypedArray(), gen, "clip$step", "synthetic", "2", "0")
                        }
                    }
                    events++
                }
            }
        }
        tape.finish()
        assertEquals(50_000, events); assertEquals((0..15).toSet(), reached)
        println("NLP: 50000 events, 200 seeds")
    }
    @Test fun outOfOrderResultsNeverApplyOrBecomePickable() {
        val tape = GoldenTape("nlp-out-of-order")
        Pairing(tape).use { p ->
            p.call(NlpOp.CONTEXT, *context("h").toTypedArray())
            val external = p.call(NlpOp.EXTERNAL_BEGIN)[0]
            val old = p.call(NlpOp.BEGIN, "100")[0]
            p.call(NlpOp.CONTEXT, *context("hel").toTypedArray())
            assertEquals(listOf("0"), p.call(NlpOp.EXTERNAL_VALID, external))
            val current = p.call(NlpOp.BEGIN, "100")[0]
            assertEquals(listOf("0"), p.call(NlpOp.COMPLETE, old, "stale", "hat", "0", "1"))
            assertEquals(listOf("1"), p.call(NlpOp.COMPLETE, current, "fresh", "help", "0", "1"))
            assertEquals(listOf("0"), p.call(NlpOp.COMPLETE, old, "stale", "hat", "0", "1"))
            assertEquals(listOf("0"), p.call(NlpOp.PICK, "stale"))
            assertEquals(listOf("1"), p.call(NlpOp.PICK, "fresh"))
            val inline = p.call(NlpOp.INLINE_BEGIN, "1")[0]
            p.call(NlpOp.CLEAR)
            // Cursor/field/privacy changes invalidate inline inflation too.
            p.call(NlpOp.CONTEXT, *context("hel", 0x81).toTypedArray())
            assertEquals(listOf("0"), p.call(NlpOp.INLINE_COMPLETE, inline, "secret", "1"))
            assertEquals(listOf("0"), p.call(NlpOp.AUTO, "-1", "0"))
        }
        tape.finish()
    }
    @Test fun candidateOrderingTypedEntryAndSharedSmartbarRules() {
        val tape = GoldenTape("nlp-candidate-order")
        Pairing(tape).use { p ->
            p.call(NlpOp.CONTEXT, *context().toTypedArray())
            p.call(NlpOp.DIRECT, "1", "help", "0", "1", "2", "helo", "0", "0", "3", "help", "0", "0", "4", "😀", "1", "0")
            assertEquals(listOf("2", "0", "1", "3", "1", "2", "4", "0"), p.call(NlpOp.SNAPSHOT))
            val gen = p.call(NlpOp.INLINE_BEGIN, "3")[0]
            p.call(NlpOp.INLINE_COMPLETE, gen, "a", "0", "b", "1", "c", "1")
            assertEquals(listOf("b", "c", "a"), p.call(NlpOp.SNAPSHOT).takeLast(3))
            p.call(NlpOp.INLINE_BEGIN, "0")
            assertEquals("0", p.call(NlpOp.SNAPSHOT)[1]) // candidates still prevent expansion
            val selection = context().toMutableList().also { it[11] = "0" }
            p.call(NlpOp.CONTEXT, *selection.toTypedArray())
            assertEquals("1", p.call(NlpOp.SNAPSHOT)[1])
        }
        tape.finish()
    }
    @Test fun platformAutofillInPasswordDoesNotEnableWordCandidates() {
        val tape = GoldenTape("nlp-password-autofill")
        Pairing(tape).use { p ->
            p.call(NlpOp.CONTEXT, *context("", 0x81).toTypedArray())
            assertEquals(listOf("0", "0", "0", "0", "0", "1"), p.call(NlpOp.POLICY, *context("", 0x81).toTypedArray()))
            val inline = p.call(NlpOp.INLINE_BEGIN, "1")
            assertEquals("1", inline[1])
            p.call(NlpOp.INLINE_COMPLETE, inline[0], "platform", "1")
            assertEquals("0", p.call(NlpOp.SNAPSHOT)[3])
            assertEquals("platform", p.call(NlpOp.SNAPSHOT).last())
        }
        tape.finish()
    }
    @Test fun rawFlagsTypingAndClipboardMatchOriginalDecisions() {
        val tape = GoldenTape("nlp-raw-flags")
        Pairing(tape).use { p ->
            val random = Random(16)
            repeat(5000) {
                val context = context(input = random.nextInt()).toMutableList().also { it[3] = if (random.nextBoolean()) "en" else "fr" }
                p.call(NlpOp.POLICY, *context.toTypedArray())
                p.call(NlpOp.CANDIDATE_PLAN, *context.toTypedArray(), random.nextInt(2).toString(), "1", "0", "1")
                p.call(NlpOp.TYPING_POLICY, *context.toTypedArray(), "hello", listOf("", "abc", ":smile", "😀")[random.nextInt(4)], "x")
                p.call(NlpOp.CLIPBOARD_PLAN, random.nextInt(2).toString(), "1", random.nextInt(2).toString(), if (random.nextBoolean()) "" else "a", "aw@example.test", random.nextInt(1001).toString(), "1000", random.nextInt(2).toString(), "example.test", "3", "14", "aw@example.test", "0", "14", "test", "11", "14")
            }
        }
        tape.finish()
    }
    @Test fun productionBulkMessagesCheckTheCurrentEditorBeforePublishing() {
        val tape = GoldenTape("nlp-bulk-editor")
        Pairing(tape).use { p ->
            val old = context("Aa")
            val newer = context("BB")
            assertEquals("Aa".hashCode(), "BB".hashCode())
            val a = p.call(NlpOp.BEGIN_FOR, *old.toTypedArray(), "100")[0]
            val b = p.call(NlpOp.BEGIN_FOR, *newer.toTypedArray(), "100")[0]
            assertEquals("0", p.call(NlpOp.COMPLETE_FOR, *newer.toTypedArray(), a, "stale", "hat", "0", "1")[0])
            assertEquals("0", p.call(NlpOp.COMPLETE_FOR, *old.toTypedArray(), a, "stale-metadata", "hat", "0", "1")[0])
            assertEquals(b, p.call(NlpOp.SNAPSHOT)[0], "Old completion metadata must not cancel the new request")
            assertEquals("1", p.call(NlpOp.COMPLETE_FOR, *newer.toTypedArray(), b, "fresh", "book", "0", "1")[0])
            assertEquals("fresh", p.call(NlpOp.SNAPSHOT)[4])
        }
        tape.finish()
    }
    @Test fun freshSynchronousAutocorrectGetsAOneShotNativeIdentity() {
        val tape = GoldenTape("nlp-auto-identity")
        Pairing(tape).use { p ->
            val current = context("teh")
            val changed = context("else")
            p.call(NlpOp.CONTEXT, *current.toTypedArray())
            assertEquals(listOf("1"), p.call(NlpOp.AUTO_CANDIDATE, *current.toTypedArray(), *current.toTypedArray(), "fresh-auto", "the", "0", "1"))
            p.call(NlpOp.BEGIN, "100") // same editor, a background strip refresh cannot cancel a fresh synchronous result
            assertEquals(listOf("1"), p.call(NlpOp.PICK, "fresh-auto"))
            assertEquals(listOf("0"), p.call(NlpOp.PICK, "fresh-auto"))
            assertEquals(listOf("0"), p.call(NlpOp.AUTO_CANDIDATE, *current.toTypedArray(), *changed.toTypedArray(), "stale-auto", "the", "0", "1"))
            assertEquals(listOf("0"), p.call(NlpOp.PICK, "stale-auto"))
        }
        tape.finish()
    }
    @Test fun clipboardAndSynchronousResultsCannotRollBackANewerRequest() {
        val tape = GoldenTape("nlp-rollback")
        Pairing(tape).use { p ->
            val old = context("old")
            val current = context("current")
            val oldClip = p.call(NlpOp.CLIPBOARD_BEGIN_FOR, *old.toTypedArray())[0]
            p.call(NlpOp.BEGIN_FOR, *old.toTypedArray(), "100")
            val fresh = p.call(NlpOp.BEGIN_FOR, *current.toTypedArray(), "101")[0]
            val before = p.call(NlpOp.SNAPSHOT)
            assertEquals(before, p.call(NlpOp.CLIPBOARD_FOR, *old.toTypedArray(), *old.toTypedArray(), oldClip, "old-clip", "clip", "2", "0"))
            assertEquals(before, p.call(NlpOp.CLIPBOARD_FOR, *old.toTypedArray(), *current.toTypedArray(), oldClip, "changed-clip", "clip", "2", "0"))
            assertEquals(listOf("0"), p.call(NlpOp.AUTO_CANDIDATE, *old.toTypedArray(), *old.toTypedArray(), "old-auto", "older", "0", "1"))
            assertEquals(before, p.call(NlpOp.SNAPSHOT), "Obsolete result metadata cannot change context or cancel pending work")
            assertEquals(listOf("1"), p.call(NlpOp.READY, fresh, "109"))
            assertEquals("1", p.call(NlpOp.COMPLETE_FOR, *current.toTypedArray(), fresh, "new", "current", "0", "0")[0])
            val clip1 = p.call(NlpOp.CLIPBOARD_BEGIN_FOR, *current.toTypedArray())[0]
            val clip2 = p.call(NlpOp.CLIPBOARD_BEGIN_FOR, *current.toTypedArray())[0]
            p.call(NlpOp.CLIPBOARD_FOR, *current.toTypedArray(), *current.toTypedArray(), clip2, "fresh-clip", "clip", "2", "0")
            val shown = p.call(NlpOp.SNAPSHOT)
            assertEquals(shown, p.call(NlpOp.CLIPBOARD_FOR, *current.toTypedArray(), *current.toTypedArray(), clip1, "stale-same-editor", "clip", "2", "0"))
            assertEquals("fresh-clip", p.call(NlpOp.SNAPSHOT)[4])
        }
        tape.finish()
    }
    @Test fun pendingRequestsHideStaleWordsWithoutFlashingExpandedActions() {
        val tape = GoldenTape("nlp-pending")
        Pairing(tape).use { p ->
            p.call(NlpOp.CONTEXT, *context().toTypedArray())
            p.call(NlpOp.DIRECT, "shown", "help", "0", "0")
            assertEquals("0", p.call(NlpOp.SNAPSHOT)[1])
            val generation = p.call(NlpOp.BEGIN, "100")[0]
            assertEquals("0", p.call(NlpOp.SNAPSHOT)[3])
            assertEquals("-1", p.call(NlpOp.SNAPSHOT)[1], "Pending leaves the existing expansion binding unchanged")
            p.call(NlpOp.COMPLETE, generation)
            assertEquals("1", p.call(NlpOp.SNAPSHOT)[1], "An empty completed response expands actions")
        }
        tape.finish()
    }
    @Test fun typingHoldsTheSmartbarWordsAcrossTheRefillOnly() {
        val tape = GoldenTape("nlp-hold")
        Pairing(tape).use { p ->
            fun words() = p.call(NlpOp.WORDS).single()
            p.call(NlpOp.CONTEXT, *context("h").toTypedArray())
            p.call(NlpOp.BEGIN, "0")
            assertEquals("0", words(), "An empty row holds nothing while its first request is in flight")
            p.call(NlpOp.DIRECT, "help", "help", "0", "0")
            assertEquals("1", words())
            // A collector's configure lands before the keystroke's beginFor: typing-only change.
            p.call(NlpOp.CONTEXT, *context("he").toTypedArray())
            assertEquals(listOf("1", "-1"), listOf(words(), p.call(NlpOp.SNAPSHOT)[1]), "Carry holds the row and the expansion")
            val generation = p.call(NlpOp.BEGIN_FOR, *context("he").toTypedArray(), "10")[0]
            assertEquals(listOf("1", "0"), listOf(words(), p.call(NlpOp.SNAPSHOT)[3]), "The begin gap holds an empty list")
            p.call(NlpOp.COMPLETE, generation, "hello", "hello", "0", "0")
            assertEquals("1", words(), "The refill shows words")
            val empty = p.call(NlpOp.BEGIN_FOR, *context("hex").toTypedArray(), "20")[0]
            assertEquals("1", words())
            p.call(NlpOp.COMPLETE, empty)
            assertEquals(listOf("0", "1"), listOf(words(), p.call(NlpOp.SNAPSHOT)[1]), "An empty refill ends the hold")
            // Changes that are not typing never hold: incognito, a selection, a cleared row.
            p.call(NlpOp.DIRECT, "help", "help", "0", "0")
            p.call(NlpOp.CONTEXT, *context("hex", 0x81).toTypedArray())
            assertEquals("0", words(), "Incognito does not hold")
            p.call(NlpOp.CONTEXT, *context("hex").toTypedArray()); p.call(NlpOp.DIRECT, "help", "help", "0", "0")
            p.call(NlpOp.CONTEXT, *context("hex").toMutableList().also { it[10] = "0" }.toTypedArray())
            assertEquals("0", words(), "A selection does not hold")
            p.call(NlpOp.CONTEXT, *context("hex").toTypedArray()); p.call(NlpOp.DIRECT, "help", "help", "0", "0")
            p.call(NlpOp.CLEAR)
            assertEquals("0", words(), "Clear does not hold")
        }
        tape.finish()
    }
    @Test fun glidePreviewClockAndCommitSliceMatchTheFrozenManager() {
        val tape = GoldenTape("nlp-glide-preview")
        Pairing(tape).use { p ->
            p.call(NlpOp.PREVIEW_START, "100")
            assertEquals(listOf("0"), p.call(NlpOp.PREVIEW, "250", "1", "150"))
            assertEquals(listOf("1"), p.call(NlpOp.PREVIEW, "251", "1", "150"))
            assertEquals(listOf("0"), p.call(NlpOp.PREVIEW, "1000", "0", "150"))
            assertEquals(listOf("1"), p.call(NlpOp.PREVIEW, "1001", "1", "150"))
            assertEquals(listOf("0", "0", "-1"), p.call(NlpOp.GLIDE_PLAN, "0", "8", "1"))
            assertEquals(listOf("1", "5", "0"), p.call(NlpOp.GLIDE_PLAN, "5", "8", "1"))
            assertEquals(listOf("0", "1", "-1"), p.call(NlpOp.GLIDE_PLAN, "5", "1", "0"))
        }
        tape.finish()
    }
    @Test fun malformedMessagesAndClosedHandlesThrow() {
        val n = NlpSession()
        assertFailsWith<IllegalArgumentException> { n.call(NlpOp.CONTEXT, "short") }
        assertFailsWith<IllegalArgumentException> { n.call(NlpOp.COMPLETE, "1", "partial") }
        assertFailsWith<IllegalArgumentException> { n.call(NlpOp.INLINE_COMPLETE) }
        // An unknown operation can't be named here; nlp's own tests check Rust refuses one.
        n.close(); n.close()
        assertFailsWith<IllegalStateException> { n.call(NlpOp.SNAPSHOT) }
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

    internal companion object {
        const val LATIN = "org.florisboard.nlp.providers.latin"
        /** An [NlpContext] row for a Latin English text field holding [text], the cursor at its end. */
        fun context(text: String = "hel", input: Int = 1): List<String> = listOf(input.toString(), "0", LATIN, "en", "1", "1", "1", "0", "1", "1", text.length.toString(), text.length.toString(), "0", text.length.toString(), text, "fixture-en", "", "1", "1", "1", "0", "0")
    }
}
