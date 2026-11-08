package com.jtn0123.audiowhisper.rust

import java.text.Normalizer
import kotlin.random.Random

/**
 * Holds [JavaText], the Rust, up against `java.lang.Character` and `String` wherever this runs:
 * every `Char`, every code point, every code point as a string, and random strings of the
 * characters whose case is hardest. Self-contained, so a device check can copy it into the app to
 * run the same comparison on a phone.
 *
 * [sigmaExact] says whether to compare how Σ lower-cases. Rust follows ICU's rule for whether a
 * Σ ends a word (ς) or not (σ), as Android does; the JVM uses its own word-break iterator, so on
 * the JVM, ς and σ are counted as one when comparing lower case.
 */
class JavaTextComparison(private val sigmaExact: Boolean) {
    /** How many comparisons failed, and the first few, described. */
    var failures = 0
        private set
    val examples = mutableListOf<String>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    private val regexSpace = Regex("\\s")

    fun everything(seed: Long, strings: Int) {
        everyChar()
        everyCodePoint()
        everyCodePointAsAString()
        randomStrings(Random(seed), strings)
        numbers(Random(seed), strings / 4)
    }

    private fun <T> check(what: String, input: String, java: T, rust: T) {
        checks++
        if (java == rust) return
        failures++
        if (examples.size < MAX_EXAMPLES) examples += "$what(${hex(input)}): Java ${show(java)}, Rust ${show(rust)}"
    }

    fun everyChar() {
        for (code in 0..0xFFFF) {
            val char = code.toChar()
            val rust = JavaText.unit(char)
            val text = char.toString()
            check("isLetter", text, char.isLetter(), rust.isLetter)
            check("isDigit", text, char.isDigit(), rust.isDigit)
            check("isLetterOrDigit", text, char.isLetterOrDigit(), rust.isLetterOrDigit)
            check("isUpperCase", text, char.isUpperCase(), rust.isUpperCase)
            check("isLowerCase", text, char.isLowerCase(), rust.isLowerCase)
            check("isTitleCase", text, char.isTitleCase(), rust.isTitleCase)
            check("isWhitespace", text, char.isWhitespace(), rust.isWhitespace)
            check("\\s", text, regexSpace.matches(text), rust.isRegexSpace)
            check("uppercaseChar", text, char.uppercaseChar(), rust.uppercase)
            check("lowercaseChar", text, char.lowercaseChar(), rust.lowercase)
            check("NFD", text, Normalizer.normalize(text, Normalizer.Form.NFD).codePointAt(0), rust.decomposedFirst)
        }
    }

    fun everyCodePoint() {
        for (codePoint in (-1..Character.MAX_CODE_POINT + 1) + listOf(Int.MIN_VALUE, Int.MAX_VALUE)) {
            val rust = JavaText.codePoint(codePoint)
            val text = "U+${Integer.toHexString(codePoint)}"
            check("isDigit", text, Character.isDigit(codePoint), rust.isDigit)
            check("digit", text, Character.digit(codePoint, 10), rust.digit)
            check("toUpperCase", text, Character.toUpperCase(codePoint), rust.upperCase)
            check("toLowerCase", text, Character.toLowerCase(codePoint), rust.lowerCase)
        }
    }

    fun everyCodePointAsAString() {
        for (codePoint in 0..Character.MAX_CODE_POINT) {
            val text = String(Character.toChars(codePoint))
            strings(text)
            for (other in caseVariants(codePoint)) pair(text, other)
        }
    }

    /** One code point's other cases, as Java's single-code-point functions give them. */
    private fun caseVariants(codePoint: Int) = listOf(
        Character.toUpperCase(codePoint),
        Character.toLowerCase(codePoint),
        Character.toTitleCase(codePoint),
    ).filter { it != codePoint }.map { String(Character.toChars(it)) }

    /** Everything that takes one string. */
    fun strings(text: String) {
        check("lowercase", text, sigma(text.lowercase()), sigma(JavaText.lowercase(text)))
        check("uppercase", text, text.uppercase(), JavaText.uppercase(text))
        check("trim", text, text.trim(), JavaText.trim(text))
        check("isBlank", text, text.isBlank(), JavaText.isBlank(text))
        check("codePointCount", text, text.codePointCount(0, text.length), JavaText.codePointCount(text))
        check("toIntOrNull", text, text.toIntOrNull(), JavaText.toIntOrNull(text))
    }

    /** Everything that takes two, with [other] as both the second string and a prefix. */
    fun pair(text: String, other: String) {
        val both = "$text|$other"
        check("equals", both, text.equals(other, ignoreCase = true), JavaText.equalsIgnoreCase(text, other))
        for (end in 0..other.length) {
            val prefix = other.substring(0, end)
            check(
                "startsWith",
                "$text|$prefix",
                text.startsWith(prefix, ignoreCase = true),
                JavaText.startsWithIgnoreCase(text, prefix),
            )
        }
    }

    private fun sigma(text: String) = if (sigmaExact) text else text.replace(FINAL_SIGMA, SMALL_SIGMA)

    fun randomStrings(random: Random, count: Int) {
        repeat(count) {
            val pieces = List(random.nextInt(MAX_PIECES + 1)) { PIECES.random(random) }
            val text = pieces.joinToString("")
            strings(text)
            pair(text, pieces.joinToString("") { recase(it, random) })
            pair(text, PIECES.random(random) + text)
        }
    }

    /** [piece] in another case, or unchanged, or now and then another piece altogether. */
    private fun recase(piece: String, random: Random): String {
        val codePoint = piece.codePointAt(0)
        return when (random.nextInt(8)) {
            0 -> piece.uppercase()
            1 -> piece.lowercase()
            2 -> String(Character.toChars(Character.toUpperCase(codePoint)))
            3 -> String(Character.toChars(Character.toLowerCase(codePoint)))
            4 -> String(Character.toChars(Character.toTitleCase(codePoint)))
            5 -> PIECES.random(random)
            else -> piece
        }
    }

    fun numbers(random: Random, count: Int) {
        for (text in NUMBER_EDGES) strings(text)
        repeat(count) {
            strings(List(random.nextInt(MAX_NUMBER_LENGTH + 1)) { NUMBER_PIECES.random(random) }.joinToString(""))
        }
    }

    companion object {
        private const val MAX_EXAMPLES = 20
        private const val MAX_PIECES = 6
        private const val MAX_NUMBER_LENGTH = 12
        private const val SMALL_SIGMA = "σ"
        private const val FINAL_SIGMA = "ς"

        /**
         * Characters whose case is hard: Σ and the letters, marks and punctuation around it;
         * the ones whose full case is longer (ß, ŉ, ǰ, ﬁ, ΐ, ᾳ, և) or whose case maps outside
         * their script's block (K, Å, µ, ſ, ı, İ, ͅ, ẞ, ϴ, ᲀ, Georgian, Cherokee); title case;
         * whitespace Java and Kotlin disagree on; digits of other scripts; supplementary letters,
         * marks and digits; and lone surrogates.
         */
        val PIECES = listOf(
            "a", "z", "A", "Z", "i", "I", "k", "K", "s", "S", "0", "9", " ", "\t", "'", ".", ":", "-", "+", "\"",
            "ß", "ẞ", "é", "É", "ÿ", "Ÿ", "µ", "ı", "İ", "ŉ", "ǰ", "ǅ", "ǆ", "Ǆ", "ǈ", "ſ", "ﬁ", "K", "Å",
            "å", "Σ", "Σ", "Σ", "σ", "ς", "Α", "α", "ΐ", "ᾳ", "ᾼ", "ϐ", "β", "ͅ", "ϴ", "θ", "Я", "я", "ᲀ",
            "ა", "Ა", "Ⴀ", "ⴀ", "Ꭰ", "ꭰ", "ᏸ", "և", "́", "̇", "­", "‍", "’", "·", " ",
            " ", "　", "\u0085", "᠎", " ", "\u001C", "٣", "９", "中", "Ⓐ", "ⓐ", "Ⅻ", "ⅻ", "ª",
            "ʰ", "𐐀", "𐐨", "𞤀", "𞤢", "𝟎", "😀",
            "𝅧", "𝅥", "𑢠", "𑣀", "\uD801", "\uDC00", "\uDC28",
        )

        private val NUMBER_PIECES = listOf(
            "0", "1", "2", "5", "7", "9", "-", "+", "٣", "９", "𝟎", "a", " ", "²", "٠",
        )

        private val NUMBER_EDGES = listOf(
            "", "-", "+", "0", "-0", "+0", "2147483647", "2147483648", "-2147483648", "-2147483649",
            "9999999999", "99999999999999999999", "00000000000000000001", "+-1", "-+1", "1-",
        )

        private fun hex(text: String) = text.map { "%04X".format(it.code) }.joinToString(" ")

        private fun show(value: Any?) = if (value is String) "\"${hex(value)}\"" else value.toString()
    }
}
