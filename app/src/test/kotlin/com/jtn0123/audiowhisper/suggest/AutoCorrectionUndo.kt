package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules

/**
 * The last auto-correction, kept so that a backspace straight after it puts back what was typed,
 * as AOSP's keyboard does ("teh " → "the ", backspace → "teh").
 *
 * [start] is where the corrected word begins in the editor.
 */
data class AutoCorrectionUndo(val typed: String, val corrected: String, val start: Int) {
    /** Replace the [length] characters left of the cursor with [text]. */
    data class Revert(val length: Int, val text: String, val phantomSpace: Boolean)

    /**
     * How to undo with the cursor at [cursor] and [textBeforeCursor] left of it, or null if that
     * text is no longer the correction followed by the separator that triggered it: a space, a
     * punctuation mark, or a punctuation mark plus the space added after it (no letter or digit).
     *
     * A space separator is dropped and left pending as a phantom space, so the user can fix the
     * word, add punctuation to it, or just type on and get the space back. Punctuation is kept.
     * The rule is in Rust (lib/rust/crates/suggest).
     */
    fun revertAt(cursor: Int, textBeforeCursor: CharSequence): Revert? =
        SuggestRules.revertAt(typed, corrected, start, cursor, textBeforeCursor.toString())
            ?.let { Revert(it.length, it.text, it.phantomSpace) }
}
