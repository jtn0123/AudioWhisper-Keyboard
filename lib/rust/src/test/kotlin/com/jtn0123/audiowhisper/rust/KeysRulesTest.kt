package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

/**
 * The text keyboard's geometry through JNI, as the keyboard calls it: factors, rects and indexes
 * unpacked whole, and bad arrays turned into exceptions. The geometry itself is tested in Rust and,
 * against the Kotlin it replaces, in the app.
 */
class KeysRulesTest {
    @Test
    fun flexGivesShrinkGrowAndWidth() {
        val shift = KeyGeometry.flex(mode = 0, code = -11, numeric = false)
        assertEquals(listOf(1.5f, 0.0f, 1.56f), listOf(shift.shrink, shift.grow, shift.widthFactor))
        val number = KeyGeometry.flex(mode = 5, code = 49, numeric = true)
        assertEquals(listOf(1.0f, 1.0f, 1.56f), listOf(number.shrink, number.grow, number.widthFactor))
    }

    @Test
    fun desiredKeyGivesTouchThenVisibleBounds() {
        assertContentEquals(
            floatArrayOf(0f, 0f, 108f, 200f, 4f, 6f, 104f, 194f),
            KeyGeometry.desiredKey(0, 1080f, 800f, 4, 300f, 4f, 6f),
        )
    }

    @Test
    fun layoutGivesEightFloatsAKeyInRowOrder() {
        val desired = floatArrayOf(0f, 0f, 100f, 50f, 5f, 5f, 95f, 45f)
        val flex = floatArrayOf(1f, 1f, 0f, 1f, 1f, 1f, 2f, 1f, 0f)
        val bounds = KeyGeometry.layout(intArrayOf(2, 0, 1), flex, desired, 210f, 250f, true)
        assertContentEquals(
            floatArrayOf(
                0f, 0f, 105f, 50f, 10f, 5f, 100f, 45f,
                105f, 0f, 210f, 50f, 110f, 5f, 200f, 45f,
                0f, 200f, 205f, 300f, 10f, 205f, 200f, 245f,
            ),
            bounds,
        )
        assertNull(KeyGeometry.layout(intArrayOf(1), floatArrayOf(1f, 1f, 0f), desired, Float.NaN, 50f, false))
        assertContentEquals(floatArrayOf(), KeyGeometry.layout(intArrayOf(), floatArrayOf(), desired, 100f, 50f, false))
    }

    @Test
    fun keyAtCountsEveryRowAndStopsAtAnEmptyOne() {
        val touch = floatArrayOf(0f, 0f, 10f, 10f, 10f, 0f, 20f, 10f, 0f, 10f, 20f, 20f)
        assertEquals(1, KeyGeometry.keyAt(intArrayOf(2, 1), touch, 10f, 0f))
        assertEquals(2, KeyGeometry.keyAt(intArrayOf(2, 1), touch, 5f, 15f))
        assertEquals(-1, KeyGeometry.keyAt(intArrayOf(2, 1), touch, 20f, 5f))
        assertEquals(-1, KeyGeometry.keyAt(intArrayOf(2, 0, 1), touch, 5f, 15f))
    }

    @Test
    fun badArraysThrow() {
        val desired = floatArrayOf(0f, 0f, 100f, 50f, 5f, 5f, 95f, 45f)
        assertFailsWith<IllegalArgumentException> {
            KeyGeometry.layout(intArrayOf(2), floatArrayOf(1f, 1f, 0f), desired, 100f, 50f, false)
        }
        assertFailsWith<IllegalArgumentException> {
            KeyGeometry.layout(intArrayOf(1), floatArrayOf(1f, 1f), desired, 100f, 50f, false)
        }
        assertFailsWith<IllegalArgumentException> {
            KeyGeometry.layout(intArrayOf(1), floatArrayOf(1f, 1f, 0f), floatArrayOf(0f), 100f, 50f, false)
        }
        assertFailsWith<IllegalArgumentException> {
            KeyGeometry.keyAt(intArrayOf(-1, 2), FloatArray(4), 0f, 0f)
        }
        assertFailsWith<IllegalArgumentException> { KeyGeometry.keyAt(intArrayOf(1), FloatArray(8), 0f, 0f) }
    }
}
