package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.rust.NlpSession
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
        EditorPlanMatchTest.Pairing().use { editor ->
            editor.start("prefix ")
            editor.char("a") // InputConnection succeeded; its selection acknowledgement is still delayed.
            val published = fields(EditorSnapshot(editor.new.session.call(EditorOp.PUBLISHED)).content)
            val current = fields(EditorSnapshot(editor.new.session.call(EditorOp.SNAPSHOT)).content)
            assertFalse(published.contentEquals(current), "Fixture must keep a newer prediction than its notification")
            NlpSession().use { nlp ->
                val wrong = nlp.call("beginFor", *published, "100")[0]
                assertEquals("0", nlp.call("completeFor", *current, wrong, "old", "alpha", "0", "0")[0])
                val right = nlp.call("beginFor", *current, "100")[0]
                assertEquals("1", nlp.call("completeFor", *current, right, "current", "alpha", "0", "0")[0])
                assertEquals("current", nlp.call("snapshot")[4])
            }
            editor.reconcile()
        }
    }
}
