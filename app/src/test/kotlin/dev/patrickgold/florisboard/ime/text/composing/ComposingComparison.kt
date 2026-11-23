package dev.patrickgold.florisboard.ime.text.composing

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.random.Random

/**
 * Compares the retained Telex and generic rule composers through JNI. The reference's answers also
 * go on [tape] (docs/phase6-kotlin-references.md).
 */
class ComposingComparison(private val sigmaExact: Boolean, composersJson: String, private val tape: GoldenTape) {
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
        tape.answer(what, kotlin)
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
        see("$name: ${label(toInsert, expected)}")
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
        2 -> "\u0000"
        3 -> "é"
        4 -> vietnamese.random(this)
        else -> ODD.random(this)
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
    private fun label(toInsert: String, action: Pair<Int, String>): String {
        val (delete, text) = action
        return when {
            delete == 0 && text == toInsert -> "keep"
            text.isNotEmpty() && text.uppercase() == text && text.lowercase() != text -> "delete ${delete.coerceAtMost(2)}, upper"
            else -> "delete ${delete.coerceAtMost(2)}"
        }
    }

    private fun hex(text: String) = text.map { "%04x".format(it.code) }.joinToString(" ")

    private fun show(value: Any?): String =
        if (value is Pair<*, *>) "(${value.first}, ${hex(value.second.toString())})" else value.toString()

    private companion object {
        const val MAX_DIFFERENCES = 20
        const val MAX_TEXT = 12
        const val CAPITAL_SIGMA = 'Σ'
        val JSON = Json { ignoreUnknownKeys = true }

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
