package com.jtn0123.audiowhisper.rust

/**
 * The `chars` crate's entry points in libawnative: lib/rust/crates/awnative/src/text.rs, which
 * documents each and how results are packed. [JavaText] unpacks them.
 */
internal object JavaTextNative {
    init {
        RustLibrary.load()
    }

    // The bits of unitFacts' first value.
    const val LETTER = 1
    const val DIGIT = 2
    const val LETTER_OR_DIGIT = 4
    const val UPPER_CASE = 8
    const val LOWER_CASE = 16
    const val TITLE_CASE = 32
    const val WHITESPACE = 64
    const val REGEX_SPACE = 128

    /** What [toIntOrNull] returns for null. */
    const val NO_INT = Long.MIN_VALUE

    @JvmStatic external fun unitFacts(unit: Char): IntArray

    @JvmStatic external fun codePointFacts(codePoint: Int): IntArray

    @JvmStatic external fun lowercase(text: String): String

    @JvmStatic external fun uppercase(text: String): String

    @JvmStatic external fun trim(text: String): String

    @JvmStatic external fun equalsIgnoreCase(a: String, b: String): Boolean

    @JvmStatic external fun startsWithIgnoreCase(a: String, b: String): Boolean

    @JvmStatic external fun isBlank(text: String): Boolean

    @JvmStatic external fun codePointCount(text: String): Int

    @JvmStatic external fun toIntOrNull(text: String): Long
}
