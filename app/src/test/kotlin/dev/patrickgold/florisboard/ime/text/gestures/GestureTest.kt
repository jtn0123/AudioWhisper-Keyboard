package dev.patrickgold.florisboard.ime.text.gestures

import dev.patrickgold.florisboard.ime.text.gestures.StatisticalGlideTypingClassifier.Gesture
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotSame

/**
 * The classifier reuses gestures across words, so a reused one must give exactly what a new one
 * did: the same points, and 0 past its end where a new gesture's storage was still empty.
 */
class GestureTest {
    private fun gestureOf(vararg points: Pair<Float, Float>, capacity: Int = 500) =
        Gesture(capacity).apply { points.forEach { (x, y) -> addPoint(x, y) } }

    private fun zigzag(points: Int) = Array(points) { i -> (i * 37f) to (if (i % 2 == 0) 100f else 260f) }

    @Test
    fun aReusedTargetMatchesANewGesture() {
        val word = gestureOf(*zigzag(6))
        // A target that grew from 8 points and still holds 300 different ones.
        val target = gestureOf(*zigzag(300), capacity = 8)

        word.resampleInto(SAMPLES, target)

        val fresh = word.resample(SAMPLES)
        assertEquals(fresh, target)
        for (i in 0 until 310) {
            assertEquals(fresh.getX(i), target.getX(i), "x at $i")
            assertEquals(fresh.getY(i), target.getY(i), "y at $i")
        }
    }

    @Test
    fun aReusedNormalizeTargetMatchesANewGesture() {
        val resampled = gestureOf(*zigzag(9)).resample(SAMPLES)
        val target = gestureOf(*zigzag(300))

        resampled.normalizeByBoxSideInto(target)

        assertEquals(resampled.normalizeByBoxSide(), target)
        assertEquals(0f, target.getX(250))
    }

    @Test
    fun pointsPastTheEndReadAsZero() {
        val gesture = gestureOf(1f to 2f, 3f to 4f, 5f to 6f)
        gesture.clear()
        gesture.addPoint(7f, 8f)

        assertEquals(7f, gesture.getX(0))
        assertEquals(0f, gesture.getX(1))
        assertEquals(0f, gesture.getY(2))
        assertEquals(0f, gesture.getX(-1))
    }

    @Test
    fun growsPastItsCapacityAndStopsAt500Points() {
        val gesture = Gesture(2)
        for (i in 0 until 600) gesture.addPoint(i.toFloat(), 0f)

        assertEquals(499f, gesture.getLastX())
        assertEquals(499f, gesture.getLength())
        assertEquals(0f, gesture.getX(500))
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
    fun aCloneIsEqualAndSeparate() {
        val gesture = gestureOf(*zigzag(5))
        val clone = gesture.clone()

        assertNotSame(gesture, clone)
        assertEquals(gesture, clone)
        assertEquals(gesture.hashCode(), clone.hashCode())

        clone.addPoint(1f, 1f)
        assertEquals(5, (0 until 10).count { gesture.getX(it) != 0f || gesture.getY(it) != 0f })
    }

    @Test
    fun aSinglePointResamplesToItselfRepeated() {
        val resampled = gestureOf(5f to 9f).resample(SAMPLES)

        // The point, then the classifier's 200 samples of it.
        for (i in 0..200) assertEquals(5f, resampled.getX(i), "x at $i")
        assertEquals(0f, resampled.getX(201))
    }

    private companion object {
        const val SAMPLES = 200
    }
}
