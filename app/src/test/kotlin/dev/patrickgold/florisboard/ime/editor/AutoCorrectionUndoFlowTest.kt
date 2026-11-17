package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.suggest.AutoCorrectionUndo
import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * One backspace straight after an auto-correction, through the native editor as EditorInstance
 * and KeyboardManager drive it: the context before each edit, the spacing each leaves, the undo
 * read from the editor's expected content, and cursor updates that may come late.
 */
class AutoCorrectionUndoFlowTest {
    private class Editor(
        val ic: PlanConnection = FakeInputConnection(),
        private val autoSpacePunctuation: Boolean = false,
        inputType: Int = 0x4001,
    ) : AutoCloseable {
        private val driver = NativeEditorDriver(ic)
        private val auto = EditorInstance.AutoSpaceState()
        private val phantom = EditorInstance.PhantomSpaceState()
        /** Cursor updates the app has sent and the keyboard has not seen yet. */
        private val unseen = ArrayDeque<IntArray>()
        private var undo: AutoCorrectionUndo? = null

        init {
            driver.input = inputType
            driver.run("start", "0", "0", "1")
        }

        val phantomSpace get() = phantom.isActive

        private fun configure() {
            driver.phantom = phantom.isActive
            driver.determine = phantom.isInactive || phantom.showComposingRegion
        }

        private fun edit(operation: String, vararg fields: String) {
            val rule = PunctuationRule.Fallback
            configure()
            driver.session.call("context", auto.bits.toString(), phantom.bits.toString(), KeyVariation.NORMAL.value.toString(), if (autoSpacePunctuation) "1" else "0",
                "en", rule.symbolsPrecedingAutoSpace, rule.symbolsFollowingAutoSpace, rule.symbolsPrecedingPhantomSpace,
                rule.symbolsFollowingPhantomSpace, "1")
            driver.run(operation, *fields)
            val spaces = driver.session.call("spaces")
            auto.bits = spaces[0].toInt()
            phantom.reset(spaces[1].toInt())
            unseen += ic.cursorUpdates()
        }

        /** The keyboard sees the updates the app sent, as EditorInstance.handleSelectionUpdate does. */
        fun catchUp() {
            while (unseen.isNotEmpty()) {
                val update = unseen.removeFirst()
                followUpdate(driver.session.isLate(update[0], update[1], update[2], update[3]), auto, phantom)
                configure()
                driver.run("update", *update.map(Int::toString).toTypedArray(), "1")
            }
        }

        private val content get() = EditorSnapshot(driver.session.call("snapshot")).content

        fun type(text: String) = text.forEach { edit("typeChar", it.toString()); catchUp() }

        fun swipe(word: String) { edit("gesture", word); catchUp() }

        /** KeyboardManager.autoCorrect committing [corrected], then [separator] as its key does. */
        fun autoCorrect(corrected: String, separator: String) {
            val typed = content.composingText
            undo = AutoCorrectionUndo(typed, corrected, content.composing.start)
            edit("completion", corrected)
            if (separator == " ") edit("typeText", " ") else edit("typeChar", separator)
        }

        /** KeyboardManager.undoAutoCorrection, for the backspace. */
        fun backspace(): Boolean {
            val undo = undo ?: return false
            this.undo = null
            val content = content
            if (!content.selection.isCursorMode) return false
            val revert = undo.revertAt(content.selection.start, content.textBeforeSelection) ?: return false
            edit("revert", revert.length.toString(), revert.text, if (revert.phantomSpace) "1" else "0")
            return true
        }

        override fun close() = driver.close()
    }

    @Test
    fun `one backspace puts back the typed word and owes the space`() = Editor().use {
        it.type("I saw teh")
        it.autoCorrect("the", " ")
        it.catchUp()
        assertEquals("I saw the ", it.ic.text)
        assertTrue(it.backspace())
        it.catchUp()
        assertEquals("I saw teh", it.ic.text)
        assertTrue(it.phantomSpace)
        it.type("y")
        assertEquals("I saw teh y", it.ic.text)
    }

    @Test
    fun `a backspace before the cursor updates arrive still undoes, and the late updates keep the space owed`() =
        Editor().use {
            it.type("I saw teh")
            it.autoCorrect("the", " ")
            assertTrue(it.backspace())
            it.catchUp()
            assertEquals("I saw teh", it.ic.text)
            assertTrue(it.phantomSpace)
            it.type("y")
            assertEquals("I saw teh y", it.ic.text)
        }

    @Test
    fun `the punctuation that triggered the correction is kept`() = Editor().use {
        it.type("I saw teh")
        it.autoCorrect("the", ",")
        it.catchUp()
        assertEquals("I saw the,", it.ic.text)
        assertTrue(it.backspace())
        it.catchUp()
        assertEquals("I saw teh,", it.ic.text)
    }

    @Test
    fun `so is the space put after the punctuation automatically`() = Editor(autoSpacePunctuation = true).use {
        it.type("I saw teh")
        it.autoCorrect("the", ",")
        it.catchUp()
        assertEquals("I saw the, ", it.ic.text)
        assertTrue(it.backspace())
        it.catchUp()
        assertEquals("I saw teh, ", it.ic.text)
    }

    @Test
    fun `a word typed after an automatic space comes back in its place`() = Editor().use {
        it.swipe("hello")
        it.type("teh")
        assertEquals("hello teh", it.ic.text)
        it.autoCorrect("the", " ")
        it.catchUp()
        assertTrue(it.backspace())
        it.catchUp()
        assertEquals("hello teh", it.ic.text)
    }

    /**
     * The Rust settings screen's try-out field (0x8001, asking for auto-correction), through the
     * InputConnection SettingsInputView gives the keyboard. Like EditText, it holds back the cursor
     * updates of the keyboard's batch of edits until the batch ends, so they match the edits the
     * keyboard expects and the undo is still on offer.
     */
    @Test
    fun `one backspace undoes in the Rust settings try-out field too`() = KeyboardRenderer(emptyArray()).use { native ->
        Editor(SettingsFieldConnection(native), inputType = 0x8001).use {
            it.type("I saw teh")
            it.autoCorrect("the", " ")
            it.catchUp()
            assertEquals("I saw the ", it.ic.text)
            assertTrue(it.backspace())
            it.catchUp()
            assertEquals("I saw teh", it.ic.text)
            assertTrue(it.phantomSpace)
            it.type("y")
            assertEquals("I saw teh y", it.ic.text)
        }
    }
}
