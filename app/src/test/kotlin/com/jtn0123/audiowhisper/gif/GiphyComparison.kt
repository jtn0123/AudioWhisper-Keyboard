package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GiphyRules
import com.jtn0123.audiowhisper.rust.parse
import java.math.BigDecimal
import java.net.URLEncoder
import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random

/**
 * Holds the reading of GIPHY's answers and the building of its URLs, in Rust
 * (lib/rust/crates/gif/src/giphy.rs, behind [GiphyRules]; the panel's session runs the same
 * functions), up against the Kotlin they replaced,
 * [ReferenceGiphy]: random GIPHY-like answers, broken in the ways kotlinx.serialization is
 * particular about (commas, whitespace, escapes, nulls, unknown values, repeats, truncation);
 * random sizes for `toFloatOrNull` (decimals, hexadecimals, names, halfway cases, far exponents,
 * suffixes, padding); and URLs for odd searches and keys. Self-contained, so a device check can
 * copy it into the app and run it on a phone, whose Java parses numbers and encodes URLs itself.
 */
class GiphyComparison(private val tape: GoldenTape) {
    /** How many comparisons failed, and the first few, described. */
    var failures = 0
        private set
    val examples = mutableListOf<String>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    /** How often each outcome worth reaching was reached, so a run that tests nothing shows. */
    val seen = sortedMapOf<String, Int>()

    private fun check(what: String, input: Any?, kotlin: Any?, rust: Any?) {
        tape.record("$what: ${show(kotlin)}")
        checks++
        if (kotlin == rust) return
        failures++
        if (examples.size < MAX_EXAMPLES) examples += "$what(${show(input)}): Kotlin ${show(kotlin)}, Rust ${show(rust)}"
    }

    private fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    /** What reading [body] gives: the GIFs, their ratios by their bits, or the failure. */
    private fun outcome(body: String, parse: (String) -> List<Gif>): Any = try {
        parse(body).map { listOf(it.id, it.title, it.previewUrl, it.aspectRatio.toRawBits(), it.sendUrl, it.pageUrl) }
    } catch (e: GiphyFormatException) {
        "unreadable"
    } catch (e: RuntimeException) {
        "threw ${e.javaClass.name}"
    }

    /** Main's Giphy.parse: the Rust's reading, a body it can't read being unreadable. */
    private fun parse(body: String): List<Gif> = try {
        GiphyRules.parse(body, ::Gif)
    } catch (e: IllegalArgumentException) {
        throw GiphyFormatException(e)
    }

    private fun compare(what: String, body: String): Any {
        val old = outcome(body, ReferenceGiphy::parse)
        check(what, body, old, outcome(body, ::parse))
        return old
    }

    /** Random answers, some of them broken. */
    fun bodies(random: Random, count: Int) = repeat(count) {
        val valid = body(random)
        val broken = random.nextInt(3) == 0
        val body = if (broken) mutate(random, valid) else valid
        val old = compare("parse", body)
        see(if (old == "unreadable") if (broken) "broken: unreadable" else "unreadable" else "readable")
        see("broken but readable", broken && old != "unreadable")
        val gifs = (old as? List<*>)?.size ?: 0
        see(if (gifs == 0) "no gifs" else if (gifs == 1) "one gif" else "several gifs", old != "unreadable")
    }

    /** Random sizes, read alone: as a width over a height of 1, and as a height under a width of 1. */
    fun sizes(random: Random, count: Int) = repeat(count) {
        val size = size(random)
        val item = { width: String, height: String ->
            """{"data":[{"id":"a","images":{"fixed_width":{"url":"u","width":${quote(width)},"height":${quote(height)}}}}]}"""
        }
        val asWidth = compare("width", item(size, "1"))
        compare("height", item("1", size))
        val value = size.toFloatOrNull()
        see(
            when {
                value == null -> "not a number"
                value.isNaN() -> "NaN"
                value.isInfinite() -> "infinite"
                value == 0f -> "zero"
                kotlin.math.abs(value) < java.lang.Float.MIN_NORMAL -> "subnormal"
                else -> "a number"
            },
        )
        see("hexadecimal", value != null && "x" in size.lowercase())
        see("a width read", asWidth is List<*> && asWidth.isNotEmpty())
    }

    /** URLs for random searches, keys and params. */
    fun urls(random: Random, count: Int) = repeat(count) {
        val key = text(random, 3)
        val query = text(random, if (random.nextBoolean()) 4 else 40)
        val cut = query.take(ReferenceGiphy.MAX_QUERY_LENGTH)
        checkUrl("searchUrl", listOf(key, query), listOf(key, cut), GiphyRules.searchUrl(key, query)) {
            ReferenceGiphy.searchUrl(it[0], it[1])
        }
        checkUrl("trendingUrl", key, listOf(key), GiphyRules.trendingUrl(key)) { ReferenceGiphy.trendingUrl(it[0]) }
        val params = linkedMapOf<String, String>()
        repeat(random.nextInt(5)) { params[NAMES.random(random)] = text(random, 3) }
        val endpoint = listOf("search", "trending", "random", text(random, 1)).random(random)
        val names = params.keys.toList()
        checkUrl("url", listOf(endpoint, params), listOf(key) + params.values, GiphyRules.url(key, endpoint, params)) {
            ReferenceGiphy.url(it[0], endpoint, names.zip(it.drop(1)).toMap(LinkedHashMap()))
        }
        see("a search cut", query.length > ReferenceGiphy.MAX_QUERY_LENGTH)
        see("half an emoji", query.any { it.isSurrogate() })
        see("a setting in the params", params.keys.any { it in SETTINGS })
    }

    /**
     * Checks a URL the Rust built from [texts] against the one [build] makes of them. JDK 22 and
     * later's URLEncoder loses a lone high surrogate that ends one of its 8-unit chunks, where
     * JDK 21 and Android, like the Rust, write `?`; on a JVM that does, the URL expected for text
     * with lone surrogates is the one for that text with them already `?`, which every encoder
     * agrees on. Android has no such quirk, so the device check holds the Rust to its own encoder.
     */
    private fun checkUrl(what: String, input: Any?, texts: List<String>, rust: String, build: (List<String>) -> String) {
        val kotlin = build(texts)
        if (kotlin == rust || !CHUNK_QUIRK || texts.none { it.any(Char::isSurrogate) }) return check(what, input, kotlin, rust)
        see("the JDK's chunk quirk")
        check(what, input, build(texts.map(::questionLoneSurrogates)), rust)
    }

    /** [text] with each surrogate that isn't half of a pair replaced by `?`. */
    private fun questionLoneSurrogates(text: String): String = buildString {
        var i = 0
        while (i < text.length) {
            val pair = text[i].isHighSurrogate() && i + 1 < text.length && text[i + 1].isLowSurrogate()
            if (pair) append(text, i, i + 2) else append(if (text[i].isSurrogate()) '?' else text[i])
            i += if (pair) 2 else 1
        }
    }

    // ---- Answers ----

    private fun body(random: Random): String {
        val members = mutableListOf<String>()
        if (random.nextInt(10) != 0) members += member(random, "data", list(random) { item(random) })
        if (random.nextInt(20) == 0) members += member(random, "data", list(random) { item(random) })
        repeat(random.nextInt(3)) { members += member(random, UNKNOWN.random(random), junk(random, 3)) }
        members.shuffle(random)
        return ws(random) + obj(random, members) + ws(random)
    }

    private fun item(random: Random): String = when (random.nextInt(150)) {
        0 -> junk(random, 2)
        1 -> "null"
        else -> {
            val members = mutableListOf<String>()
            if (random.nextInt(8) != 0) members += member(random, "id", stringOr(random, IDS.random(random)))
            if (random.nextBoolean()) members += member(random, "title", stringOr(random, text(random, 3)))
            if (random.nextInt(4) != 0) members += member(random, "url", stringOr(random, "https://giphy.com/gifs/x"))
            if (random.nextInt(10) != 0) members += member(random, "images", images(random))
            repeat(random.nextInt(3)) { members += member(random, UNKNOWN.random(random), junk(random, 2)) }
            if (random.nextInt(10) == 0) members += member(random, "id", stringOr(random, IDS.random(random)))
            members.shuffle(random)
            obj(random, members)
        }
    }

    private fun images(random: Random): String {
        if (random.nextInt(150) == 0) return junk(random, 1)
        val members = mutableListOf<String>()
        if (random.nextInt(6) != 0) members += member(random, "fixed_width", rendition(random))
        if (random.nextBoolean()) members += member(random, "downsized", rendition(random))
        repeat(random.nextInt(3)) { members += member(random, UNKNOWN.random(random), junk(random, 2)) }
        members.shuffle(random)
        return obj(random, members)
    }

    private fun rendition(random: Random): String {
        if (random.nextInt(8) == 0) return "null"
        if (random.nextInt(200) == 0) return junk(random, 1)
        val members = mutableListOf<String>()
        if (random.nextInt(4) != 0) members += member(random, "url", nullableString(random, "https://m/${random.nextInt(9)}.gif"))
        if (random.nextBoolean()) members += member(random, "webp", nullableString(random, "https://m/${random.nextInt(9)}.webp"))
        if (random.nextInt(6) != 0) members += member(random, "width", nullableString(random, plausibleSize(random)))
        if (random.nextInt(6) != 0) members += member(random, "height", nullableString(random, plausibleSize(random)))
        repeat(random.nextInt(3)) { members += member(random, UNKNOWN.random(random), junk(random, 1)) }
        members.shuffle(random)
        return obj(random, members)
    }

    private fun member(random: Random, key: String, value: String) =
        ws(random) + quote(key) + ws(random) + ":" + ws(random) + value + ws(random)

    /** An object's members, mostly with commas between them, which kotlinx doesn't need. */
    private fun obj(random: Random, members: List<String>): String = buildString {
        append('{')
        if (random.nextInt(300) == 0) append(',')
        members.forEachIndexed { i, it ->
            if (i > 0) append(if (random.nextInt(8) == 0) "" else ",")
            append(it)
        }
        append('}')
    }

    private fun list(random: Random, item: () -> String): String =
        List(random.nextInt(6)) { ws(random) + item() + ws(random) }.joinToString(if (random.nextInt(200) == 0) "" else ",", if (random.nextInt(300) == 0) "[," else "[", "]")

    /** A value of the wrong type sometimes, else [value] as a string. */
    private fun stringOr(random: Random, value: String): String = when (random.nextInt(400)) {
        0 -> "null"
        1 -> "${random.nextInt(100)}"
        2 -> "true"
        else -> escaped(random, value)
    }

    private fun nullableString(random: Random, value: String) = when (random.nextInt(200)) {
        in 0..9 -> "null"
        10 -> listOf("nullx", "nul", "null1", "NULL", "null\"\"").random(random)
        else -> stringOr(random, value)
    }

    /**
     * [text] quoted with some of its units escaped, as `\uXXXX` in either case or the short
     * escapes, now and then a raw control character, and rarely an escape kotlinx refuses.
     */
    private fun escaped(random: Random, text: String): String {
        if (random.nextInt(3) != 0) return quote(text)
        return buildString {
            append('"')
            for (unit in text) {
                when {
                    unit == '"' || unit == '\\' -> append('\\').append(unit)
                    random.nextInt(5) != 0 -> append(unit)
                    random.nextBoolean() -> append("\\u").append("%04x".format(unit.code).let { if (random.nextBoolean()) it.uppercase() else it })
                    else -> append(SHORT_ESCAPES[unit] ?: "\\u%04X".format(unit.code))
                }
            }
            when (random.nextInt(40)) {
                0 -> append(SHORT_ESCAPES.values.random(random))
                1 -> append(listOf("\u0001", "\u001F", "\t", "\n").random(random))
                2 -> append(listOf("\\x", "\\u12g4", "\\uG000", "\\u00e", "\\U0041", "\\'").random(random))
            }
            append('"')
        }
    }

    /** An unknown key's value: any JSON nested up to [depth], and now and then something kotlinx skips or trips on. */
    private fun junk(random: Random, depth: Int): String = when (if (depth <= 0) random.nextInt(4) else random.nextInt(7)) {
        0 -> quote(text(random, 3))
        1 -> listOf("null", "true", "false", "-1.5e3", "0", "{}", "[]", "[{}]", "{\"a\":[]}", "\":\"", "\"]\"").random(random)
        2 -> "\"${listOf("\\\"", "\\\\", "\\/", "\\b", "\\u00E9", "\\uD83D\\uDE00", "\\ud800", "\u0001").random(random)}\""
        3 -> if (random.nextInt(30) != 0) "\"x\"" else ODD.random(random)
        4 -> list(random) { junk(random, depth - 1) }
        5 -> obj(random, List(random.nextInt(3)) { member(random, UNKNOWN.random(random), junk(random, depth - 1)) })
        else -> if (random.nextInt(8) != 0) "[1, 2]" else "[" + List(random.nextInt(4)) { listOf("1", ":", ",", "{", "}", "[", "x", "\"s\"").random(random) }.joinToString("") + "]"
    }

    /** [valid] broken: a unit dropped, punctuation added, a piece repeated or the end cut. */
    private fun mutate(random: Random, valid: String): String {
        var body = valid
        repeat(1 + random.nextInt(3)) {
            if (body.isEmpty()) return body
            val at = random.nextInt(body.length + 1)
            body = when (random.nextInt(5)) {
                0 -> body.removeRange(at.coerceAtMost(body.length - 1), (at + 1).coerceAtMost(body.length))
                1 -> body.substring(0, at) + listOf(",", ":", "{", "}", "[", "]", "\"", "\\", " ", "\u000B", "\u00A0", "null").random(random) + body.substring(at)
                2 -> body.substring(0, at) + body.substring(random.nextInt(body.length + 1).let { minOf(it, at) }, at) + body.substring(at)
                3 -> body.substring(0, at)
                else -> body.substring(0, at) + ws(random) + body.substring(at)
            }
        }
        return body
    }

    /** Mostly nothing or JSON's whitespace; now and then a unit kotlinx doesn't take as whitespace. */
    private fun ws(random: Random): String = when {
        random.nextInt(4) != 0 -> ""
        random.nextInt(60) == 0 -> NOT_WHITESPACE.random(random)
        else -> WHITESPACE.random(random)
    }

    private fun quote(text: String) = "\"" + text.replace("\\", "\\\\").replace("\"", "\\\"") + "\""

    // ---- Sizes ----

    /** A size as GIPHY sends them half the time, else any. */
    private fun plausibleSize(random: Random): String = if (random.nextBoolean()) "${random.nextInt(-5, 500)}" else size(random)

    private fun size(random: Random): String {
        val number = when (random.nextInt(11)) {
            10 -> hexadecimalTie(random)
            0 -> NUMBERS.random(random)
            1, 2 -> halfway(random)
            3 -> hexadecimal(random)
            4 -> Float.fromBits(random.nextInt()).toString()
            5 -> java.lang.Float.toHexString(Float.fromBits(random.nextInt()))
            else -> decimal(random)
        }
        val padded = pad(random) + listOf("", "", "", "+", "-").random(random) + number + suffix(random) + pad(random)
        return if (random.nextInt(30) == 0) padded.substring(0, random.nextInt(padded.length + 1)) else padded
    }

    private fun digits(random: Random, max: Int, from: String = "0123456789") =
        String(CharArray(random.nextInt(max + 1)) { from.random(random) })

    private fun decimal(random: Random): String {
        val whole = if (random.nextInt(5) == 0) "0".repeat(random.nextInt(30)) + digits(random, 3) else digits(random, 12)
        val fraction = if (random.nextBoolean()) "." + digits(random, if (random.nextInt(10) == 0) 60 else 8) else ""
        return whole + fraction + exponent(random, "eE")
    }

    private fun hexadecimal(random: Random): String {
        val hex = "0123456789abcdefABCDEF"
        val whole = digits(random, if (random.nextInt(5) == 0) 30 else 8, hex)
        val fraction = if (random.nextBoolean()) "." + digits(random, 20, hex) else ""
        val exponent = if (random.nextInt(15) == 0) "" else exponent(random, "pP").ifEmpty { "p0" }
        return listOf("0x", "0X").random(random) + whole + fraction + exponent
    }

    /**
     * A hexadecimal exactly halfway between two floats, written with at least 16 digits, and
     * sometimes a digit beyond them that tips it: the case where digits past a long's worth decide.
     */
    private fun hexadecimalTie(random: Random): String {
        val bits = random.nextInt(0, 0x7F80_0000)
        val field = bits ushr 23
        val significand = (bits and 0x7F_FFFF) or (if (field == 0) 0 else 0x80_0000)
        val power = (if (field == 0) -149 else field - 150) - 1
        val digits = java.lang.Long.toHexString(2L * significand + 1)
        val zeros = 16 - digits.length + random.nextInt(4)
        val tail = listOf("", "0", "1", "00001", "8").random(random)
        val shift = 4 * (zeros + tail.length)
        return "0x" + digits + "0".repeat(zeros) + tail + "p" + (power - shift)
    }

    private fun exponent(random: Random, letters: String): String {
        if (random.nextInt(3) == 0) return ""
        val sign = listOf("", "+", "-").random(random)
        val digits = when (random.nextInt(8)) {
            0 -> digits(random, 25)
            1 -> ""
            else -> "${random.nextInt(-160, 160).let { if (it < 0) -it else it }}"
        }
        return letters.random(random) + sign + digits
    }

    /** A float's exact halfway point to its neighbour, or just either side of it. */
    private fun halfway(random: Random): String {
        val float = Float.fromBits(random.nextInt(0, 0x7F80_0000))
        val half = BigDecimal(Math.ulp(float).toDouble()).divide(BigDecimal(2))
        val nudge = BigDecimal.ONE.movePointLeft(70)
        val value = BigDecimal(float.toDouble()) + half + listOf(BigDecimal.ZERO, nudge, nudge.negate()).random(random)
        return if (random.nextBoolean()) value.toPlainString() else value.toString()
    }

    private fun suffix(random: Random) = if (random.nextInt(6) != 0) "" else listOf("f", "F", "d", "D", "x", "ff", "L").random(random)

    private fun pad(random: Random) = if (random.nextInt(8) != 0) "" else listOf(" ", "\t", "\n", "\u0000", "\u001F", "\u00A0", "\u2007", "  ").random(random)

    // ---- Text ----

    private fun text(random: Random, parts: Int): String = buildString {
        repeat(random.nextInt(parts + 1)) { append(PARTS.random(random)) }
    }

    private fun show(value: Any?): String = when (value) {
        is String -> "\"" + value.map { if (it.code in 0x20..0x7E) "$it" else "\\u%04X".format(it.code) }.joinToString("") + "\""
        is List<*> -> value.joinToString(", ", "[", "]") { show(it) }
        is Map<*, *> -> value.entries.joinToString(", ", "{", "}") { "${show(it.key)}=${show(it.value)}" }
        else -> value.toString()
    }

    private companion object {
        const val MAX_EXAMPLES = 10

        /** Whether this JVM's URLEncoder loses a lone high surrogate at the end of a chunk (see [checkUrl]). */
        val CHUNK_QUIRK = !URLEncoder.encode("\u00E9".repeat(6) + "\uD800", "UTF-8").endsWith("%3F")

        val WHITESPACE = listOf(" ", "\t", "\n", "\r", "  \n", "\r\n\t")
        val NOT_WHITESPACE = listOf("\u000B", "\u000C", "\u00A0", "\u0000", "\u2028", "\uFEFF")
        val IDS = listOf("a", "b", "c", "abc", "A", "", " ", "\t", "\u00A0", "\u2007", "\u3000")
        val UNKNOWN = listOf("meta", "pagination", "rating", "user", "type", "size", "mp4", "id ", "Data", "")
        /** Values kotlinx skips, or trips on, where JSON would have a value. */
        val ODD = listOf("nul", "nullx", "tru-e", "tr\\ue", "x\\", "1,2", "\"\\x\"", "\"\\u12\"", "", "]", ":")
        val SHORT_ESCAPES = mapOf(
            '\b' to "\\b", '\t' to "\\t", '\n' to "\\n", '\u000C' to "\\f", '\r' to "\\r", '/' to "\\/",
        )
        val NAMES = listOf("q", "offset", "api_key", "limit", "rating", "lang", "random_id", "", "x y", "é")
        val SETTINGS = setOf("api_key", "limit", "rating", "lang")
        val NUMBERS = listOf(
            "NaN", "Infinity", "nan", "inf", "INFINITY", "0", "00", "-0", ".", "", "e5", "1e", "1e+", "1.2.3", "١٢", "２",
            "3.4028235e38", "3.40282357e38", "3.4028236e38", "1.4e-45", "7e-46", "7.1e-46", "1.17549435e-38",
            "16777217", "16777219", "1e999999999999", "1e-999999999999", "0x1p-150", "0x1.fffffep127", "0x1.ffffffp127",
            "0x", "0x.p1", "0xp1", "0x1", "0x1p", "200", "100", "480", "270.5", "1_000", "0b1", "1L",
        )
        val PARTS = listOf(
            "cat", "happy dance", "a", "Z", "0", "9", ".", "-", "*", "_", "~", "+", "&", "=", "?", "/", "%", "#", "'",
            "!", "(", ")", " ", "  ", "\t", "\n", "é", "ß", "İ", "Σ", "中文", "😀", "\uD800", "\uDC00", "\u0000",
            "\u007F", "\u0080", "\u00A0", "\u2028", "\uFEFF", "\uFFFF", "x".repeat(25), "y".repeat(49),
        )
    }
}
