package com.jtn0123.audiowhisper.dictation

/** Joining phrases that were transcribed separately. */
object Phrases {
    /**
     * The model capitalises the start of every phrase, but a pause is not
     * always the end of a sentence. When [before] (the text just typed)
     * doesn't end one, lower-case the first letter of [text] — unless it is
     * "I" or a word in capitals, like an acronym.
     */
    fun continueSentence(before: CharSequence?, text: String): String {
        val last = before?.lastOrNull { !it.isWhitespace() } ?: return text
        if (!last.isLetterOrDigit() && last != ',') return text
        val word = text.substringBefore(' ')
        val keepsCapital = word == "I" || word.startsWith("I'") ||
            (word.length > 1 && word[1].isUpperCase())
        return if (keepsCapital) text else text.replaceFirstChar { it.lowercaseChar() }
    }
}
