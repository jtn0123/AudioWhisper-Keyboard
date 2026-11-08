package com.jtn0123.audiowhisper.rust

/**
 * Kotlin's character and string rules as the Rust (lib/rust/crates/chars) has them, for tests
 * that hold them up against `java.lang.Character` and `String`: on a computer, in lib/rust's JVM
 * tests, and on a phone, whose Java answers some characters differently. The keyboard itself
 * never calls these; its Rust uses the crate directly.
 */
object JavaText {
    /** What Rust says of one `Char`: its predicates, its single-char cases and its NFD. */
    class Unit(private val bits: Int, val uppercase: Char, val lowercase: Char, val decomposedFirst: Int) {
        val isLetter get() = has(JavaTextNative.LETTER)
        val isDigit get() = has(JavaTextNative.DIGIT)
        val isLetterOrDigit get() = has(JavaTextNative.LETTER_OR_DIGIT)
        val isUpperCase get() = has(JavaTextNative.UPPER_CASE)
        val isLowerCase get() = has(JavaTextNative.LOWER_CASE)
        val isTitleCase get() = has(JavaTextNative.TITLE_CASE)
        val isWhitespace get() = has(JavaTextNative.WHITESPACE)

        /** Whether `\s` in a `Regex` matches it. */
        val isRegexSpace get() = has(JavaTextNative.REGEX_SPACE)

        private fun has(bit: Int) = bits and bit != 0
    }

    /** What Rust says of one code point: `Character`'s functions that take an `Int`. */
    class CodePoint(val isDigit: Boolean, val digit: Int, val upperCase: Int, val lowerCase: Int)

    fun unit(char: Char): Unit {
        val facts = JavaTextNative.unitFacts(char)
        return Unit(facts[0], facts[1].toChar(), facts[2].toChar(), facts[3])
    }

    fun codePoint(codePoint: Int): CodePoint {
        val facts = JavaTextNative.codePointFacts(codePoint)
        return CodePoint(facts[0] != 0, facts[1], facts[2], facts[3])
    }

    fun lowercase(text: String): String = JavaTextNative.lowercase(text)

    fun uppercase(text: String): String = JavaTextNative.uppercase(text)

    fun trim(text: String): String = JavaTextNative.trim(text)

    fun equalsIgnoreCase(a: String, b: String): Boolean = JavaTextNative.equalsIgnoreCase(a, b)

    fun startsWithIgnoreCase(text: String, prefix: String): Boolean =
        JavaTextNative.startsWithIgnoreCase(text, prefix)

    fun isBlank(text: String): Boolean = JavaTextNative.isBlank(text)

    fun codePointCount(text: String): Int = JavaTextNative.codePointCount(text)

    fun toIntOrNull(text: String): Int? =
        JavaTextNative.toIntOrNull(text).takeIf { it != JavaTextNative.NO_INT }?.toInt()
}
