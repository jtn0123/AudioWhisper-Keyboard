package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules

/** Which typed characters end a word (and so may trigger auto-correction of it). */
object WordCharacters {
    /**
     * True if typing [codePoint] right after a word continues it rather than ending it: AOSP's
     * word connectors (`'` and `-`, plus the typographic apostrophe) and digits, so "don't",
     * "e-mail" and "mp3" are not auto-corrected halfway through; and `@`, so the name part of an
     * e-mail address is left alone. The rule is in Rust (lib/rust/crates/suggest).
     */
    fun continuesWord(codePoint: Int): Boolean = SuggestRules.continuesWord(codePoint)
}
