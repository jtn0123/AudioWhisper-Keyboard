package dev.patrickgold.florisboard.ime.text.composing

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.random.Random

/**
 * Puts the text composers in Rust ([HangulUnicode], [KanaUnicode] and [WithRules], which call
 * lib/rust/crates/composing) up against the Kotlin they replaced ([ReferenceHangulUnicode],
 * [ReferenceKanaUnicode], [ReferenceWithRules]) wherever this runs: the JVM's tests and, copied in
 * for a check, the phone. It records each difference and how often each outcome was seen, so a
 * caller can tell the runs reached them all.
 *
 * [composersJson] is the bundled composers' extension.json, whose Telex rules are compared as the
 * app reads them. [sigmaExact] says whether text given to the rules may hold Σ, which lower-cases
 * to ς at the end of a word and to σ elsewhere. Rust decides that by ICU's rule, as Android does;
 * the JVM uses its own, so off a phone Σ is made σ. Hangul and kana judge no case, so they take Σ
 * everywhere.
 */
class ComposingComparison(private val sigmaExact: Boolean, composersJson: String) {
    val differences = mutableListOf<String>()
    val seen = mutableMapOf<String, Int>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    private val telex: WithRules
    private val telexReference: ReferenceWithRules
    private val vietnamese: List<String>

    init {
        val composers = JSON.parseToJsonElement(composersJson).jsonObject.getValue("composers").jsonArray
        val element = composers.single { it.jsonObject["id"]?.jsonPrimitive?.content == "telex" }
        telex = JSON.decodeFromJsonElement(WithRules.serializer(), element)
        telexReference = ReferenceWithRules(telex.id, telex.label, telex.rules)
        // Every letter the rules type or read, with every tone mark, in both cases.
        val letters = telex.rules.flatMap { (key, value) -> (key + value).map { it.toString() } }.toSet()
        vietnamese = (letters + letters.map { it.uppercase() }).filter { sigmaExact || CAPITAL_SIGMA !in it }.toList()
        check(element is JsonObject && telex.rules.size > 100) { "the Telex rules weren't read" }
    }

    private fun record(what: String, input: String, kotlin: Any?, rust: Any?) {
        checks++
        if (kotlin != rust && differences.size < MAX_DIFFERENCES) {
            differences += "$what(${hex(input)}): kotlin ${show(kotlin)}, rust ${show(rust)}"
        }
    }

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    /** What [body] gives, or the exception it throws, so both sides must throw alike. */
    private inline fun outcome(body: () -> Any?): Any? = try {
        body()
    } catch (e: RuntimeException) {
        e::class.java.name
    }

    private fun clean(text: String) = if (sigmaExact) text else text.replace(CAPITAL_SIGMA, 'σ')

    /** One keystroke on [kotlin] and [rust], compared and labelled; Kotlin's action. */
    private fun compare(name: String, kotlin: Composer, rust: Composer, preceding: String, toInsert: String): Pair<Int, String> {
        val expected = kotlin.getActions(preceding, toInsert)
        record(name, "$preceding | $toInsert", expected, rust.getActions(preceding, toInsert))
        see("$name: ${label(name, preceding, toInsert, expected)}")
        return expected
    }

    /** What the editor holds once [action] is applied: it deletes only a positive count. */
    private fun apply(text: String, action: Pair<Int, String>): String {
        val (delete, typed) = action
        return (if (delete > 0) text.dropLast(delete) else text) + typed
    }

    /**
     * Runs of [steps] keystrokes typed into a growing text, each read as the keyboard reads it
     * (the last [Composer.toRead] characters) or whole.
     */
    private fun typing(random: Random, name: String, kotlin: Composer, rust: Composer, steps: Int, key: Random.() -> String) {
        var text = ""
        repeat(steps) {
            val toInsert = random.key()
            val read = if (random.nextInt(4) == 0) text else text.takeLast(kotlin.toRead.coerceAtLeast(random.nextInt(2)))
            val preceding = if (name == "telex" || name == "rules") clean(read) else read
            text = apply(text, compare(name, kotlin, rust, preceding, toInsert)).takeLast(MAX_TEXT)
            if (random.nextInt(20) == 0) text = ""
        }
    }

    /** Korean typed jamo by jamo, and odd keystrokes after every kind of syllable. */
    fun hangul(random: Random, runs: Int, steps: Int) {
        val kotlin = ReferenceHangulUnicode()
        val rust = HangulUnicode()
        repeat(runs) { typing(random, "hangul", kotlin, rust, steps) { hangulKey() } }
        repeat(runs * steps) {
            val preceding = random.text(random.nextInt(3)) + random.hangulLast()
            compare("hangul", kotlin, rust, if (random.nextInt(30) == 0) "" else preceding, random.hangulKey())
        }
    }

    /** Kana typed with their marks and the small-kana key, sticky and not. */
    fun kana(random: Random, runs: Int, steps: Int) {
        val composers = listOf(
            ReferenceKanaUnicode(sticky = false) to KanaUnicode(),
            ReferenceKanaUnicode(sticky = true) to JSON.decodeFromString(KanaUnicode.serializer(), """{"sticky":true}"""),
        )
        check(composers[1].second.sticky) { "sticky wasn't read" }
        repeat(runs) {
            val (kotlin, rust) = composers.random(random)
            typing(random, "kana", kotlin, rust, steps) { kanaKey() }
        }
        repeat(runs * steps) {
            val (kotlin, rust) = composers.random(random)
            val preceding = if (random.nextInt(30) == 0) "" else random.text(random.nextInt(3)) + random.kanaLast()
            compare("kana", kotlin, rust, preceding, random.kanaKey())
        }
    }

    /** Vietnamese typed with the bundled Telex rules. */
    fun telex(random: Random, runs: Int, steps: Int) {
        repeat(runs) { typing(random, "telex", telexReference, telex, steps) { clean(telexKey()) } }
    }

    /**
     * Random rule tables: empty endings, endings that overlap or end alike, several characters,
     * and letters whose case maps to another length; an empty table, which neither can make.
     */
    fun randomRules(random: Random, tables: Int, steps: Int) = repeat(tables) {
        // A few letters each, so endings overlap and match often.
        val letters = List(random.nextInt(2, 6)) { RULE_PIECES.random(random) }
        val rules = List(random.nextInt(9)) { random.rulesText(letters, 3) to random.rulesText(letters, 3) }.toMap()
        val kotlin = outcome { ReferenceWithRules("random", "Random", rules) }
        val rust = outcome { WithRules("random", "Random", rules) }
        if (kotlin !is Composer || rust !is Composer) {
            record("rules", rules.toString(), kotlin, rust)
            see("rules: no table")
            return@repeat
        }
        record("toRead", rules.toString(), kotlin.toRead, rust.toRead)
        typing(random, "rules", kotlin, rust, steps) { if (nextInt(10) == 0) piece() else rulesText(letters, 1) }
    }

    /** Every UTF-16 unit typed after odd texts, and after each composer's own characters. */
    fun everyUnit() {
        val composers = listOf(
            "hangul" to (ReferenceHangulUnicode() to HangulUnicode()),
            "kana" to (ReferenceKanaUnicode() to KanaUnicode()),
            "telex" to (telexReference to telex),
        )
        for ((name, pair) in composers) {
            val (kotlin, rust) = pair
            val before = EVERY_UNIT_BEFORE + if (name == "telex") vietnamese.take(20) else EVERY_UNIT_AFTER
            for (unit in 0..0xFFFF) {
                val c = unit.toChar().toString()
                if (name == "telex" && !sigmaExact && unit == CAPITAL_SIGMA.code) continue
                compare(name, kotlin, rust, before[unit % before.size], c)
                compare(name, kotlin, rust, "x$c", EVERY_UNIT_AFTER[unit % EVERY_UNIT_AFTER.size])
            }
        }
    }

    private fun Random.text(length: Int) = buildString { repeat(length) { append(piece()) } }

    private fun Random.piece(): String = when (nextInt(6)) {
        0 -> nextInt(0x10000).toChar().toString()
        1 -> listOf("\uD800", "\uDC00", "\uDBFF", "😀", "𛅐").random(this)
        2 -> JAMO.random(this)
        3 -> KANA.random(this)
        4 -> vietnamese.random(this)
        else -> ODD.random(this)
    }

    /** A syllable, built from its blocks so finals and vowels that combine come up often. */
    private fun Random.syllable(): String {
        val initial = nextInt(19)
        val medial = if (nextBoolean()) listOf(8, 13, 18).random(this) else nextInt(21)
        val last = when (nextInt(5)) {
            0, 1 -> 0
            2 -> listOf(1, 4, 8, 17).random(this)
            else -> nextInt(28)
        }
        return (0xAC00 + initial * 588 + medial * 28 + last).toChar().toString()
    }

    private fun Random.hangulLast(): String = when (nextInt(10)) {
        in 0..3 -> syllable()
        in 4..6 -> JAMO.random(this)
        7 -> listOf("꯿", "가", "힣", "힤", "㄰", "㆏").random(this)
        else -> piece()
    }

    private fun Random.hangulKey(): String = when (nextInt(20)) {
        in 0..6 -> HANGUL_MEDIALS.random(this)
        in 7..13 -> HANGUL_CONSONANTS.random(this)
        14 -> JAMO.random(this)
        15 -> "_"
        16 -> ""
        17 -> JAMO.random(this) + piece()
        else -> piece()
    }

    private fun Random.kanaLast(): String = when (nextInt(8)) {
        in 0..4 -> KANA.random(this)
        5 -> KANA_MARKS.random(this)
        else -> piece()
    }

    private fun Random.kanaKey(): String = when (nextInt(12)) {
        in 0..4 -> KANA_MARKS.random(this)
        in 5..8 -> KANA.random(this)
        9 -> ""
        10 -> KANA_MARKS.random(this) + piece()
        else -> piece()
    }

    private fun Random.telexKey(): String = when (nextInt(12)) {
        in 0..4 -> TELEX_KEYS.random(this).let { if (nextInt(4) == 0) it.uppercase() else it }
        in 5..8 -> vietnamese.random(this)
        9 -> ""
        else -> piece()
    }

    /** Up to [maxLength] of [letters], rarely none. */
    private fun Random.rulesText(letters: List<String>, maxLength: Int): String {
        val length = if (nextInt(12) == 0) 0 else nextInt(1, maxLength + 1)
        return clean(buildString { repeat(length) { append(letters.random(this@rulesText)) } })
    }

    /** An outcome's name: what kind of action, for what kind of keystroke. */
    private fun label(name: String, preceding: String, toInsert: String, action: Pair<Int, String>): String = when (name) {
        "hangul" -> hangulLabel(preceding, toInsert, action)
        "kana" -> kanaLabel(toInsert, action)
        else -> {
            val (delete, text) = action
            when {
                delete == 0 && text == toInsert -> "keep"
                text.isNotEmpty() && text.uppercase() == text && text.lowercase() != text -> "delete ${delete.coerceAtMost(2)}, upper"
                else -> "delete ${delete.coerceAtMost(2)}"
            }
        }
    }

    private fun hangulLabel(preceding: String, toInsert: String, action: Pair<Int, String>): String {
        val (delete, text) = action
        val last = preceding.lastOrNull()
        fun isSyllable(c: Char?) = c != null && c in '가'..'힣'
        fun final(c: Char) = (c.code - 0xAC00) % 28
        return when {
            delete == 0 && isSyllable(last) && toInsert.startsWith("_") -> "underscore after a syllable"
            delete == 0 && text == toInsert -> "keep"
            delete != 1 -> "other"
            text.length == 2 -> if (final(text[0]) == 0) "final starts a syllable" else "composed final split"
            text.length == 1 && isSyllable(text[0]) -> when {
                !isSyllable(last) -> "initial and vowel"
                final(text[0]) == 0 -> "vowels combine in a syllable"
                final(last!!) == 0 -> "final added"
                else -> "finals combine in a syllable"
            }
            text.length == 1 && text[0] in 'ㅏ'..'ㅣ' -> "vowels combine"
            text.length == 1 -> "finals combine"
            else -> "other"
        }
    }

    private fun kanaLabel(toInsert: String, action: Pair<Int, String>): String {
        val (delete, text) = action
        val kind = when (toInsert.firstOrNull()) {
            '゙', '゛', 'ﾞ' -> "dakuten"
            '゚', '゜', 'ﾟ' -> "handakuten"
            '〓' -> "small"
            else -> "other"
        }
        val shape = when {
            delete == 0 && text == toInsert -> "keep"
            delete == 0 && text.isEmpty() -> "nothing"
            delete == 0 -> "mark alone"
            text.isEmpty() -> "marks cancel"
            text == "゙" || text == "゚" -> "second mark"
            else -> "made"
        }
        return "$kind $shape"
    }

    private fun hex(text: String) = text.map { "%04x".format(it.code) }.joinToString(" ")

    private fun show(value: Any?): String =
        if (value is Pair<*, *>) "(${value.first}, ${hex(value.second.toString())})" else value.toString()

    private companion object {
        const val MAX_DIFFERENCES = 20
        const val MAX_TEXT = 12
        const val CAPITAL_SIGMA = 'Σ'
        val JSON = Json { ignoreUnknownKeys = true }

        /** Every Hangul Compatibility Jamo, U+3131 to U+318E. */
        val JAMO = (0x3131..0x318E).map { it.toChar().toString() }
        val HANGUL_MEDIALS = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ".map { it.toString() }
        val HANGUL_CONSONANTS = "ㄱㄲㄳㄴㄵㄶㄷㄸㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅃㅄㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ".map { it.toString() }

        /** Every hiragana and katakana, their phonetic extensions, halfwidth katakana and the small kana outside the BMP. */
        val KANA = ((0x3041..0x30FF) + (0x31F0..0x31FF) + (0xFF66..0xFF9D)).map { it.toChar().toString() } +
            (0x1B150..0x1B167).map { String(Character.toChars(it)) }
        val KANA_MARKS = listOf("゙", "゚", "゛", "゜", "ﾞ", "ﾟ", "〓")

        val TELEX_KEYS = "abcdeghiklmnopqrstuvxyw".map { it.toString() } + listOf("w", "s", "f", "r", "x", "j", "z", "a", "e", "o", "d")

        val ODD = listOf("a", "A", "x", " ", "_", "'", "1", "ß", "İ", "ı", "ǅ", "ﬁ", "Σ", "σ", "̇", "\u0000", "￿")
        val RULE_PIECES = listOf(
            "a", "b", "A", "B", "ß", "ẞ", "İ", "i", "I", "ı", "̇", "ǅ", "ǆ", "Ǆ", "ﬁ", "Σ", "σ", "ς", "😀",
            "\uD800", "\uDC00", "1", " ", "é", "É",
        )

        /** Texts to type every unit after, and keystrokes to type after every unit. */
        val EVERY_UNIT_BEFORE = listOf("", "x", "가", "각", "갃", "ㄱ", "ㅗ", "か", "が", "ぱ", "っ", "゙", "\uD800", "a", "A", "â", "Â")
        val EVERY_UNIT_AFTER = listOf("ㅏ", "ㄱ", "ㅅ", "ㅣ", "_", "゙", "゚", "゛", "ﾟ", "〓", "a", "A", "s", "w", "", "xy")
    }
}
