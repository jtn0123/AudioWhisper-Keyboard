package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.text.Normalizer
import java.util.Locale
import kotlin.random.Random

/**
 * Asks the suggest package's rules in Rust (lib/rust/crates/suggest, behind [WordCharacters],
 * [WordCase], [PreviousWords], [AutoCorrection], [AutoCorrectionUndo], [TapOffset], [TapTrail] and
 * [QwertyGeometry]) the questions the Kotlin they replaced was asked, and puts every answer on
 * [tape], whose golden files hold that Kotlin's answers (docs/phase6-kotlin-references.md).
 *
 * [sigmaExact] says whether to tell how Σ lower-cases. Rust follows ICU's rule for whether a Σ
 * ends a word (ς) or not (σ), as Android does; the JVM the goldens came from used its own, so off a
 * phone ς and σ are counted as one.
 */
class SuggestComparison(private val sigmaExact: Boolean, private val tape: GoldenTape) {
    private fun check(what: String, answer: Any?) = tape.answer(what, answer)

    /** What [body] gives, or the exception it throws, so a throw is an answer too. */
    private inline fun outcome(body: () -> Any?): Any? = try {
        body()
    } catch (e: IllegalArgumentException) {
        "threw ${e.javaClass.name}: ${e.message}"
    }

    private fun sigma(text: String) = if (sigmaExact) text else text.replace(FINAL_SIGMA, SMALL_SIGMA)

    /** Everything about [text] as a typed word. */
    fun word(text: String) {
        check("of", WordCase.of(text).name)
        check("isMostlyCaps", WordCase.isMostlyCaps(text))
        check("decoderInput", sigma(WordCase.decoderInput(text)))
        for (case in WordCase.entries) check("apply ${case.name}", case.apply(text))
    }

    /** The context before a word typed after [text]. */
    fun context(text: String) {
        val rust = PreviousWords.from(text)
        check("PreviousWords.from", "${rust.words.joinToString("|")} ${rust.isBeginningOfSentence} ${rust.slotCount}")
    }

    /** The score for correcting [before] to [after], and whether that passes [threshold]. */
    fun score(before: String, after: String, score: Int, kindAndFlags: Int, threshold: Float) {
        check("normalizedScore", AutoCorrection.normalizedScore(before, after, score))
        val suggestion = RawSuggestion(after, score, kindAndFlags)
        check("suggestionExceedsThreshold", AutoCorrection.suggestionExceedsThreshold(suggestion, before, threshold))
    }

    fun distance(before: IntArray, after: IntArray) = check("editDistance", outcome { AutoCorrection.editDistance(before, after) })

    /** Undoing the correction of [typed] to [corrected] at [start]. */
    fun revert(typed: String, corrected: String, start: Int, cursor: Int, text: String) =
        check("revertAt", AutoCorrectionUndo(typed, corrected, start).revertAt(cursor, text))

    fun tapOffset(x: Float, y: Float, centerX: Float, centerY: Float, width: Float, height: Float) =
        check("TapOffset.of", TapOffset.of(x, y, centerX, centerY, width, height))

    fun touchPoint(codePoint: Int, offset: TapOffset?) = check("touchPoint", outcome { QwertyGeometry.touchPoint(codePoint, offset) })

    /** The layout, its grid and the rules' numbers, which don't depend on any input. */
    fun constants() {
        check("keys", QwertyGeometry.keys.map { "${it.codePoint},${it.x},${it.y}" })
        check("proximityGrid", QwertyGeometry.proximityGrid().toList())
        check("sizes", listOf(QwertyGeometry.KEY_WIDTH, QwertyGeometry.KEY_HEIGHT, QwertyGeometry.KEYBOARD_WIDTH, QwertyGeometry.KEYBOARD_HEIGHT))
        check("grid", listOf(QwertyGeometry.GRID_WIDTH, QwertyGeometry.GRID_HEIGHT, QwertyGeometry.MAX_PROXIMITY_CHARS))
        check("TapOffset.LIMIT", TapOffset.LIMIT)
        check("MODEST_THRESHOLD", AutoCorrection.MODEST_THRESHOLD)
        check("MAX_COUNT", PreviousWords.MAX_COUNT)
    }

    /** Every code point, and values that are none, through each rule that takes one. */
    fun everyCodePoint() {
        val offset = TapOffset(0.3f, -0.45f)
        for (codePoint in (-1..Character.MAX_CODE_POINT + 1) + EDGE_INTS) {
            check("continuesWord", WordCharacters.continuesWord(codePoint))
            check("keyCenter", QwertyGeometry.keyCenter(codePoint))
            touchPoint(codePoint, null)
            touchPoint(codePoint, offset)
            val base = if (codePoint < 0) 'a'.code else baseLowerCase(codePoint)
            distance(intArrayOf(codePoint), intArrayOf(base))
            distance(intArrayOf(codePoint, 'e'.code), intArrayOf('e'.code, codePoint))
        }
    }

    /**
     * The letter to ask [codePoint]'s edit distance to: its accent stripped (below U+0500, as
     * LatinIME's `CharUtils::toBaseLowerCase` does) and lower-cased, by the JDK's NFD and lower
     * case. Only a question: the distance is the answer.
     */
    private fun baseLowerCase(codePoint: Int): Int {
        val base = if (codePoint < 0x500) Normalizer.normalize(String(Character.toChars(codePoint)), Normalizer.Form.NFD).codePointAt(0) else codePoint
        return Character.toLowerCase(base)
    }

    /** Every UTF-16 unit in words, contexts, separators and scores where it matters. */
    fun everyUnit() {
        for (code in 0..0xFFFF) {
            val c = code.toChar().toString()
            for (word in listOf(c, "${c}x", "x$c", "A$c", "${c}A", "AB$c", "$c$c", "I$c", "${c}é", "Σ$c")) word(word)
            for (text in unitContexts(c)) context(text)
            revert("teh", "the", 2, 6, "x the$c")
            revert("teh", "the", 2, 7, "x the$c ")
            revert("teh", "the", 2, 7, "x the $c")
            revert("teh", "the", 0, 5, "the$c$c")
            score("t${c}e", "the", 500_000, RawSuggestion.FLAG_APPROPRIATE_FOR_AUTO_CORRECTION, MODEST)
            score("the", "th$c", 900_000, 0, MODEST)
        }
    }

    private fun unitContexts(c: String) = listOf(
        c, "a$c", "a $c", "a${c}b c ", "a b$c", "hi${c}there ", "${c}x y z ", "a.${c}b ", "x\n$c y ", "a b ${c}c",
        "one two$c ", "one ${c}two ", "a$c\nb ",
    )

    /** Random words, contexts, corrections and undos built from [PIECES]. */
    fun randomStrings(random: Random, count: Int) {
        repeat(count) {
            val text = random.text(8)
            word(text)
            context(text)
            val typed = random.text(3)
            score(typed, random.text(3), SCORES.random(random), random.kindAndFlags(), THRESHOLDS.random(random))
            distance(random.codePoints(), random.codePoints())
            randomRevert(random, typed)
        }
    }

    private fun randomRevert(random: Random, typed: String) {
        val corrected = random.text(2)
        val start = when (random.nextInt(4)) {
            0 -> Int.MAX_VALUE - random.nextInt(4)
            1 -> Int.MIN_VALUE + random.nextInt(4)
            else -> random.nextInt(0, 20)
        }
        val cursor = start + corrected.length + random.nextInt(-1, 4)
        val text = when (random.nextInt(4)) {
            0 -> random.text(4)
            else -> random.text(2) + corrected + List(random.nextInt(3)) { SEPARATORS.random(random) }.joinToString("")
        }
        revert(typed, corrected, start, cursor, text)
    }

    /** Taps on keys of every size, and touch points of every offset. */
    fun taps(random: Random, count: Int) {
        repeat(count) {
            tapOffset(random.coordinate(), random.coordinate(), random.coordinate(), random.coordinate(), random.size(), random.size())
            val offset = if (random.nextInt(5) == 0) null else TapOffset(random.coordinate(), random.coordinate())
            touchPoint(TAP_CODE_POINTS.random(random), offset)
        }
    }

    /** Random presses, typing, deletes and clears on a trail, asking it for offsets after each. */
    fun trails(random: Random, count: Int) {
        repeat(count) {
            val capacity = if (random.nextBoolean()) random.nextInt(0, 6) else null
            val rust = capacity?.let { TapTrail(it) } ?: TapTrail()
            val typed = StringBuilder()
            repeat(random.nextInt(1, 100)) {
                trailStep(random, rust, typed)
                check("offsetsFor", rust.offsetsFor(query(random, typed)))
            }
        }
    }

    private fun trailStep(random: Random, rust: TapTrail, typed: StringBuilder) {
        when (random.nextInt(12)) {
            0, 1 -> {
                rust.positions.deleted()
                if (typed.isNotEmpty()) typed.setLength(typed.offsetByCodePoints(typed.length, -1))
            }
            2 -> {
                rust.positions.clear()
                typed.setLength(0)
            }
            else -> {
                val codePoint = TAP_CODE_POINTS.random(random)
                val offset = if (random.nextInt(4) == 0) null else TapOffset(random.coordinate(), random.coordinate())
                rust.pressed(codePoint, offset)
                if (random.nextInt(6) != 0) {
                    val text = List(random.nextInt(1, 3)) { TAP_TEXT.random(random) }.joinToString("")
                    rust.positions.typed(text)
                    typed.append(text)
                }
            }
        }
    }

    /** A word to ask a trail about: usually the end of what was typed, in some case. */
    private fun query(random: Random, typed: StringBuilder): String {
        if (random.nextInt(5) == 0 || typed.isEmpty()) return random.text(2)
        val count = typed.codePointCount(0, typed.length)
        val start = typed.offsetByCodePoints(0, random.nextInt(count + 1))
        val end = typed.substring(start)
        return when (random.nextInt(4)) {
            0 -> end.uppercase(Locale.US)
            1 -> end.lowercase(Locale.US)
            else -> end
        }
    }

    private fun Random.text(maxPieces: Int) = List(nextInt(maxPieces + 1)) { PIECES.random(this) }.joinToString("")

    private fun Random.codePoints() = IntArray(nextInt(6)) {
        when (nextInt(10)) {
            0 -> EDGE_INTS.random(this)
            1 -> nextInt(0, Character.MAX_CODE_POINT + 1)
            else -> PIECES.random(this).codePointAt(0)
        }
    }

    private fun Random.kindAndFlags(): Int {
        val kind = if (nextInt(5) == 0) nextInt() and RawSuggestion.KIND_MASK else KINDS.random(this)
        val flags = if (nextInt(5) == 0) nextInt() and RawSuggestion.KIND_MASK.inv() else FLAGS.random(this)
        return kind or flags
    }

    private fun Random.coordinate() = if (nextBoolean()) FLOATS.random(this) else nextFloat() * 4f - 2f

    private fun Random.size() = if (nextBoolean()) SIZES.random(this) else nextFloat() * 200f

    companion object {
        private const val SMALL_SIGMA = "σ"
        private const val FINAL_SIGMA = "ς"
        private const val MODEST = 0.185f

        private val EDGE_INTS = listOf(Int.MIN_VALUE, Int.MIN_VALUE + 1, -2, Int.MAX_VALUE, 0x110000, 0xFFFFFF)

        /**
         * Pieces of words and the text around them: letters whose case is hard, digits of other
         * scripts, the punctuation the rules look for, whitespace the JVM and Android disagree on,
         * supplementary characters and lone surrogates.
         */
        private val PIECES = listOf(
            "a", "e", "t", "h", "x", "A", "E", "T", "H", "I", "i", "K", "k", "s", "S", "é", "É", "ß", "ẞ", "ǅ",
            "Σ", "σ", "ς", "ı", "İ", "K", "ª", "ʰ", "ﬁ", "Ⓐ", "0", "7", "٣", "²", "Ⅻ", "'", "’", "-", "@", ".", "?",
            "!", ",", ";", "(", ")", "\"", " ", "  ", "\t", "\n", "\r", " ", " ", " ", "　", "\u0085", "᠎",
            "\u001C", "​", "the", "The", "THE", "teh", "don't", "I'm", "iPhone", "NASA", "e-mail", "mp3",
            "😀", "𐐀", "𐐨", "𝟎", "\uD800", "\uDC00", "́",
        )

        private val SEPARATORS = listOf(" ", " ", ".", ",", "!", "'", "a", "1", " ", "\n", "😀", "\uDC00")

        private val SCORES = listOf(0, -1, 1, 185_000, 500_000, 1_000_000, 2_000_000, Int.MAX_VALUE, Int.MIN_VALUE)

        private val THRESHOLDS = listOf(MODEST, 0f, -0f, 0.5f, 1f, Float.NaN, Float.NEGATIVE_INFINITY, Float.POSITIVE_INFINITY)

        private val KINDS = listOf(0, RawSuggestion.KIND_CORRECTION, RawSuggestion.KIND_WHITELIST, RawSuggestion.KIND_SHORTCUT, RawSuggestion.KIND_PREDICTION)

        private val FLAGS = listOf(
            0,
            RawSuggestion.FLAG_APPROPRIATE_FOR_AUTO_CORRECTION,
            RawSuggestion.FLAG_EXACT_MATCH,
            RawSuggestion.FLAG_APPROPRIATE_FOR_AUTO_CORRECTION or RawSuggestion.FLAG_EXACT_MATCH,
        )

        private val FLOATS = listOf(
            0f, -0f, 0.5f, -0.5f, 0.6f, -0.6f, 0.61f, 0.6000001f, 1f, 100f, 1e30f, -1e30f, Float.MIN_VALUE,
            Float.MAX_VALUE, Float.NaN, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, 0.005f, 0.995f, -0.005f,
        )

        private val SIZES = listOf(0f, -0f, -1f, 1e-30f, 1f, 100f, Float.MIN_VALUE, Float.NaN, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY)

        /** Keys a press can be for: letters in both cases, ones whose lower case is another key's. */
        private val TAP_CODE_POINTS = listOf(
            'a'.code, 'A'.code, 't'.code, 'T'.code, 'h'.code, 'e'.code, 'E'.code, 'k'.code, 0x212A, 'i'.code, 'I'.code,
            0x130, 0x131, 'Σ'.code, 'σ'.code, 'ς'.code, 0x10400, 0x10428, '0'.code, '\''.code, -1, -5, 0, 0x110000,
            Int.MIN_VALUE,
        )

        private val TAP_TEXT = listOf(
            "a", "A", "t", "T", "h", "e", "E", "k", "K", "K", "i", "I", "İ", "ı", "Σ", "σ", "ς", "𐐀", "𐐨", "0", "'",
            " ", "the", "😀", "\uDC00", "\u0000",
        )
    }
}
