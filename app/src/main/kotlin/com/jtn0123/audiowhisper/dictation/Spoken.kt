package com.jtn0123.audiowhisper.dictation

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
         * because the model only writes it that way when the speaker says it on its own.
         */
        fun parse(phrase: String): List<Spoken> {
            val parts = ArrayList<Spoken>()
            val words = ArrayList<String>()
            fun flushWords() {
                if (words.isNotEmpty()) parts += Words(words.joinToString(" "))
                words.clear()
            }
            for (sentence in phrase.trim().split(SENTENCE_BREAK)) {
                val command = COMMANDS[key(sentence)]
                if (command == null) {
                    if (sentence.isNotBlank()) words += sentence
                } else {
                    flushWords()
                    parts += command
                }
            }
            flushWords()
            return parts
        }

        private fun key(sentence: String) =
            sentence.trim().trimEnd('.', '!', '?', ',').trim().lowercase().replace(Regex("\\s+"), " ")

        private val SENTENCE_BREAK = Regex("(?<=[.!?])\\s+")

        private val COMMANDS: Map<String, Spoken> = mapOf(
            "new line" to Break("\n"), "newline" to Break("\n"), "next line" to Break("\n"),
            "new paragraph" to Break("\n\n"),
            "scratch that" to ScratchThat, "delete that" to ScratchThat,
            "period" to Mark('.'), "full stop" to Mark('.'), "comma" to Mark(','),
            "question mark" to Mark('?'), "exclamation mark" to Mark('!'), "exclamation point" to Mark('!'),
            "colon" to Mark(':'), "semicolon" to Mark(';'),
        )
    }
}
