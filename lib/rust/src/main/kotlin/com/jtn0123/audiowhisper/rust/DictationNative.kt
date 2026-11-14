package com.jtn0123.audiowhisper.rust

/**
 * Dictation's text rules' entry points in libawnative: lib/rust/crates/awnative/src/dictation.rs,
 * which documents each and how results are packed into strings. [DictationRules] unpacks them,
 * and [MicKeyState] wraps the mic key's.
 */
internal object DictationNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun parse(phrase: String): Array<String>


    @JvmStatic external fun punctuate(before: String, lastPhrase: String?, mark: Char): String

    @JvmStatic external fun revertsAt(typed: String, start: Int, cursor: Int, textBeforeCursor: String): Boolean

    @JvmStatic external fun format(phrase: String, vocabulary: String, removeFillers: Boolean): String

    @JvmStatic external fun holdMs(): Long

    @JvmStatic external fun newMicKey(): Long

    @JvmStatic external fun freeMicKey(key: Long)

    @JvmStatic external fun micKeyPress(key: Long, status: Int, now: Long): Int

    @JvmStatic external fun micKeyRelease(key: Long, now: Long): Int
}
