package com.jtn0123.audiowhisper.dictation

/**
 * Dictation's text rules as they were in Kotlin before moving to Rust (lib/rust/crates/dictation),
 * copied unchanged from commit 2b55078b: Spoken.kt, Phrases.kt and DictationUndo.kt. They are what
 * [DictationRulesMatchTest] holds the Rust to; don't change them.
 */
object ReferenceDictation {
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

    /**
     * Joining phrases that were transcribed separately.
     *
     * Each pause is transcribed on its own, so the model treats every phrase as a whole
     * sentence: it capitalises the first word and usually ends with a full stop, even
     * where the speaker only paused ("I need to talk to." "John about it.").
     */
    object Phrases {
        /** Delete [replacing] characters before the cursor (a full stop this dictation typed), then type [text]. */
        data class Typing(val replacing: Int, val text: String)

        /**
         * How to type [phrase] after [before], the text up to the cursor. [lastPhrase] is what
         * this dictation typed last: when the cursor is still right after it, its full stop is
         * taken back if [phrase] carries the sentence on. [isName] says a capitalised word is
         * a name ("John"), which keeps its capital mid-sentence.
         */
        fun join(before: CharSequence?, lastPhrase: String?, phrase: String, isName: (String) -> Boolean): Typing {
            val text = before?.toString().orEmpty()
            // A full stop said out loud ("period") has no words with it, and is never taken back.
            val takesBackStop = lastPhrase != null && lastPhrase.endsWith('.') && lastPhrase.any(Char::isLetterOrDigit) &&
                isRightAfter(text, lastPhrase) && carriesOn(lastPhrase, phrase)
            val kept = if (takesBackStop) text.dropLast(1) else text
            val body = when {
                startsLine(kept) || endsSentence(kept) -> phrase.replaceFirstChar { it.uppercaseChar() }
                continuesSentence(kept) -> lowerFirst(phrase, isName)
                else -> phrase
            }
            val lead = if (kept.isEmpty() || kept.last().isWhitespace()) "" else " "
            return Typing(if (takesBackStop) 1 else 0, lead + body)
        }

        /**
         * How to type a spoken punctuation [mark] ("question mark"): in place of the punctuation
         * that ends [lastPhrase] if the cursor is still right after it ("See you soon." → "See you
         * soon?"), otherwise straight after the text.
         */
        fun punctuate(before: CharSequence?, lastPhrase: String?, mark: Char): Typing {
            val text = before?.toString().orEmpty()
            val replaces = lastPhrase != null && lastPhrase.lastOrNull() in MARKS && isRightAfter(text, lastPhrase)
            return Typing(if (replaces) 1 else 0, mark.toString())
        }

        /** Only what the editor shows before the cursor can be compared, so match as much of [phrase] as that. */
        private fun isRightAfter(text: String, phrase: String): Boolean =
            text.isNotEmpty() && text.endsWith(phrase.takeLast(text.length))

        /**
         * The speaker paused mid-sentence: [last] stops on a word a sentence can't end on ("talk to."),
         * the model wrote [next] in lower case (it heard a fragment), or [next] opens with "and" or the like.
         */
        private fun carriesOn(last: String, next: String): Boolean {
            val lastWord = last.dropLast(1).substringAfterLast(' ').lowercase()
            val firstWord = firstWord(next)
            return lastWord in OPEN_ENDINGS || firstWord.firstOrNull()?.isLowerCase() == true ||
                firstWord.lowercase() in JOINERS
        }

        /** The cursor is at the start of a line below text ("new line"); the empty field is left to the model. */
        private fun startsLine(text: String): Boolean = '\n' in text && text.substringAfterLast('\n').isBlank()

        private fun continuesSentence(text: String): Boolean {
            val last = text.lastOrNull { !it.isWhitespace() } ?: return false
            return last.isLetterOrDigit() || last == ',' || last == ';'
        }

        private fun endsSentence(text: String): Boolean = text.lastOrNull { !it.isWhitespace() } in SENTENCE_ENDS

        /** Lower-cases the first word unless it is "I", an acronym, or a name. */
        private fun lowerFirst(phrase: String, isName: (String) -> Boolean): String {
            val word = firstWord(phrase)
            val keepsCapital = word.isEmpty() || word == "I" || word.startsWith("I'") ||
                (word.length > 1 && word[1].isUpperCase()) || isName(word)
            return if (keepsCapital) phrase else phrase.replaceFirstChar { it.lowercaseChar() }
        }

        private fun firstWord(text: String): String = text.takeWhile { it.isLetter() || it == '\'' || it == '’' }

        private val SENTENCE_ENDS = setOf('.', '!', '?')

        /** Punctuation the model ends a phrase with, which a spoken mark replaces. */
        private val MARKS = setOf('.', ',', ';', ':', '!', '?')

        /** Words a spoken sentence practically never ends on, so a full stop after one is a pause. */
        private val OPEN_ENDINGS = setOf(
            "a", "an", "the", "and", "or", "but", "nor", "of", "to", "for", "with", "about", "from",
            "into", "onto", "at", "as", "than", "because", "if", "my", "your", "our", "their", "its", "i",
        )

        /** First words that join on to the sentence before even when the model capitalised them. */
        private val JOINERS = setOf("and", "but", "or", "nor", "because")
    }

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

        /** How to undo with the cursor at [cursor] after [textBeforeCursor], or null if it has moved or the text changed. */
        fun revertAt(cursor: Int, textBeforeCursor: CharSequence): Revert? {
            if (typed.isEmpty() || cursor != start + typed.length) return null
            if (!textBeforeCursor.endsWith(typed)) return null
            return Revert(typed.length, restore)
        }
    }
}
