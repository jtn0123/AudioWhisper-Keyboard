package com.jtn0123.audiowhisper.suggest

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
     * text is no longer the correction followed by the separator that triggered it.
     *
     * A space separator is dropped and left pending as a phantom space, so the user can fix the
     * word, add punctuation to it, or just type on and get the space back. Punctuation is kept.
     */
    fun revertAt(cursor: Int, textBeforeCursor: CharSequence): Revert? {
        val separatorLength = cursor - start - corrected.length
        if (separatorLength !in 1..MAX_SEPARATOR_LENGTH) return null
        val length = corrected.length + separatorLength
        if (textBeforeCursor.length < length) return null
        val replaced = textBeforeCursor.substring(textBeforeCursor.length - length)
        if (!replaced.startsWith(corrected)) return null
        val separator = replaced.substring(corrected.length)
        if (separator.any { it.isLetterOrDigit() }) return null
        return if (separator == " ") {
            Revert(length, typed, phantomSpace = true)
        } else {
            Revert(length, typed + separator, phantomSpace = false)
        }
    }

    private companion object {
        /** A space, a punctuation mark, or a punctuation mark plus the space added after it. */
        const val MAX_SEPARATOR_LENGTH = 2
    }
}
