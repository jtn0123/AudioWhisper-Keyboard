package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.suggest.AutoCorrectionUndo.Revert
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class AutoCorrectionUndoTest {
    /** "I saw teh" was corrected to "I saw the" when space was typed. */
    private val undo = AutoCorrectionUndo(typed = "teh", corrected = "the", start = 6)

    @Test
    fun `a space separator becomes a phantom space`() {
        assertEquals(Revert(4, "teh", phantomSpace = true), undo.revertAt(10, "I saw the "))
    }

    @Test
    fun `punctuation is kept`() {
        assertEquals(Revert(4, "teh,", phantomSpace = false), undo.revertAt(10, "I saw the,"))
    }

    @Test
    fun `punctuation followed by an automatic space is kept`() {
        assertEquals(Revert(5, "teh. ", phantomSpace = false), undo.revertAt(11, "I saw the. "))
    }

    @Test
    fun `only the visible window of text needs to hold the correction`() {
        assertEquals(Revert(4, "teh", phantomSpace = true), undo.revertAt(10, "the "))
    }

    @Test
    fun `nothing to undo once the cursor has moved on`() {
        assertNull(undo.revertAt(11, "I saw the a"))
        assertNull(undo.revertAt(9, "I saw the"))
    }

    @Test
    fun `nothing to undo once the cursor is further away than a separator`() {
        assertNull(undo.revertAt(12, "I saw the   "))
    }

    @Test
    fun `nothing to undo if the word was changed`() {
        assertNull(undo.revertAt(10, "I saw tho "))
    }

    @Test
    fun `letters after the word are not a separator`() {
        val shortWord = AutoCorrectionUndo(typed = "i", corrected = "I", start = 0)
        assertNull(shortWord.revertAt(3, "Ian"))
    }

    @Test
    fun `nothing to undo when the text is shorter than the correction`() {
        assertNull(undo.revertAt(10, "e "))
    }
}
