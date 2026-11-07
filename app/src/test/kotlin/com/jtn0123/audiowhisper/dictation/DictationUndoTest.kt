package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.dictation.DictationUndo.Revert
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DictationUndoTest {
    @Test
    fun `takes back the whole phrase, space in front included`() {
        val undo = DictationUndo(" about it.", start = 11, restore = "")
        assertEquals(Revert(10, ""), undo.revertAt(21, "Talk to him about it."))
    }

    @Test
    fun `puts back a full stop joining took`() {
        val undo = DictationUndo(" John about it.", start = 16, restore = ".")
        assertEquals(Revert(15, "."), undo.revertAt(31, "I need to talk to John about it."))
    }

    @Test
    fun `does nothing once the cursor or the text has moved on`() {
        val undo = DictationUndo(" about it.", start = 11, restore = "")
        assertNull(undo.revertAt(20, "Talk to him about it"), "cursor moved")
        assertNull(undo.revertAt(21, "Talk to him abowt it."), "text edited")
        assertNull(DictationUndo("", start = 0, restore = "").revertAt(0, ""))
    }
}
