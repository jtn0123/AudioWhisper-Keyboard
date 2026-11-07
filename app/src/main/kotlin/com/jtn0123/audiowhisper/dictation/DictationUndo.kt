package com.jtn0123.audiowhisper.dictation

/**
 * The phrase dictation typed last, so a backspace straight after it takes the whole phrase
 * back instead of one letter.
 *
 * [typed] begins at [start] in the editor, space in front included. [restore] is what it
 * replaced there: a full stop that joining the phrase took back, or nothing.
 */
data class DictationUndo(val typed: String, val start: Int, val restore: String) {
    /** Replace the [length] characters left of the cursor with [text]. */
    data class Revert(val length: Int, val text: String)

    /** How to undo with the cursor at [cursor] after [textBeforeCursor], or null if it has moved or the text changed. */
    fun revertAt(cursor: Int, textBeforeCursor: CharSequence): Revert? {
        if (typed.isEmpty() || cursor != start + typed.length) return null
        if (!textBeforeCursor.endsWith(typed)) return null
        return Revert(typed.length, restore)
    }
}
