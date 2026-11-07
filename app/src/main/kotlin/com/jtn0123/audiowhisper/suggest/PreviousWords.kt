package com.jtn0123.audiowhisper.suggest

/**
 * The n-gram context before the word being typed: up to [MAX_COUNT] previous words, most recent
 * first, and whether a beginning-of-sentence marker follows them. Mirrors AOSP `NgramContext`.
 */
data class PreviousWords(val words: List<String>, val isBeginningOfSentence: Boolean) {
    /** Native n-gram slots: each word, then the sentence-start marker if there is room. */
    val slotCount: Int
        get() = (words.size + if (isBeginningOfSentence) 1 else 0).coerceAtMost(MAX_COUNT)

    companion object {
        /** Native MAX_PREV_WORD_COUNT_FOR_N_GRAM. */
        const val MAX_COUNT = 3

        /** No usable context (for example after a comma). */
        val NONE = PreviousWords(emptyList(), isBeginningOfSentence = false)

        private const val SENTENCE_TERMINATORS = ".?!"
        private const val WORD_CONNECTORS = "'’-"
        private val WHITESPACE = Regex("\\s+")

        /** Stands in for the word being typed, so a prefix glued to it ("(word") stays with it. */
        private const val CURRENT_WORD = ""

        /**
         * The context for a word that starts right after [textBeforeWord]. Same rules as AOSP
         * `NgramContextUtils.getNgramContextFromNthPreviousWord`: only the current line counts,
         * words are split on whitespace, the start of the line or a word ending in `.?!` is a
         * sentence start, and any other trailing punctuation leaves the context unknown.
         */
        fun from(textBeforeWord: CharSequence): PreviousWords {
            val line = textBeforeWord.toString().substringAfterLast('\n')
            val tokens = (line + CURRENT_WORD).split(WHITESPACE)
            val words = ArrayList<String>(MAX_COUNT)
            for (i in 0 until MAX_COUNT) {
                val index = tokens.size - 2 - i
                val following = tokens[index + 1].firstOrNull()
                if (following != null && following in WORD_CONNECTORS) break
                if (index < 0) return PreviousWords(words, isBeginningOfSentence = true)
                val token = tokens[index]
                if (token.isEmpty() || token.last() in SENTENCE_TERMINATORS) {
                    return PreviousWords(words, isBeginningOfSentence = true)
                }
                if (!token.last().isLetterOrDigit()) break
                words += token
            }
            return PreviousWords(words, isBeginningOfSentence = false)
        }
    }
}
