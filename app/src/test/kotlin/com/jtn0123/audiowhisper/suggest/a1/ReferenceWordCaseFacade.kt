package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules

/**
 * How the user capitalized the word being typed, and how suggestions are made to match it
 * (the same three cases as AOSP `Suggest.getTransformedSuggestedWordInfo`). The rules are in Rust
 * (lib/rust/crates/suggest), which cases text as Locale.US does.
 */
enum class WordCase(private val rust: SuggestRules.Case) {
    /** Lower case, or mixed like "iPhone": suggestions keep the dictionary's spelling. */
    AS_TYPED(SuggestRules.Case.AS_TYPED),

    /** Only the first letter is upper case (often auto-capitalization): capitalize suggestions. */
    CAPITALIZED(SuggestRules.Case.CAPITALIZED),

    /** Two or more letters, all upper case: upper-case suggestions. */
    ALL_CAPS(SuggestRules.Case.ALL_CAPS);

    /** [word] in this case: as it is, with its first code point upper-cased, or all upper case. */
    fun apply(word: String): String = SuggestRules.applyCase(rust, word)

    companion object {
        /**
         * The case of [typed]: all caps if it has two or more letters and all are upper case,
         * capitalized if its only capital is its first character, otherwise as typed.
         */
        fun of(typed: String): WordCase = when (SuggestRules.case(typed)) {
            SuggestRules.Case.AS_TYPED -> AS_TYPED
            SuggestRules.Case.CAPITALIZED -> CAPITALIZED
            SuggestRules.Case.ALL_CAPS -> ALL_CAPS
        }

        /** AOSP `WordComposer.isMostlyCaps`: more than one capital letter; never auto-corrected. */
        fun isMostlyCaps(typed: String): Boolean = SuggestRules.isMostlyCaps(typed)

        /**
         * The form handed to the decoder. A leading capital is usually auto-capitalization, so
         * "Im" at the start of a sentence is looked up as "im" (whitelisted to "I'm") and the
         * capital is put back on the results by [apply]. All caps is looked up in lower case.
         */
        fun decoderInput(typed: String): String = SuggestRules.decoderInput(typed)
    }
}
