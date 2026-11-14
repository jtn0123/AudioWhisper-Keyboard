package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.DictationRules
import com.jtn0123.audiowhisper.rust.DictationRules.Kind

/** What part of a dictated phrase asks for: words to type, or a spoken command. */
sealed interface Spoken {
    data class Words(val text: String) : Spoken

    /** "Comma", "question mark": put in place of the punctuation the dictation just typed. */
    data class Mark(val mark: Char) : Spoken

    /** "New line", "new paragraph". */
    data class Break(val text: String) : Spoken

    /** "Scratch that": take back the phrase typed last. */
    data object ScratchThat : Spoken

    companion object {
        /**
         * Splits [phrase] into words and commands. A command counts only as a sentence of its
         * own ("Hello there. New line. How are you?"), never inside one ("I said new line"),
         * because the model only writes it that way when the speaker says it on its own. The
         * rules and the list of commands are in Rust: lib/rust/crates/dictation/src/spoken.rs.
         */
        fun parse(phrase: String): List<Spoken> = DictationRules.parse(phrase).map { part ->
            when (part.kind) {
                Kind.WORDS -> Words(part.text)
                Kind.MARK -> Mark(part.text.single())
                Kind.BREAK -> Break(part.text)
                Kind.SCRATCH_THAT -> ScratchThat
            }
        }
    }
}
