package com.jtn0123.audiowhisper.rust

/**
 * Dictation's text rules' entry points in libawnative: lib/rust/crates/awnative/src/dictation.rs,
 * which documents each and how results are packed into strings. [DictationRules] unpacks them.
 */
internal object DictationNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun parse(phrase: String): Array<String>

    @JvmStatic external fun join(before: String, lastPhrase: String?, phrase: String, names: DictationNames): String

    @JvmStatic external fun punctuate(before: String, lastPhrase: String?, mark: Char): String

    @JvmStatic external fun revertsAt(typed: String, start: Int, cursor: Int, textBeforeCursor: String): Boolean
}
