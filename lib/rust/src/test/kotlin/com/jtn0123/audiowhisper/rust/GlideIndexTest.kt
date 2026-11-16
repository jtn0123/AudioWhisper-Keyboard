package com.jtn0123.audiowhisper.rust

import kotlin.math.ceil
import kotlin.math.hypot
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/** The Rust scorer through JNI, as the keyboard calls it; its scoring is tested in Rust. */
class GlideIndexTest {
    /** QWERTY letters on 100 x 150 keys, each row shifted half a key. */
    private val rows = listOf("qwertyuiop", "asdfghjkl", "zxcvbnm")
    private val keys = run {
        val letters = rows.flatMapIndexed { row, letters -> letters.mapIndexed { i, c -> Triple(c, i, row) } }
        GlideKeys(
            codes = IntArray(letters.size) { letters[it].first.code },
            bounds = FloatArray(letters.size * 4) { at ->
                val (_, i, row) = letters[at / 4]
                when (at % 4) {
                    0 -> 50f + 100f * i + 50f * row
                    1 -> 75f + 150f * row
                    2 -> 100f
                    else -> 150f
                }
            },
        )
    }

    private fun centre(c: Char): Pair<Float, Float> {
        val at = keys.codes.indexOf(c.code)
        return keys.bounds[at * 4] to keys.bounds[at * 4 + 1]
    }

    /** A swipe through [word]'s keys, with a point every 20 px. */
    private fun swipe(word: String): Pair<FloatArray, FloatArray> {
        val xs = ArrayList<Float>()
        val ys = ArrayList<Float>()
        var (x, y) = centre(word[0])
        xs += x
        ys += y
        for (c in word.drop(1)) {
            val (toX, toY) = centre(c)
            val steps = ceil(hypot(toX - x, toY - y) / 20f).toInt().coerceAtLeast(1)
            for (s in 1..steps) {
                xs += x + (toX - x) * s / steps
                ys += y + (toY - y) * s / steps
            }
            x = toX
            y = toY
        }
        return xs.toFloatArray() to ys.toFloatArray()
    }

    /** Lower-case letters on their own keys; [noKey] characters have none. */
    private fun describe(c: Char, noKey: Set<Char> = emptySet()) = when {
        c in noKey || c.lowercaseChar() !in 'a'..'z' -> GlideChar(c, GlideChar.NO_KEY, GlideChar.NO_KEY)
        else -> GlideChar(c.lowercaseChar(), c.lowercaseChar().code, c.lowercaseChar().code)
    }

    private fun guesses(words: List<String>, swiped: String, max: Int = 3): List<String> {
        val index = GlideIndex.build(GlideWords(words)) { describe(it) }
        val (xs, ys) = swipe(swiped)
        return index.query(xs, ys, keys).use { query ->
            val candidates = query.candidates()
            val scores = query.score(candidates, FloatArray(candidates.size) { 1f }, max)
            scores.words.map { words[it] }
        }
    }

    @Test
    fun aCleanSwipeFindsItsWord() {
        val words = listOf("hello", "help", "hell", "jello", "world", "word", "would", "hold")

        assertEquals("hello", guesses(words, "hello").first())
        assertEquals("world", guesses(words, "world").first())
    }

    @Test
    fun capitalsUseTheirLowerCaseKeys() {
        assertEquals(listOf("Hello"), guesses(listOf("Hello", "quiz"), "hello"))
    }

    @Test
    fun weightsRiseWithTheMostLikelyFirst() {
        val words = listOf("hello", "help", "hell", "jello")
        val index = GlideIndex.build(GlideWords(words)) { describe(it) }
        val (xs, ys) = swipe("hello")

        val scores = index.query(xs, ys, keys).use { query ->
            val candidates = query.candidates()
            query.score(candidates, FloatArray(candidates.size) { 1f }, 8)
        }

        assertTrue(scores.words.isNotEmpty())
        assertEquals(scores.weights.sorted(), scores.weights.toList())
    }

    @Test
    fun distinctCharsAreListedOnceInOrder() {
        assertContentEquals("abn".toCharArray(), GlideWords(listOf("banana", "nab")).distinctChars())
    }

    @Test
    fun aClosedQueryThrows() {
        val index = GlideIndex.build(GlideWords(listOf("hello"))) { describe(it) }
        val (xs, ys) = swipe("hello")
        val query = index.query(xs, ys, keys)
        query.close()

        assertFailsWith<IllegalArgumentException> { query.candidates() }
        query.close()
    }

    @Test
    fun mismatchedArgumentsThrow() {
        val index = GlideIndex.build(GlideWords(listOf("hello"))) { describe(it) }
        val (xs, ys) = swipe("hello")

        assertFailsWith<IllegalArgumentException> { index.query(xs, ys.copyOf(1), keys) }
        assertFailsWith<IllegalArgumentException> { GlideKeys(intArrayOf(1), FloatArray(3)) }
        index.query(xs, ys, keys).use { query ->
            assertFailsWith<IllegalArgumentException> { query.score(intArrayOf(0), FloatArray(0), 3) }
            assertFailsWith<IllegalArgumentException> { query.score(intArrayOf(5), floatArrayOf(1f), 3) }
            assertFailsWith<IllegalArgumentException> { query.score(intArrayOf(-1), floatArrayOf(1f), 3) }
        }
    }

    @Test
    fun aWordWithoutEndKeysIsLeftOut() {
        val words = listOf("hello", "hello'")
        val index = GlideIndex.build(GlideWords(words)) { describe(it, noKey = setOf('\'')) }
        val (xs, ys) = swipe("hello")

        index.query(xs, ys, keys).use { query ->
            assertContentEquals(intArrayOf(0), query.candidates())
        }
    }

    /** The best of [words] for the swipe through [xs] and [ys] (drawn at [times]), each at its frequency in [frequencies]. */
    private fun best(words: List<String>, frequencies: FloatArray, xs: FloatArray, ys: FloatArray, times: FloatArray, tuned: Boolean): String {
        val index = GlideIndex.build(GlideWords(words)) { describe(it) }
        return index.query(xs, ys, times, keys).use { query ->
            val candidates = query.candidates()
            val candidateFrequencies = FloatArray(candidates.size) { frequencies[candidates[it]] }
            val scores = if (tuned) query.scoreTuned(candidates, candidateFrequencies, 2) else query.score(candidates, candidateFrequencies, 2)
            words[scores.words[0]]
        }
    }

    @Test
    fun theKeyboardsScoringWeighsTheKeyTheSwipeEndsOn() {
        // From "a" to 35 px past "s", towards "d": at these frequencies FlorisBoard's classifier
        // picks "ad", and the keyboard the "s" the swipe stops on.
        val words = listOf("as", "ad")
        val frequencies = floatArrayOf(0.9f, 1f)
        val xs = FloatArray(28) { 100f + 5f * it }
        val ys = FloatArray(28) { 225f }
        assertEquals("ad", best(words, frequencies, xs, ys, FloatArray(0), tuned = false))
        assertEquals("as", best(words, frequencies, xs, ys, FloatArray(0), tuned = true))
    }

    @Test
    fun theKeyboardsScoringWeighsWhereTheFingerSlowed() {
        // Through "w" and "e" from "a" to "d": "awd" and "aed" are mirror images, and where the
        // finger slows tells them apart.
        val words = listOf("awd", "aed")
        val frequencies = floatArrayOf(1f, 1f)
        val (xs, ys) = swipe("awed")
        fun slowAt(c: Char): FloatArray {
            val (x, y) = centre(c)
            var t = 0f
            return FloatArray(xs.size) { i ->
                if (i > 0) t += if (hypot(xs[i] - x, ys[i] - y) < 60f) 8f else 1f
                t
            }
        }
        assertEquals("awd", best(words, frequencies, xs, ys, slowAt('w'), tuned = true))
        assertEquals("aed", best(words, frequencies, xs, ys, slowAt('e'), tuned = true))
    }

    @Test
    fun theOriginalScoreIgnoresTimes() {
        val words = listOf("hello", "help", "hell", "jello")
        val index = GlideIndex.build(GlideWords(words)) { describe(it) }
        val (xs, ys) = swipe("hello")
        val times = FloatArray(xs.size) { it * it.toFloat() }
        fun scores(times: FloatArray?) = (if (times == null) index.query(xs, ys, keys) else index.query(xs, ys, times, keys)).use { query ->
            val candidates = query.candidates()
            query.score(candidates, FloatArray(candidates.size) { 1f }, 8)
        }
        val untimed = scores(null)
        val timed = scores(times)
        assertContentEquals(untimed.words, timed.words)
        assertContentEquals(untimed.weights, timed.weights)
    }

    @Test
    fun timesAreOnePerPointOrNone() {
        val index = GlideIndex.build(GlideWords(listOf("hello"))) { describe(it) }
        val (xs, ys) = swipe("hello")

        assertFailsWith<IllegalArgumentException> { index.query(xs, ys, FloatArray(2), keys) }
        index.query(xs, ys, FloatArray(0), keys).use { query ->
            val candidates = query.candidates()
            assertContentEquals(intArrayOf(0), query.scoreTuned(candidates, floatArrayOf(1f), 3).words)
        }
    }
}
