package com.jtn0123.audiowhisper.rust

/**
 * Says whether a capitalised word is a name ("John"), which keeps its capital mid-sentence. The
 * Rust calls [isName] by name, so lib/rust's consumer rules keep it through R8.
 */
fun interface DictationNames {
    fun isName(word: String): Boolean
}

object LegacyDictation {
    /**
     * How to type [phrase] after [before], the text up to the cursor, when [lastPhrase] is what
     * dictation typed last. [names] is asked at most once.
     */
    fun join(before: String, lastPhrase: String?, phrase: String, names: DictationNames): DictationRules.Typing =
        LegacyFixtureNative.join(before, lastPhrase, phrase, names).let { DictationRules.Typing(it[0].code, it.substring(1)) }

}
