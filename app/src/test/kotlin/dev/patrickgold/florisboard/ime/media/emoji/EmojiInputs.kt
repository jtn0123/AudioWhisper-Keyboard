package dev.patrickgold.florisboard.ime.media.emoji

import kotlin.random.Random

/**
 * Random inputs for [EmojiComparison]: damaged emoji files, queries and composing texts, emoji
 * values with tones, and locales with lists of files. Each puts the odd cases where they reach the
 * output: spaces Kotlin's `trim` does and doesn't take, letters whose case maps to ASCII (ı, İ, ſ,
 * the Kelvin sign) or across a surrogate pair (Deseret), every line end, and Ints at their ends.
 */
object EmojiInputs {
    /** Spaces and near-spaces: Kotlin's `isWhitespace` takes the first eleven on the JVM, not the rest. */
    private val SPACES = listOf(
        " ", "\t", " ", " ", " ", "　", " ", "\u001C", "\u001F", "\u000B", "\u000C",
        "\u0085", "᠎", "​", "﻿",
    )

    /** Units that are odd in a value, a name or a keyword. */
    private val ODD = listOf(
        "\u0000", "\uD83C", "\uDFFB", "\uDBFF", "\uDC00", "🏽", "🦱", "￿", "İ", "ı", "ß", "𐐀", "𐐨", "ﬀ", "ǅ",
        "K", "ſ", "Σ", "ς", "é", "É", "́", "‍", "|", ";", "[", "#",
    )

    /** What tone detection looks for, and what it mustn't mistake for it. */
    private val TONE_PARTS = listOf(
        "😀", "👋", "🧑", "🏻", "🏼", "🏽", "🏾", "🏿", "🦰", "🦱", "🦲", "🦳", "‍", "\u0000",
        "\uD83C", "\uDFFB", "\uD83E", "\uDDB0", "a", "️", "\uDFFF\uD83C",
    )

    private val LINE_ENDS = listOf("\n", "\n", "\n", "\n", "\r\n", "\r")

    private fun header(random: Random): String {
        val id = EmojiCategory.entries.random(random).id
        return listOf(
            "[$id]", "[$id]", "[$id", "[$id]x", "[]", "[", "[${id.uppercase()}]", "[ $id]", "[$id ]", "[unknown]",
            "[recently_used]", "[${id}]\u0085",
        ).random(random)
    }

    private fun spaces(random: Random) = List(random.nextInt(3)) { SPACES.random(random) }.joinToString("")

    private fun insert(random: Random, line: String, piece: String): String {
        val at = random.nextInt(line.length + 1)
        return line.substring(0, at) + piece + line.substring(at)
    }

    /** Letters swapped for others whose case maps oddly, or the whole text's case changed. */
    private fun confuse(random: Random, text: String): String = when (random.nextInt(4)) {
        0 -> text.uppercase()
        1 -> text.map { if (random.nextBoolean()) it.uppercaseChar() else it }.joinToString("")
        else -> buildString {
            for (c in text) {
                append(
                    when {
                        random.nextInt(3) != 0 -> c.toString()
                        c == 'i' -> listOf("ı", "İ", "I").random(random)
                        c == 's' -> "ſ"
                        c == 'k' -> "K"
                        c == 'e' -> listOf("é", "E").random(random)
                        c == ' ' -> SPACES.random(random)
                        else -> c.toString()
                    },
                )
            }
        }
    }

    private fun damage(random: Random, line: String, out: MutableList<String>) {
        val fields = line.split(";")
        val emoji = fields.size == 3
        when (random.nextInt(13)) {
            0 -> Unit // dropped
            1 -> out += listOf(line, spaces(random))
            2 -> out += if (random.nextBoolean()) "#$line" else listOf("#", "# $line").random(random)
            3 -> out += listOf(header(random), line)
            4 -> out += if (line.startsWith("\t")) line.drop(1) else "\t$line"
            5 -> out += insert(random, line, ";")
            6 -> out += if (line.contains(';')) line.replaceFirst(";", "") else insert(random, line, "|")
            7 -> out += if (emoji) fields.joinToString(";") { spaces(random) + it + spaces(random) } else spaces(random) + line
            8 -> out += if (emoji && random.nextBoolean()) "${fields[0]};;${fields[2]}" else if (emoji) "${fields[0]};${fields[1]};" else line
            9 -> out += if (emoji) "${fields[0]};${confuse(random, fields[1])};${confuse(random, fields[2])}" else line
            10 -> out += if (emoji) "${insert(random, fields[0], TONE_PARTS.random(random))};${fields[1]};${fields[2]}" else line
            11 -> out += line.replace("|", listOf("||", "| |", "|\t|").random(random))
            else -> out += insert(random, line, ODD.random(random))
        }
    }

    /**
     * Some lines of a bundled file, usually under the category they were in, with about one line
     * in six damaged, joined with random line ends.
     */
    fun damagedFile(random: Random, text: String): String {
        val lines = text.split("\n")
        val start = random.nextInt(lines.size)
        val window = lines.subList(start, minOf(lines.size, start + 20 + random.nextInt(300)))
        val out = mutableListOf<String>()
        val header = lines.subList(0, start + 1).lastOrNull { it.startsWith("[") }
        if (header != null && random.nextInt(5) != 0) out += header
        for (line in window) {
            if (random.nextInt(6) == 0) damage(random, line, out) else out += line
        }
        return buildString {
            for ((i, line) in out.withIndex()) {
                append(line)
                if (i < out.size - 1 || random.nextBoolean()) append(LINE_ENDS.random(random))
            }
        }
    }

    /** The runs of ASCII letters in [text]. */
    private fun letterRuns(text: String) = Regex("[A-Za-z]+").findAll(text).map { it.value }.toList()

    /**
     * A query: usually part of a word of a random emoji's name or keywords, its case changed or not,
     * else random letters, nothing, something very long, or letters whose case maps oddly.
     */
    fun query(random: Random, emojis: List<ReferenceEmoji>): String {
        val emoji = emojis.randomOrNull(random)
        val words = emoji?.let { letterRuns(it.name) + it.keywords.flatMap(::letterRuns) }.orEmpty()
        return when (random.nextInt(12)) {
            in 0..5 -> words.randomOrNull(random)?.let { word ->
                val start = random.nextInt(word.length)
                val piece = word.substring(start, start + 1 + random.nextInt(word.length - start))
                if (random.nextBoolean()) piece else piece.map { if (random.nextBoolean()) it.uppercaseChar() else it.lowercaseChar() }.joinToString("")
            } ?: "smi"
            6 -> List(1 + random.nextInt(6)) { ('a' + random.nextInt(26)).let { if (random.nextBoolean()) it.uppercaseChar() else it } }.joinToString("")
            7 -> ""
            8 -> List(30 + random.nextInt(200)) { 'a' + random.nextInt(26) }.joinToString("")
            9 -> confuse(random, words.randomOrNull(random) ?: "kiss")
            10 -> List(1 + random.nextInt(4)) { ODD.random(random) }.joinToString("")
            else -> (words.randomOrNull(random) ?: "e") + ODD.random(random)
        }
    }

    /**
     * The composing text around a query: usually after the [prefix] the setting asks for, else
     * after another, with colons, spaces and line ends, or none of the query at all.
     */
    fun composing(random: Random, query: String, prefix: String): String = when (random.nextInt(14)) {
        in 0..6 -> "$prefix$query"
        7 -> ":$query"
        8 -> query
        9 -> "::$query"
        10 -> listOf(" :$query", ":$query ", ": $query", " $query").random(random)
        11 -> listOf(":$query:", "$query:", ":${query}\n", "${query}\n", ":$query\r").random(random)
        12 -> "$prefix${query.dropLast(1)}"
        else -> listOf("", ":", "::", " ", ":${SPACES.random(random)}").random(random)
    }

    /** An emoji value made of tones, hair styles, emoji, NUL and surrogate halves. */
    fun toneValue(random: Random): String = List(random.nextInt(6)) { TONE_PARTS.random(random) }.joinToString("")

    private val LANGUAGES = listOf("en", "EN", "de", "De", "pt", "fr", "it", "es", "root", "İt", "ıt", "ß", "ǅx", "", " ", "x y", "ﬀ")
    private val COUNTRIES = listOf("", "", " ", "\t", " ", "\u0085", "US", "us", "GB", "BR", "419")
    private val VARIANTS = listOf("", "", " ", "POSIX", "x", " ", "\u0085")
    private val FILES = listOf("de.txt", "en.txt", "es.txt", "fr.txt", "it.txt", "pt.txt", "root.txt", "en_US.txt", "en_US_POSIX.txt", "pt_BR.txt")

    /** A locale's language, country and variant. Σ is left out: see [EmojiComparison]'s users. */
    fun locale(random: Random) = Triple(LANGUAGES.random(random), COUNTRIES.random(random), VARIANTS.random(random))

    /** Some of the usual files, and usually some of the names the locale looks for, or near misses. */
    fun assets(random: Random, language: String, country: String, variant: String): List<String> {
        val lower = language.lowercase()
        val names = FILES.filter { random.nextInt(3) == 0 }.toMutableList()
        val candidates = listOf(
            "${lower}_${country}_$variant.txt", "${lower}_$country.txt", "$lower.txt",
            "${language}_$country.txt", "${lower}__$variant.txt", "$lower.TXT", "$lower.txt ",
        )
        for (candidate in candidates) if (random.nextInt(3) == 0) names += candidate
        return names.shuffled(random)
    }
}
