package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.DictationRules

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

    /**
     * How to undo with the cursor at [cursor] after [textBeforeCursor], or null if it has moved or
     * the text changed. Decided in Rust: lib/rust/crates/dictation/src/undo.rs.
     */
    fun revertAt(cursor: Int, textBeforeCursor: CharSequence): Revert? =
        if (DictationRules.revertsAt(typed, start, cursor, textBeforeCursor.toString())) Revert(typed.length, restore) else null
}
