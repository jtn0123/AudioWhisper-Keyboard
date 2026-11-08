package com.jtn0123.audiowhisper.dictation

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * Dictation's text rules in Rust ([Spoken], [Phrases] and [DictationUndo], which call
 * lib/rust/crates/dictation) give exactly what the Kotlin they replaced gave ([ReferenceDictation]),
 * including which words they ask [Phrases.join]'s `isName` about.
 *
 * The rules judge characters as Kotlin does (`isLetter`, `isWhitespace`, `uppercaseChar`, ...),
 * which the Rust answers from tables generated from Java (lib/rust/crates/chars). So every UTF-16
 * unit is put through each rule, and then random phrases built from the characters and words the
 * rules care about. The tables are Unicode 16, as is the JDK 24 these tests run on.
 */
class DictationRulesMatchTest {
    private val names = setOf("John", "Paris", "I'm")

    private fun join(before: String?, last: String?, phrase: String): String {
        val asked = mutableListOf<String>()
        val typing = Phrases.join(before, last, phrase) { asked += it; it in names }
        return "${typing.replacing} [${typing.text}] asked $asked"
    }

    private fun referenceJoin(before: String?, last: String?, phrase: String): String {
        val asked = mutableListOf<String>()
        val typing = ReferenceDictation.Phrases.join(before, last, phrase) { asked += it; it in names }
        return "${typing.replacing} [${typing.text}] asked $asked"
    }

    private fun punctuate(before: String?, last: String?, mark: Char) =
        Phrases.punctuate(before, last, mark).let { "${it.replacing} [${it.text}]" }

    private fun referencePunctuate(before: String?, last: String?, mark: Char) =
        ReferenceDictation.Phrases.punctuate(before, last, mark).let { "${it.replacing} [${it.text}]" }

    private fun parse(phrase: String) = Spoken.parse(phrase).map {
        when (it) {
            is Spoken.Words -> "words [${it.text}]"
            is Spoken.Mark -> "mark ${it.mark}"
            is Spoken.Break -> "break [${it.text}]"
            Spoken.ScratchThat -> "scratch that"
        }
    }

    private fun referenceParse(phrase: String) = ReferenceDictation.Spoken.parse(phrase).map {
        when (it) {
            is ReferenceDictation.Spoken.Words -> "words [${it.text}]"
            is ReferenceDictation.Spoken.Mark -> "mark ${it.mark}"
            is ReferenceDictation.Spoken.Break -> "break [${it.text}]"
            ReferenceDictation.Spoken.ScratchThat -> "scratch that"
        }
    }

    private fun revert(typed: String, start: Int, cursor: Int, text: String) =
        DictationUndo(typed, start, ".").revertAt(cursor, text).toString()

    private fun referenceRevert(typed: String, start: Int, cursor: Int, text: String) =
        ReferenceDictation.DictationUndo(typed, start, ".").revertAt(cursor, text)
            ?.let { DictationUndo.Revert(it.length, it.text) }.toString()

    /** Collects what differs, so a failure shows the first few cases rather than only one. */
    private class Differences {
        var compared = 0
        val found = mutableListOf<String>()

        fun check(case: String, rust: Any, reference: Any) {
            compared++
            if (rust != reference) found += "$case: Rust gave $rust, Kotlin gave $reference"
        }

        fun assertNone() = assertEquals(
            emptyList(),
            found.take(20),
            "${found.size} of $compared cases differ",
        )
    }

    /**
     * Phrases that are a command when [c] reads as the letter it replaces or as a space, or is
     * trimmed: each letter the commands use, and the places trimming and squeezing look at.
     */
    private fun commandProbes(c: String) = listOf(
        "${c}ew line", "n${c}xt line", "new${c}line", "newli${c}e", "new p${c}ragraph", "new parag${c}aph",
        "scr${c}tch that", "scratc${c} that", "delete t${c}at", "${c}elete that", "pe${c}iod", "per${c}od",
        "${c}ull stop", "full ${c}top", "fu${c}l stop", "${c}omma", "co${c}ma", "question mar${c}",
        "${c}uestion mark", "q${c}estion mark", "e${c}clamation point", "exclamation poi${c}t",
        "exclamati${c}n mark", "co${c}on", "colon$c", "${c}colon", "semicolo${c}", "new  line$c$c",
    ).joinToString(". ")

    @Test
    fun everyCharacterIsJudgedAsKotlinJudgesIt() {
        val differences = Differences()
        for (code in 0..0xFFFF) {
            val c = code.toChar().toString()
            val joins = listOf(
                Triple("Hi. ", null, "${c}x"), // upper-casing a sentence's first letter
                Triple("hi", null, "${c}x"), // letters, lower-casing, and asking for a name
                Triple("hi", null, "A$c"), // an acronym's second letter
                Triple("x$c", null, "Ab"), // what a sentence goes on after, and whitespace
                Triple("$c.", "$c.", "and x"), // a phrase with a letter or digit in it
                Triple("Hi.", "Hi.", "${c}x"), // a phrase starting in lower case
                Triple("go $c.", "go $c.", "Xy"), // "a" and "i" end a phrase open
                Triple("go ${c}nto.", "go ${c}nto.", "Xy"),
                Triple("Hi.", "Hi.", "${c}nd x"), // "and" joins on
                Triple("Hi.", "Hi.", "${c}ecause x"),
                Triple("\n$c", null, "ab"), // a blank line
                Triple("a$c\n", "a$c\n", "ab"),
            )
            for ((before, last, phrase) in joins) {
                differences.check("join($before, $last, $phrase)", join(before, last, phrase), referenceJoin(before, last, phrase))
            }
            for ((before, last, mark) in listOf(Triple("x$c", "x$c", '!'), Triple(c, c, '?'), Triple("y", "y", c[0]))) {
                differences.check("punctuate($before, $last, $mark)", punctuate(before, last, mark), referencePunctuate(before, last, mark))
            }
            for (phrase in listOf(commandProbes(c), "${c}new line$c. Hi.${c}there.$c${c}Comma$c", c)) {
                differences.check("parse($phrase)", parse(phrase), referenceParse(phrase))
            }
        }
        differences.assertNone()
    }

    /** Characters and words the rules treat specially, and some they should treat as any other. */
    private val pieces = listOf(
        " ", " ", " ", "  ", "\t", "\n", "\u000B", "\u000C", "\r", "\u001C", "\u0085", " ", " ",
        " ", " ", "　", " ", ".", ".", ".", "!", "?", ",", ";", ":", "'", "’",
        "a", "A", "i", "I", "k", "x", "İ", "ı", "ß", "K", "Σ", "σ", "ǅ", "ᾳ", "é", "É", "0", "٣", "²",
        "Ⅻ", "😀", "\uD800", "\uDC00", "́", "\u0000",
        "new", "New", "NEW", "line", "Line", "newline", "next", "paragraph", "scratch", "Scratch", "that",
        "delete", "period", "Period", "full", "stop", "comma", "Comma", "question", "mark", "Mark",
        "exclamation", "point", "colon", "semicolon", "and", "And", "But", "because", "Nor", "to", "the",
        "THE", "I'm", "I’d", "John", "Paris", "NASA", "Talk", "talk", "an", "into", "Its",
    )

    private fun Random.text(maxPieces: Int) = buildString { repeat(nextInt(maxPieces + 1)) { append(pieces.random(this@text)) } }

    @Test
    fun randomPhrasesGiveTheSame() {
        val random = Random(2026_11_08)
        val differences = Differences()
        repeat(100_000) {
            val phrase = random.text(12)
            differences.check("parse($phrase)", parse(phrase), referenceParse(phrase))

            // Often the cursor is right after the last phrase, or right after part of it.
            val last = if (random.nextInt(4) == 0) null else random.text(6)
            val before = when (random.nextInt(5)) {
                0 -> null
                1 -> random.text(4)
                2 -> last.orEmpty().takeLast(random.nextInt(8))
                else -> random.text(4) + last.orEmpty()
            }
            differences.check("join($before, $last, $phrase)", join(before, last, phrase), referenceJoin(before, last, phrase))
            val mark = ".,;:!?x".random(random)
            differences.check("punctuate($before, $last, $mark)", punctuate(before, last, mark), referencePunctuate(before, last, mark))

            val typed = random.text(3)
            val start = if (random.nextBoolean()) random.nextInt(0, 200) else Int.MAX_VALUE - random.nextInt(4)
            val cursor = start + typed.length + random.nextInt(-1, 2)
            val text = if (random.nextInt(3) == 0) random.text(4) else random.text(3) + typed
            differences.check("revert($typed, $start, $cursor, $text)", revert(typed, start, cursor, text), referenceRevert(typed, start, cursor, text))
        }
        differences.assertNone()
    }
}
