package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.security.MessageDigest
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The production owner sees, under seeded executor schedules, exactly what the frozen pre-port owner
 * saw: its snapshots are golden digests (docs/phase6-kotlin-references.md, generated at e00807e9d).
 */
class DictationSessionMatchTest {
    private class Side {
        val platform = FakePlatform()
        val editor = FakeEditor()
        val flows = DictationFlows()
        val native = DictationSession(editor, platform, flows)
        fun press() = native.pressMic()
        fun lift() = native.releaseMic()
        fun stop() = native.stopListening()
        fun cancel() = native.abandon()
        fun visible(value: Boolean) = native.keyboardVisible(value)
        fun release() = native.release()
        fun free() = native.freeIfIdle()
        fun snapshot(): List<String> = listOf(flows.status.value, flows.warming.value, flows.level.value.toRawBits(),
            hash(flows.preview.value), hash(editor.text.toString()), editor.problems.toList(), editor.undone, editor.ticks,
            platform.appSettingsOpened, platform.modelSettingsOpened, platform.model.loaded, platform.schedule(),
            platform.queued(), platform.listening.map { listOf(it.stopped, it.kept) }).map { it.toString() }
    }
    @Test fun seededExecutorAndLateResultStreams() {
        val tape = GoldenTape("dictation-session-streams")
        var compared = 0
        repeat(200) { seed ->
            val rng = Random(seed)
            val sides = listOf(Side())
            repeat(250) { step ->
                val event = rng.nextInt(23)
                val age = rng.nextInt(12)
                val elapsed = listOf(0L, 1L, 50L, 299L, 300L, 60_000L, 300_000L)[rng.nextInt(7)]
                val value = rng.nextBoolean()
                val corpus = listOf("", " ", "synthetic fixture", "I need to.", "John arrives.", "comma", "scratch that", "new paragraph", FakeModel.FAIL,
                    "Alpha. New line. Beta.", "Scratch that. And continue.", "\uD800", "\u03A3\u00df test", "NASA and I.")
                val phrase = corpus[rng.nextInt(corpus.size)]
                for (s in sides) {
                    val p = s.platform
                    val listener = p.listening.getOrNull((p.listening.size - 1 - age).coerceAtLeast(0))
                    when (event) {
                        0 -> s.press()
                        1 -> s.lift()
                        2 -> s.stop()
                        3 -> s.cancel()
                        4 -> s.visible(value)
                        5 -> p.now += elapsed
                        6 -> p.runOne(value)
                        7 -> p.runAll()
                        8 -> listener?.phrase(phrase)
                        9 -> listener?.preview(phrase)
                        10 -> listener?.level(if (value) .5f else 0f)
                        11 -> listener?.error()
                        12 -> p.micAllowed = value
                        13 -> p.modelState = if (value) ModelState.Ready else ModelState.Missing(listOf("fixture"))
                        14 -> p.modelState = if (value) ModelState.Ready else ModelState.Damaged(listOf("fixture"))
                        15 -> p.model.loads = value
                        16 -> p.micOpens = value
                        17 -> s.editor.allows = value
                        18 -> listener?.unfinished = phrase
                        19 -> s.free()
                        21 -> listener?.speech(true)
                        22 -> listener?.speech(false)
                        20 -> { s.press(); p.now += elapsed; s.lift() }
                    }
                }
                tape.record(sides[0].snapshot())
                compared++
            }
            sides.forEach { it.release(); it.platform.runAll() }
            tape.record(sides[0].snapshot())
        }
        assertEquals(50_000, compared)
        tape.finish()
    }
    @Test fun editorFiltersAreObservedBetweenSpokenOperations() {
        val sides=listOf(Side())
        for (s in sides) {
            s.editor.text.append("ab");s.editor.maxLength=6
            s.press();s.platform.listening.single().phrase("hello. Comma. And continue.");s.platform.runAll()
        }
        GoldenTape.check("dictation-session-editor-filters") { it.record(sides[0].snapshot()) }
    }
    @Test fun releasedOwnerCannotOverwriteANewOwnersUiOrSubmitLateWork() {
        val flows = DictationFlows()
        val firstPlatform = FakePlatform()
        val first = DictationSession(FakeEditor(), firstPlatform, flows)
        first.pressMic()
        val oldListener = firstPlatform.listening.single()
        first.release()
        firstPlatform.runAll()
        val nextPlatform = FakePlatform()
        val next = DictationSession(FakeEditor(), nextPlatform, flows)
        next.pressMic()
        first.pressMic(); first.releaseMic(); first.keyboardVisible(false)
        oldListener.phrase("synthetic late fixture")
        firstPlatform.runAll()
        assertEquals(Dictation.Status.LISTENING, flows.status.value)
        next.release(); nextPlatform.runAll()
    }
    companion object {
        private fun hash(s: String): String = MessageDigest.getInstance("SHA-256")
            .digest(ByteArray(s.length * 2) { i -> (s[i / 2].code ushr (8 * (i % 2))).toByte() }).joinToString("") { "%02x".format(it) }
    }
}
