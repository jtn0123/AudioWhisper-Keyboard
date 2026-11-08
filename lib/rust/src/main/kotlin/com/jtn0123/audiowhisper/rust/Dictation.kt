package com.jtn0123.audiowhisper.rust

/**
 * Says whether a capitalised word is a name ("John"), which keeps its capital mid-sentence. The
 * Rust calls [isName] by name, so lib/rust's consumer rules keep it through R8.
 */
fun interface DictationNames {
    fun isName(word: String): Boolean
}

/**
 * Dictation's text rules, in Rust (lib/rust/crates/dictation): finding spoken commands in a
 * transcribed phrase, joining phrases that were transcribed separately, and whether a backspace can
 * still take back the phrase typed last. The keyboard's `Spoken`, `Phrases` and `DictationUndo`
 * call these; their documentation explains the rules.
 */
object DictationRules {
    /** What a part of a phrase asks for. */
    enum class Kind { WORDS, MARK, BREAK, SCRATCH_THAT }

    /** A part of a phrase: its [kind], and its words, its mark or its line breaks as [text]. */
    class Part(val kind: Kind, val text: String)

    /** Delete [replacing] characters before the cursor, then type [text]. */
    class Typing(val replacing: Int, val text: String)

    /** [phrase]'s words and commands, in order. */
    fun parse(phrase: String): List<Part> = DictationNative.parse(phrase).map { part ->
        val kind = when (part[0]) {
            'w' -> Kind.WORDS
            'm' -> Kind.MARK
            'b' -> Kind.BREAK
            's' -> Kind.SCRATCH_THAT
            else -> error("unknown part")
        }
        Part(kind, part.substring(1))
    }

    /**
     * How to type [phrase] after [before], the text up to the cursor, when [lastPhrase] is what
     * dictation typed last. [names] is asked at most once.
     */
    fun join(before: String, lastPhrase: String?, phrase: String, names: DictationNames): Typing =
        typing(DictationNative.join(before, lastPhrase, phrase, names))

    /** How to type the spoken punctuation [mark] after [before]. */
    fun punctuate(before: String, lastPhrase: String?, mark: Char): Typing =
        typing(DictationNative.punctuate(before, lastPhrase, mark))

    /**
     * Whether a backspace with the cursor at [cursor], after [textBeforeCursor], can still take
     * back [typed], the phrase dictation typed at [start].
     */
    fun revertsAt(typed: String, start: Int, cursor: Int, textBeforeCursor: String): Boolean =
        DictationNative.revertsAt(typed, start, cursor, textBeforeCursor)

    private fun typing(packed: String) = Typing(packed[0].code, packed.substring(1))
}
