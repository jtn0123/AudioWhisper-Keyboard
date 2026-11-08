package com.jtn0123.audiowhisper.suggest

import java.text.Normalizer
import java.util.Locale
import kotlin.math.roundToInt

/**
 * The suggest package's leaf rules as they were in Kotlin before moving to Rust
 * (lib/rust/crates/suggest), copied unchanged from commit d36ff93f: WordCharacters.kt, WordCase.kt,
 * TapTrail.kt, QwertyGeometry.kt, PreviousWords.kt, AutoCorrection.kt and AutoCorrectionUndo.kt.
 * They are what [SuggestRulesMatchTest] holds the Rust to; don't change them. [RawSuggestion]
 * stays Kotlin, so both use it.
 */
object ReferenceSuggest {
    /** Which typed characters end a word (and so may trigger auto-correction of it). */
    object WordCharacters {
        /**
         * AOSP's word connectors (`'` and `-`, plus the typographic apostrophe) and digits continue
         * the word being typed, so "don't", "e-mail" and "mp3" are not auto-corrected halfway through;
         * `@` does too, so the name part of an e-mail address is left alone.
         */
        private val CONTINUING_PUNCTUATION = setOf('\''.code, '’'.code, '-'.code, '@'.code)

        /** True if typing [codePoint] right after a word continues it rather than ending it. */
        fun continuesWord(codePoint: Int): Boolean =
            Character.isDigit(codePoint) || codePoint in CONTINUING_PUNCTUATION
    }

    /**
     * How the user capitalized the word being typed, and how suggestions are made to match it
     * (the same three cases as AOSP `Suggest.getTransformedSuggestedWordInfo`).
     */
    enum class WordCase {
        /** Lower case, or mixed like "iPhone": suggestions keep the dictionary's spelling. */
        AS_TYPED,

        /** Only the first letter is upper case (often auto-capitalization): capitalize suggestions. */
        CAPITALIZED,

        /** Two or more letters, all upper case: upper-case suggestions. */
        ALL_CAPS;

        /** [word] in this case. */
        fun apply(word: String, locale: Locale): String = when (this) {
            AS_TYPED -> word
            CAPITALIZED -> capitalizeFirst(word, locale)
            ALL_CAPS -> word.uppercase(locale)
        }

        companion object {
            /** The case of [typed]. */
            fun of(typed: String): WordCase {
                val letters = typed.filter(Char::isLetter)
                val upper = letters.count(Char::isUpperCase)
                return when {
                    letters.length > 1 && upper == letters.length -> ALL_CAPS
                    upper == 1 && typed.isNotEmpty() && typed[0].isUpperCase() -> CAPITALIZED
                    else -> AS_TYPED
                }
            }

            /** AOSP `WordComposer.isMostlyCaps`: more than one capital letter; never auto-corrected. */
            fun isMostlyCaps(typed: String): Boolean = typed.count(Char::isUpperCase) > 1

            /**
             * The form handed to the decoder. A leading capital is usually auto-capitalization, so
             * "Im" at the start of a sentence is looked up as "im" (whitelisted to "I'm") and the
             * capital is put back on the results by [apply].
             */
            fun decoderInput(typed: String, locale: Locale): String = when (of(typed)) {
                AS_TYPED -> typed
                CAPITALIZED -> lowerFirst(typed, locale)
                ALL_CAPS -> typed.lowercase(locale)
            }

            private fun capitalizeFirst(word: String, locale: Locale): String {
                if (word.isEmpty()) return word
                val end = word.offsetByCodePoints(0, 1)
                return word.substring(0, end).uppercase(locale) + word.substring(end)
            }

            private fun lowerFirst(word: String, locale: Locale): String {
                if (word.isEmpty()) return word
                val end = word.offsetByCodePoints(0, 1)
                return word.substring(0, end).lowercase(locale) + word.substring(end)
            }
        }
    }

    /**
     * Where a tap landed in its key, from the key's centre, in key widths ([dx], right is positive)
     * and key heights ([dy], down is positive). A tap on the key's right edge is dx = 0.5.
     */
    data class TapOffset(val dx: Float, val dy: Float) {
        companion object {
            /** A tap in the gap beside a key still counts for it, but no further out than this. */
            const val LIMIT = 0.6f

            /** The offset of a tap at ([x], [y]) on a key centred at ([centerX], [centerY]). */
            fun of(x: Float, y: Float, centerX: Float, centerY: Float, width: Float, height: Float): TapOffset? {
                if (width <= 0f || height <= 0f) return null
                return TapOffset(
                    ((x - centerX) / width).coerceIn(-LIMIT, LIMIT),
                    ((y - centerY) / height).coerceIn(-LIMIT, LIMIT),
                )
            }
        }
    }

    /**
     * Where each recently typed character was tapped, so the decoder hears that a tap on the O/P
     * border was nearly a P, not a clean O. The keyboard view reports each key press ([pressed]); the
     * character that press types is then [typed]. A character with no press behind it (a long-press
     * popup, a hardware key) has no position.
     *
     * [offsetsFor] lines the trail up with the word being suggested for from its end, so the trail
     * needs no resetting as words come and go: a word that doesn't match it (the cursor moved, a
     * whole word was deleted) simply gets no positions, which is how every word was decoded before.
     *
     * Written on the main thread, read while suggesting. Holds positions only, never logged.
     */
    class TapTrail(private val capacity: Int = CAPACITY) {
        private class Typed(val codePoint: Int, val offset: TapOffset?)

        private var pressedCodePoint = NONE
        private var pressedOffset: TapOffset? = null
        private val trail = ArrayDeque<Typed>()

        /** A key for [codePoint] went down at [offset]. */
        @Synchronized
        fun pressed(codePoint: Int, offset: TapOffset?) {
            pressedCodePoint = codePoint
            pressedOffset = offset
        }

        /** [text] went into the editor; its first character came from the last press if they match. */
        @Synchronized
        fun typed(text: String) {
            var first = true
            text.codePoints().forEach { codePoint ->
                val fromPress = first && sameKey(codePoint, pressedCodePoint)
                add(Typed(codePoint, if (fromPress) pressedOffset else null))
                first = false
            }
            pressedCodePoint = NONE
            pressedOffset = null
        }

        /** The last typed character was deleted. */
        @Synchronized
        fun deleted() {
            trail.removeLastOrNull()
        }

        @Synchronized
        fun clear() {
            trail.clear()
            pressedCodePoint = NONE
            pressedOffset = null
        }

        /**
         * Where each character of [word] was tapped, if the last characters typed spell [word] (in
         * any case); otherwise an empty list.
         */
        @Synchronized
        fun offsetsFor(word: String): List<TapOffset?> {
            val codePoints = word.codePoints().toArray()
            if (codePoints.isEmpty() || codePoints.size > trail.size) return emptyList()
            val start = trail.size - codePoints.size
            val matches = codePoints.indices.all { sameKey(trail[start + it].codePoint, codePoints[it]) }
            if (!matches) return emptyList()
            val offsets = List(codePoints.size) { trail[start + it].offset }
            return if (offsets.all { it == null }) emptyList() else offsets
        }

        private fun add(typed: Typed) {
            trail.addLast(typed)
            while (trail.size > capacity) trail.removeFirst()
        }

        private fun sameKey(a: Int, b: Int) = a == b || Character.toLowerCase(a) == Character.toLowerCase(b)

        private companion object {
            /** Longer than any word worth correcting. */
            const val CAPACITY = 64
            const val NONE = -1
        }
    }

    /**
     * A fixed QWERTY layout for the decoder's spatial model, in abstract units (a key is 100 x 150).
     * Each typed letter is fed to the decoder as a touch on its key here, at the same spot in the key
     * as the real tap ([touchPoint]), so the real on-screen layout and its size don't matter.
     *
     * The proximity grid follows AOSP `ProximityInfo.computeNearestNeighbors`: each grid cell lists
     * the keys whose edge lies within 1.2 key widths of the cell's center.
     */
    object QwertyGeometry {
        const val KEY_WIDTH = 100
        const val KEY_HEIGHT = 150
        const val KEYBOARD_WIDTH = 10 * KEY_WIDTH
        const val KEYBOARD_HEIGHT = 3 * KEY_HEIGHT
        const val GRID_WIDTH = 32
        const val GRID_HEIGHT = 16

        /** Native MAX_PROXIMITY_CHARS_SIZE. */
        const val MAX_PROXIMITY_CHARS = 16

        private const val SEARCH_DISTANCE = 1.2f
        private val ROWS = listOf("qwertyuiop", "asdfghjkl", "zxcvbnm")
        private val ROW_OFFSETS = intArrayOf(0, KEY_WIDTH / 2, KEY_WIDTH * 3 / 2)

        /** A key's code point and its top-left corner. */
        data class Key(val codePoint: Int, val x: Int, val y: Int)

        val keys: List<Key> = ROWS.flatMapIndexed { row, letters ->
            letters.mapIndexed { column, letter ->
                Key(letter.code, ROW_OFFSETS[row] + column * KEY_WIDTH, row * KEY_HEIGHT)
            }
        }

        private val keysByCodePoint = keys.associateBy { it.codePoint }

        /** The center of the key that types [codePoint] (either case), or null if there is none. */
        fun keyCenter(codePoint: Int): Pair<Int, Int>? {
            val key = keysByCodePoint[Character.toLowerCase(codePoint)] ?: return null
            return Pair(key.x + KEY_WIDTH / 2, key.y + KEY_HEIGHT / 2)
        }

        /**
         * Where a tap on the key for [codePoint] lands here: [offset] from its centre ([TapOffset]),
         * or the centre when the tap's position is unknown. Null if no key types [codePoint].
         */
        fun touchPoint(codePoint: Int, offset: TapOffset?): Pair<Int, Int>? {
            val (x, y) = keyCenter(codePoint) ?: return null
            if (offset == null) return Pair(x, y)
            return Pair(
                (x + offset.dx * KEY_WIDTH).roundToInt().coerceIn(0, KEYBOARD_WIDTH - 1),
                (y + offset.dy * KEY_HEIGHT).roundToInt().coerceIn(0, KEYBOARD_HEIGHT - 1),
            )
        }

        /** `GRID_WIDTH * GRID_HEIGHT * MAX_PROXIMITY_CHARS` code points near each cell, -1 padded. */
        fun proximityGrid(): IntArray {
            val cellWidth = (KEYBOARD_WIDTH + GRID_WIDTH - 1) / GRID_WIDTH
            val cellHeight = (KEYBOARD_HEIGHT + GRID_HEIGHT - 1) / GRID_HEIGHT
            val threshold = (KEY_WIDTH * SEARCH_DISTANCE).toInt()
            val grid = IntArray(GRID_WIDTH * GRID_HEIGHT * MAX_PROXIMITY_CHARS) { -1 }
            for (cell in 0 until GRID_WIDTH * GRID_HEIGHT) {
                val centerX = (cell % GRID_WIDTH) * cellWidth + cellWidth / 2
                val centerY = (cell / GRID_WIDTH) * cellHeight + cellHeight / 2
                keys.filter { squaredDistanceToEdge(it, centerX, centerY) < threshold * threshold }
                    .take(MAX_PROXIMITY_CHARS)
                    .forEachIndexed { slot, key -> grid[cell * MAX_PROXIMITY_CHARS + slot] = key.codePoint }
            }
            return grid
        }

        private fun squaredDistanceToEdge(key: Key, x: Int, y: Int): Int {
            val dx = maxOf(0, key.x - x, x - (key.x + KEY_WIDTH))
            val dy = maxOf(0, key.y - y, y - (key.y + KEY_HEIGHT))
            return dx * dx + dy * dy
        }
    }

    /**
     * The n-gram context before the word being typed: up to [MAX_COUNT] previous words, most recent
     * first, and whether a beginning-of-sentence marker follows them. Mirrors AOSP `NgramContext`.
     */
    data class PreviousWords(val words: List<String>, val isBeginningOfSentence: Boolean) {
        /** Native n-gram slots: each word, then the sentence-start marker if there is room. */
        val slotCount: Int
            get() = (words.size + if (isBeginningOfSentence) 1 else 0).coerceAtMost(MAX_COUNT)

        companion object {
            /** Native MAX_PREV_WORD_COUNT_FOR_N_GRAM. */
            const val MAX_COUNT = 3

            /** No usable context (for example after a comma). */
            val NONE = PreviousWords(emptyList(), isBeginningOfSentence = false)

            private const val SENTENCE_TERMINATORS = ".?!"
            private const val WORD_CONNECTORS = "'’-"
            private val WHITESPACE = Regex("\\s+")

            /** Stands in for the word being typed, so a prefix glued to it ("(word") stays with it. */
            private const val CURRENT_WORD = ""

            /**
             * The context for a word that starts right after [textBeforeWord]. Same rules as AOSP
             * `NgramContextUtils.getNgramContextFromNthPreviousWord`: only the current line counts,
             * words are split on whitespace, the start of the line or a word ending in `.?!` is a
             * sentence start, and any other trailing punctuation leaves the context unknown.
             */
            fun from(textBeforeWord: CharSequence): PreviousWords {
                val line = textBeforeWord.toString().substringAfterLast('\n')
                val tokens = (line + CURRENT_WORD).split(WHITESPACE)
                val words = ArrayList<String>(MAX_COUNT)
                for (i in 0 until MAX_COUNT) {
                    val index = tokens.size - 2 - i
                    val following = tokens[index + 1].firstOrNull()
                    if (following != null && following in WORD_CONNECTORS) break
                    if (index < 0) return PreviousWords(words, isBeginningOfSentence = true)
                    val token = tokens[index]
                    if (token.isEmpty() || token.last() in SENTENCE_TERMINATORS) {
                        return PreviousWords(words, isBeginningOfSentence = true)
                    }
                    if (!token.last().isLetterOrDigit()) break
                    words += token
                }
                return PreviousWords(words, isBeginningOfSentence = false)
            }
        }
    }

    /**
     * The AOSP auto-correction threshold check, ported from `AutoCorrectionUtils` (Java) and the
     * native `AutocorrectionThresholdUtils::calcNormalizedScore` / Damerau-Levenshtein edit distance
     * (Apache-2.0, The Android Open Source Project).
     */
    object AutoCorrection {
        /** AOSP's "modest" auto-correction threshold, the HeliBoard default. */
        const val MODEST_THRESHOLD = 0.185f

        /** Native SUGGEST_INTERFACE_OUTPUT_SCALE: decoder scores are fixed-point with this scale. */
        private const val OUTPUT_SCALE = 1_000_000f

        /** Native BASE_CHARS covers code points below this; above it characters compare as-is. */
        private const val BASE_CHARS_SIZE = 0x500

        /**
         * Port of `AutoCorrectionUtils.suggestionExceedsThreshold`: whitelisted words always pass,
         * words the decoder did not mark as appropriate never do, and the rest need a normalized
         * score of at least [threshold].
         */
        fun suggestionExceedsThreshold(suggestion: RawSuggestion, typed: String, threshold: Float): Boolean {
            if (suggestion.isWhitelist) return true
            if (!suggestion.isAppropriateForAutoCorrection) return false
            return normalizedScore(typed, suggestion.word, suggestion.score) >= threshold
        }

        /**
         * Port of `calcNormalizedScore`: the decoder score scaled back to a float and weighted down
         * by how much of [after] had to be edited to get there from [before].
         */
        fun normalizedScore(before: String, after: String, score: Int): Float {
            val beforeCodePoints = before.codePoints().toArray()
            val afterCodePoints = after.codePoints().toArray()
            if (beforeCodePoints.isEmpty() || afterCodePoints.isEmpty()) return 0f
            if (afterCodePoints.all { it == ' '.code }) return 0f
            val distance = editDistance(beforeCodePoints, afterCodePoints)
            if (score <= 0 || distance >= afterCodePoints.size) return 0f
            val weight = 1f - distance.toFloat() / afterCodePoints.size.toFloat()
            return score.toFloat() / OUTPUT_SCALE * weight
        }

        /**
         * Port of the native `EditDistance` with `DamerauLevenshteinEditDistancePolicy`: unit cost
         * insert/delete/substitute plus adjacent transposition, comparing base lower-case letters.
         */
        fun editDistance(before: IntArray, after: IntArray): Int {
            val a = IntArray(before.size) { baseLowerCase(before[it]) }
            val b = IntArray(after.size) { baseLowerCase(after[it]) }
            val width = b.size + 1
            val dp = IntArray((a.size + 1) * width)
            for (i in 0..a.size) dp[width * i] = i
            for (j in 0..b.size) dp[j] = j
            for (i in a.indices) {
                for (j in b.indices) {
                    val substitution = if (a[i] == b[j]) 0 else 1
                    var best = minOf(dp[width * i + j + 1] + 1, dp[width * (i + 1) + j] + 1, dp[width * i + j] + substitution)
                    if (i > 0 && j > 0 && a[i] == b[j - 1] && b[j] == a[i - 1]) {
                        best = minOf(best, dp[width * (i - 1) + j - 1] + substitution)
                    }
                    dp[width * (i + 1) + j + 1] = best
                }
            }
            return dp[dp.size - 1]
        }

        /** Native `CharUtils::toBaseLowerCase`: strip the accent (Latin range only), then lower-case. */
        internal fun baseLowerCase(codePoint: Int): Int {
            var base = codePoint
            if (codePoint < BASE_CHARS_SIZE) {
                val decomposed = Normalizer.normalize(String(Character.toChars(codePoint)), Normalizer.Form.NFD)
                base = decomposed.codePointAt(0)
            }
            return Character.toLowerCase(base)
        }
    }

    /**
     * The last auto-correction, kept so that a backspace straight after it puts back what was typed,
     * as AOSP's keyboard does ("teh " → "the ", backspace → "teh").
     *
     * [start] is where the corrected word begins in the editor.
     */
    data class AutoCorrectionUndo(val typed: String, val corrected: String, val start: Int) {
        /** Replace the [length] characters left of the cursor with [text]. */
        data class Revert(val length: Int, val text: String, val phantomSpace: Boolean)

        /**
         * How to undo with the cursor at [cursor] and [textBeforeCursor] left of it, or null if that
         * text is no longer the correction followed by the separator that triggered it.
         *
         * A space separator is dropped and left pending as a phantom space, so the user can fix the
         * word, add punctuation to it, or just type on and get the space back. Punctuation is kept.
         */
        fun revertAt(cursor: Int, textBeforeCursor: CharSequence): Revert? {
            val separatorLength = cursor - start - corrected.length
            if (separatorLength !in 1..MAX_SEPARATOR_LENGTH) return null
            val length = corrected.length + separatorLength
            if (textBeforeCursor.length < length) return null
            val replaced = textBeforeCursor.substring(textBeforeCursor.length - length)
            if (!replaced.startsWith(corrected)) return null
            val separator = replaced.substring(corrected.length)
            if (separator.any { it.isLetterOrDigit() }) return null
            return if (separator == " ") {
                Revert(length, typed, phantomSpace = true)
            } else {
                Revert(length, typed + separator, phantomSpace = false)
            }
        }

        private companion object {
            /** A space, a punctuation mark, or a punctuation mark plus the space added after it. */
            const val MAX_SEPARATOR_LENGTH = 2
        }
    }
}
