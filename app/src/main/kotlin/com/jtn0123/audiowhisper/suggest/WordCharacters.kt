package com.jtn0123.audiowhisper.suggest

/** Which typed characters end a word (and so may trigger auto-correction of it). */
object WordCharacters {
    /**
     * AOSP's word connectors (`'` and `-`, plus the typographic apostrophe) and digits continue
     * the word being typed, so "don't", "e-mail" and "mp3" are not auto-corrected halfway through;
     * `@` does too, so the name part of an e-mail address is left alone.
     */
    private val CONTINUING_PUNCTUATION = setOf('\''.code, '’'.code, '-'.code, '@'.code)

    /** True if typing [codePoint] right after a word continues it rather than ending it. */
    fun continuesWord(codePoint: Int): Boolean =
        Character.isDigit(codePoint) || codePoint in CONTINUING_PUNCTUATION
}
