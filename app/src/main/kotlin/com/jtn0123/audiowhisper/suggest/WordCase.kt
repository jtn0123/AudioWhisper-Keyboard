package com.jtn0123.audiowhisper.suggest

import java.util.Locale

/**
 * How the user capitalized the word being typed, and how suggestions are made to match it
 * (the same three cases as AOSP `Suggest.getTransformedSuggestedWordInfo`).
 */
enum class WordCase {
    /** Lower case, or mixed like "iPhone": suggestions keep the dictionary's spelling. */
    AS_TYPED,

    /** Only the first letter is upper case (often auto-capitalization): capitalize suggestions. */
    CAPITALIZED,

    /** Two or more letters, all upper case: upper-case suggestions. */
    ALL_CAPS;

    /** [word] in this case. */
    fun apply(word: String, locale: Locale): String = when (this) {
        AS_TYPED -> word
        CAPITALIZED -> capitalizeFirst(word, locale)
        ALL_CAPS -> word.uppercase(locale)
    }

    companion object {
        /** The case of [typed]. */
        fun of(typed: String): WordCase {
            val letters = typed.filter(Char::isLetter)
            val upper = letters.count(Char::isUpperCase)
            return when {
                letters.length > 1 && upper == letters.length -> ALL_CAPS
                upper == 1 && typed.isNotEmpty() && typed[0].isUpperCase() -> CAPITALIZED
                else -> AS_TYPED
            }
        }

        /** AOSP `WordComposer.isMostlyCaps`: more than one capital letter; never auto-corrected. */
        fun isMostlyCaps(typed: String): Boolean = typed.count(Char::isUpperCase) > 1

        /**
         * The form handed to the decoder. A leading capital is usually auto-capitalization, so
         * "Im" at the start of a sentence is looked up as "im" (whitelisted to "I'm") and the
         * capital is put back on the results by [apply].
         */
        fun decoderInput(typed: String, locale: Locale): String = when (of(typed)) {
            AS_TYPED -> typed
            CAPITALIZED -> lowerFirst(typed, locale)
            ALL_CAPS -> typed.lowercase(locale)
        }

        private fun capitalizeFirst(word: String, locale: Locale): String {
            if (word.isEmpty()) return word
            val end = word.offsetByCodePoints(0, 1)
            return word.substring(0, end).uppercase(locale) + word.substring(end)
        }

        private fun lowerFirst(word: String, locale: Locale): String {
            if (word.isEmpty()) return word
            val end = word.offsetByCodePoints(0, 1)
            return word.substring(0, end).lowercase(locale) + word.substring(end)
        }
    }
}
