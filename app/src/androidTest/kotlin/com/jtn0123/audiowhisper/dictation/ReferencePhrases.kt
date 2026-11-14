package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.LegacyDictation
import com.jtn0123.audiowhisper.rust.DictationRules

/**
 * Joining phrases that were transcribed separately.
 *
 * Each pause is transcribed on its own, so the model treats every phrase as a whole
 * sentence: it capitalises the first word and usually ends with a full stop, even
 * where the speaker only paused ("I need to talk to." "John about it.").
 *
 * The rules are in Rust: lib/rust/crates/dictation/src/phrases.rs.
 */
object Phrases {
    /** Delete [replacing] characters before the cursor (a full stop this dictation typed), then type [text]. */
    data class Typing(val replacing: Int, val text: String)

    /**
     * How to type [phrase] after [before], the text up to the cursor. [lastPhrase] is what
     * this dictation typed last: when the cursor is still right after it, its full stop is
     * taken back if [phrase] carries the sentence on ("talk to." then "John about it", or a
     * phrase starting in lower case or with "and"). Otherwise the phrase starts a sentence
     * after one that ended, and is lowered mid-sentence unless its first word is "I", an
     * acronym, or a name: [isName] says a capitalised word is one ("John"), and is asked at
     * most once.
     */
    fun join(before: CharSequence?, lastPhrase: String?, phrase: String, isName: (String) -> Boolean): Typing =
        typing(LegacyDictation.join(before?.toString().orEmpty(), lastPhrase, phrase) { isName(it) })

    /**
     * How to type a spoken punctuation [mark] ("question mark"): in place of the punctuation
     * that ends [lastPhrase] if the cursor is still right after it ("See you soon." → "See you
     * soon?"), otherwise straight after the text.
     */
    fun punctuate(before: CharSequence?, lastPhrase: String?, mark: Char): Typing =
        typing(DictationRules.punctuate(before?.toString().orEmpty(), lastPhrase, mark))

    private fun typing(typing: DictationRules.Typing) = Typing(typing.replacing, typing.text)
}
