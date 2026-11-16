package dev.patrickgold.florisboard.ime.text.gestures

import dev.patrickgold.florisboard.ime.text.gestures.StatisticalGlideTypingClassifier.Gesture
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertNotSame
import kotlin.test.assertTrue

/**
 * The swipe as the classifier collects it. Its storage is reused and grows, so what it hands the
 * scorer ([Gesture.xs], [Gesture.ys], [Gesture.times]) and how it compares (the suggestion cache's key) must see only
 * the points in it now. The scoring itself is tested in lib/rust and by [SwipeIndexTest].
 */
class GestureTest {
    private fun gestureOf(vararg points: Pair<Float, Float>, capacity: Int = 500) =
        Gesture(capacity).apply { points.forEach { (x, y) -> addPoint(x, y) } }

    private fun zigzag(points: Int) = Array(points) { i -> (i * 37f) to (if (i % 2 == 0) 100f else 260f) }

    @Test
    fun aClearedGestureHandsOverOnlyItsNewPoints() {
        val gesture = gestureOf(1f to 2f, 3f to 4f, 5f to 6f)
        gesture.clear()
        assertTrue(gesture.isEmpty)
        gesture.addPoint(7f, 8f)

        assertContentEquals(floatArrayOf(7f), gesture.xs())
        assertContentEquals(floatArrayOf(8f), gesture.ys())
        assertEquals(7f, gesture.getLastX())
        assertEquals(8f, gesture.getLastY())
    }

    @Test
    fun growsPastItsCapacityAndStopsAt500Points() {
        val gesture = Gesture(2)
        for (i in 0 until 600) gesture.addPoint(i.toFloat(), 0f)

        assertEquals(499f, gesture.getLastX())
        assertContentEquals(FloatArray(500) { it.toFloat() }, gesture.xs())
        assertEquals(500, gesture.ys().size)
    }

    @Test
    fun equalGesturesHashTheSame() {
        val reused = gestureOf(*zigzag(20))
        reused.clear()
        reused.addPoint(1f, 1f)
        reused.addPoint(2f, 3f)
        val fresh = gestureOf(1f to 1f, 2f to 3f, capacity = 2)

        assertEquals(fresh, reused)
        assertEquals(fresh.hashCode(), reused.hashCode())
    }

    @Test
    fun eachPointKeepsItsTimeAndTimesCount() {
        val gesture = Gesture(2)
        for (i in 0 until 5) gesture.addPoint(i.toFloat(), 0f, i * 10f)
        assertContentEquals(floatArrayOf(0f, 10f, 20f, 30f, 40f), gesture.times())
        val clone = gesture.clone()
        assertContentEquals(gesture.times(), clone.times())
        gesture.clear()
        gesture.addPoint(7f, 8f)
        assertContentEquals(floatArrayOf(0f), gesture.times(), "a point added without a time has none")

        val sooner = Gesture().apply { addPoint(1f, 1f, 5f) }
        val later = Gesture().apply { addPoint(1f, 1f, 9f) }
        assertNotEquals(sooner, later, "the same points drawn at other times may score differently")
    }

    @Test
    fun aCloneIsEqualAndSeparate() {
        val gesture = gestureOf(*zigzag(5))
        val clone = gesture.clone()

        assertNotSame(gesture, clone)
        assertEquals(gesture, clone)
        assertEquals(gesture.hashCode(), clone.hashCode())

        clone.addPoint(1f, 1f)
        assertEquals(5, gesture.xs().size)
        assertEquals(6, clone.xs().size)
    }
}
