package com.jtn0123.audiowhisper.rust

import kotlin.math.exp
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Swipe typing's rules through JNI, as the keyboard calls them: arrays in and out whole, a
 * dictionary's worth of words in one call, and bad input thrown. The rules themselves are tested in
 * Rust and, against the Kotlin they replace, in the app.
 */
class SwipeRulesTest {
    private val none = SwipeRules.Context(0, -1, -1)

    @Test
    fun guessesComeBackInOrder() {
        val habit = SwipeRules.Context(pairCount = 1, ngram = -1, probability = -1)
        val order = SwipeRules.rankGlide(floatArrayOf(3.28f, 3.35f, 3.54f), listOf(none, habit, none))
        assertContentEquals(intArrayOf(1, 0, 2), order)
        val odd = floatArrayOf(Float.NaN, Float.POSITIVE_INFINITY, 0f, -0f, Float.NEGATIVE_INFINITY)
        assertContentEquals(intArrayOf(4, 3, 2, 1, 0), SwipeRules.rankGlide(odd, List(5) { none }))
        assertContentEquals(intArrayOf(), SwipeRules.rankGlide(floatArrayOf(), emptyList()))
    }

    @Test
    fun bonusesAreCapped() {
        assertEquals(0f, SwipeRules.glideBonus(0, -1, -1))
        assertEquals(SwipeRules.maxGlideBonus, SwipeRules.glideBonus(1000, 200, 159))
        assertEquals(1.6f, SwipeRules.maxGlideBonus)
    }

    @Test
    fun aDictionaryIsFilteredInOneCall() {
        val words = List(200_000) { if (it % 3 == 0) "mp$it" else "word" }
        val swipeable = SwipeRules.swipeable(words)
        assertEquals(133_333, swipeable.size)
        assertTrue(swipeable.all { it == "word" })
        assertEquals(listOf("", "to", "", "don't").filter { SwipeRules.isSwipeable(it) }, SwipeRules.swipeable(listOf("", "to", "", "don't")))
        assertEquals(emptyList(), SwipeRules.swipeable(emptyList()))
        assertTrue(SwipeRules.isSwipeable("rock’n’roll"))
        assertFalse(SwipeRules.isSwipeable("US$"))
        assertFalse(SwipeRules.isSwipeable("a\uD800"))
    }

    @Test
    fun userWordsKeepTheirOrderAndStrings() {
        val learned = listOf("bae", "mp3", "bae")
        val user = SwipeRules.userWords(learned, listOf("Kaito", "bae", "yeet"), intArrayOf(300, 200, 130))
        // Filled as the Kotlin filled it, so it lists them in the same (hash) order.
        val kotlin = HashMap<String, Int>().apply {
            put("bae", 120)
            put("Kaito", 255)
            put("bae", 200)
            put("yeet", 130)
        }
        assertEquals(kotlin.toList(), user.toList())
    }

    @Test
    fun frequenciesComeFromRust() {
        assertEquals(1.0, SwipeRules.frequencyOf(255))
        assertEquals(exp(-SwipeRules.costPerStep * SwipeRules.common), SwipeRules.frequencyOf(0), 1e-12)
        assertEquals(0.0, SwipeRules.frequency(-1, -1))
        assertEquals(SwipeRules.frequencyOf(SwipeRules.learnedProbability), SwipeRules.frequency(10, 120))
        assertEquals(listOf(255, 170, 120), listOf(SwipeRules.maxProbability, SwipeRules.common, SwipeRules.learnedProbability))
    }

    @Test
    fun badInputThrows() {
        assertFailsWith<IllegalArgumentException> { SwipeRules.rankGlide(floatArrayOf(1f), emptyList()) }
        assertFailsWith<IllegalArgumentException> { SwipeRules.userWords(emptyList(), listOf("a"), intArrayOf()) }
        assertFailsWith<IllegalArgumentException> { SwipeNative.rankGlide(floatArrayOf(1f), intArrayOf(0), intArrayOf(), intArrayOf(0)) }
        assertFailsWith<IllegalArgumentException> { SwipeNative.swipeable("toofab", intArrayOf(3, 2)) }
        assertFailsWith<IllegalArgumentException> { SwipeNative.swipeable("to", intArrayOf(3)) }
        assertFailsWith<IllegalArgumentException> { SwipeNative.swipeable("to", intArrayOf(3, -1)) }
    }
}
