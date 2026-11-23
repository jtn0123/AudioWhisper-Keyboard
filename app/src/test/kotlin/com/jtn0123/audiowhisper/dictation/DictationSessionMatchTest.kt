package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.security.MessageDigest
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/** The frozen pre-port owner and production owner see identical executor schedules. */
class DictationSessionMatchTest {
    private class Side(reference: Boolean) {
        val platform = FakePlatform()
        val editor = FakeEditor()
        val flows = DictationFlows()
        val oldFlows = ReferenceDictationFlows()
        val native = if (!reference) DictationSession(editor, platform, flows) else null
        val old = if (reference) ReferenceDictationSession(editor, platform, oldFlows) else null
        fun press() { native?.pressMic() ?: old!!.pressMic() }
        fun lift() { native?.releaseMic() ?: old!!.releaseMic() }
        fun stop() { native?.stopListening() ?: old!!.stopListening() }
        fun cancel() { native?.abandon() ?: old!!.abandon() }
        fun visible(value: Boolean) { native?.keyboardVisible(value) ?: old!!.keyboardVisible(value) }
        fun release() { native?.release() ?: old!!.release() }
        fun free() { native?.freeIfIdle() ?: old!!.freeIfIdle() }
        fun snapshot(): List<Any> {
            val status = native?.let { flows.status.value } ?: oldFlows.status.value
            val warming = native?.let { flows.warming.value } ?: oldFlows.warming.value
            val level = native?.let { flows.level.value } ?: oldFlows.level.value
            val preview = native?.let { flows.preview.value } ?: oldFlows.preview.value
            return listOf(status, warming, level.toRawBits(), hash(preview), hash(editor.text.toString()),
                editor.problems.toList(), editor.undone, editor.ticks, platform.appSettingsOpened,
                platform.modelSettingsOpened, platform.model.loaded, platform.schedule(), platform.queued(),
                platform.listening.map { listOf(it.stopped, it.kept) })
        }
    }
    @Test fun seededExecutorAndLateResultStreams() {
        val tape = GoldenTape.ofReference("dictation-session-streams")
        var compared = 0
        repeat(200) { seed ->
            val rng = Random(seed)
            val sides = listOf(Side(true), Side(false))
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
                assertEquals(sides[0].snapshot(), sides[1].snapshot(), "seed=$seed step=$step event=$event")
                tape.record(sides[0].snapshot().map { it.toString() })
                compared++
            }
            sides.forEach { it.release(); it.platform.runAll() }
            assertEquals(sides[0].snapshot(), sides[1].snapshot(), "release seed=$seed")
            tape.record(sides[0].snapshot().map { it.toString() })
        }
        assertEquals(50_000, compared)
        tape.finish()
    }
    @Test fun editorFiltersAreObservedBetweenSpokenOperations() {
        val sides=listOf(Side(true),Side(false))
        for (s in sides) {
            s.editor.text.append("ab");s.editor.maxLength=6
            s.press();s.platform.listening.single().phrase("hello. Comma. And continue.");s.platform.runAll()
        }
        assertEquals(sides[0].snapshot(),sides[1].snapshot(),"Filtered editor command sequence")
        GoldenTape.ofReference("dictation-session-editor-filters").apply { record(sides[0].snapshot().map { it.toString() }) }.finish()
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
