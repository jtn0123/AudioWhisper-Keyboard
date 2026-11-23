package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

/** Real editor prediction/published divergence must not discard the current request. */
class NlpEditorHandoffMatchTest {
    private fun fields(c: EditorContent): Array<String> = arrayOf("1", "0", "org.florisboard.nlp.providers.latin", "en",
        "1", "1", "1", "0", "1", if (c.localSelection.isValid) "1" else "0",
        c.selection.start.toString(), c.selection.end.toString(), c.composing.start.toString(), c.composing.end.toString(),
        c.text, "en-fixture", "", "1", "1", "1", c.offset.toString(), "0")

    @Test fun currentPredictionRatherThanFlowNotificationStartsTheRequest() {
        val tape=GoldenTape.ofReference("editor-nlp-handoff")
        EditorPlanMatchTest.Pairing(tape).use { editor ->
            editor.start("prefix ")
            editor.char("a") // InputConnection succeeded; its selection acknowledgement is still delayed.
            val published = fields(EditorSnapshot(editor.new.session.call(EditorOp.PUBLISHED)).content)
            val current = fields(EditorSnapshot(editor.new.session.call(EditorOp.SNAPSHOT)).content)
            assertFalse(published.contentEquals(current), "Fixture must keep a newer prediction than its notification")
            NlpSession().use { nlp ->
                val wrong = nlp.call(NlpOp.BEGIN_FOR, *published, "100")[NlpBegin.GENERATION]
                assertEquals("0", nlp.call(NlpOp.COMPLETE_FOR, *current, wrong, "old", "alpha", "0", "0")[0])
                val right = nlp.call(NlpOp.BEGIN_FOR, *current, "100")[NlpBegin.GENERATION]
                assertEquals("1", nlp.call(NlpOp.COMPLETE_FOR, *current, right, "current", "alpha", "0", "0")[0])
                assertEquals("current", nlp.call(NlpOp.SNAPSHOT)[NlpSnapshot.SIZE])
            }
            editor.reconcile()
        }
        tape.finish()
    }
}
