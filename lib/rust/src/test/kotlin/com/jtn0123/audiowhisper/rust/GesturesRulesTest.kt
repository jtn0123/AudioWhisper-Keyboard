package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * The swipe and glide detectors through JNI, as the keyboard calls them: every direction and the
 * unit counts unpacked, and every part of a glide's outcome. The rules themselves are tested in
 * Rust and, against the Kotlin they replace, in the app.
 */
class GesturesRulesTest {
    @Test
    fun everyDirectionCrosses() {
        val moves = listOf(
            -1f to -1f to SwipeDirection.UP_LEFT, 0f to -1f to SwipeDirection.UP, 1f to -1f to SwipeDirection.UP_RIGHT,
            1f to 0f to SwipeDirection.RIGHT, 1f to 1f to SwipeDirection.DOWN_RIGHT, 0f to 1f to SwipeDirection.DOWN,
            -1f to 1f to SwipeDirection.DOWN_LEFT, -1f to 0f to SwipeDirection.LEFT,
        )
        for ((move, direction) in moves) {
            val swipes = SwipeDetector()
            swipes.down(7, 500f, 500f, 2f)
            val (x, y) = move
            assertEquals(direction, swipes.move(7, 500f + x * 200, 500f + y * 200, 2f, 32, false)?.direction)
        }
    }

    @Test
    fun aSwipeCarriesItsUnits() {
        val swipes = SwipeDetector()
        swipes.down(0, 0f, 0f, 1f)
        assertNull(swipes.move(0, 16f, 0f, 1f, 32, false))
        assertEquals(Swipe(SwipeDirection.RIGHT, 2, 0, 2, 0), swipes.move(0, 17f, 0f, 1f, 32, false))
        assertEquals(Swipe(SwipeDirection.UP, 2, -3, 0, -3), swipes.move(0, 17f, -24f, 1f, 32, false))
        assertEquals(Swipe(SwipeDirection.UP_LEFT, -5, -9, -5, -9), swipes.up(0, -40f, -72f, -3000f, 0f, 1f, 32, 1900))
        assertNull(swipes.move(0, 0f, 0f, 1f, 32, true))
    }

    @Test
    fun cancelAndResetForgetFingers() {
        val swipes = SwipeDetector()
        swipes.down(0, 0f, 0f, 1f)
        swipes.down(1, 0f, 0f, 1f)
        swipes.cancel(0)
        assertNull(swipes.move(0, 0f, 0f, 1f, 32, true))
        assertEquals(Swipe(SwipeDirection.RIGHT, 0, 0, 0, 0), swipes.move(1, 0f, 0f, 1f, 32, true))
        swipes.reset()
        assertNull(swipes.move(1, 0f, 0f, 1f, 32, true))
        assertNull(swipes.up(1, 900f, 0f, 9000f, 0f, 1f, 32, 1900))
    }

    @Test
    fun aGlidesOutcomeUnpacks() {
        val glide = GlideDetector(36f)
        val down = glide.touch(DOWN, 0, floatArrayOf(0f, 0f), 2, 1_000, 1f, null)
        assertTrue(down.clearedBefore && down.took && down.started)
        assertFalse(down.gesture || down.complete || down.cancelled || down.clearedAfter)
        assertNull(down.decision)
        assertEquals(-1, down.emitFrom)
        val points = floatArrayOf(10f, 0f, 40f, 0f, 99f, 99f)
        val move = glide.touch(MOVE, 0, points, 4, 1_010, 1f, 'a'.code)
        assertTrue(move.gesture && move.took)
        assertFalse(move.clearedBefore || move.started)
        assertEquals(true, move.decision)
        assertEquals(0, move.emitFrom)
        assertEquals(3, glide.touch(MOVE, 0, floatArrayOf(50f, 0f), 2, 1_020, 1f, 'a'.code).emitFrom)
        val up = glide.touch(UP, 0, FloatArray(0), 0, 1_030, 1f, 'a'.code)
        assertTrue(up.complete && up.clearedAfter)
        assertFalse(up.gesture)
        assertNull(up.decision)
    }

    @Test
    fun aSlowTouchIsDecidedNotAGlideAndACancelReportsAGlide() {
        val glide = GlideDetector(36f)
        glide.touch(DOWN, 0, floatArrayOf(0f, 0f), 2, 0, 1f, null)
        assertEquals(false, glide.touch(MOVE, 0, floatArrayOf(1f, 0f), 2, 600, 1f, null).decision)
        glide.touch(DOWN, 0, floatArrayOf(0f, 0f), 2, 0, 1f, null)
        glide.touch(MOVE, 0, floatArrayOf(100f, 0f), 2, 1, 1f, null)
        val cancel = glide.touch(CANCEL, 0, FloatArray(0), 0, 2, 1f, null)
        assertTrue(cancel.cancelled && cancel.clearedAfter)
        glide.touch(DOWN, 0, floatArrayOf(0f, 0f), 2, 0, 1f, KEY_SPACE)
        assertNull(glide.touch(MOVE, 0, floatArrayOf(100f, 0f), 2, 1, 1f, KEY_SPACE).decision)
    }

    @Test
    fun badPointsThrowRatherThanCrash() {
        val glide = GlideDetector(36f)
        assertFailsWith<IllegalArgumentException> { glide.touch(DOWN, 0, FloatArray(0), 0, 0, 1f, null) }
        assertFailsWith<IllegalArgumentException> { glide.touch(MOVE, 0, FloatArray(2), 3, 0, 1f, null) }
        assertFailsWith<IllegalArgumentException> { glide.touch(MOVE, 0, FloatArray(2), -1, 0, 1f, null) }
        assertFailsWith<IllegalArgumentException> { glide.touch(MOVE, 0, FloatArray(3), 1, 0, 1f, null) }
    }

    private companion object {
        const val DOWN = 0
        const val UP = 1
        const val MOVE = 2
        const val CANCEL = 3
        const val KEY_SPACE = 32
    }
}
