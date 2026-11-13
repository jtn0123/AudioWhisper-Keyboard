package com.jtn0123.audiowhisper.rust

import kotlin.math.abs
import kotlin.math.exp
import kotlin.math.ln
import kotlin.math.sign
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Test-only reference for Compose AndroidFlingSpline/FlingCalculator (Apache 2.0).
 * Constants and float/double boundaries checked against the installed Compose animation AAR.
 */
private object ReferencePanelScroll {
    private val positions = FloatArray(101) { 1f }.also { samples ->
        var lower = 0f
        repeat(100) { index ->
            val alpha = index / 100f
            var upper = 1f
            while (true) {
                val x = lower + (upper - lower) / 2f
                val coefficient = 3f * x * (1 - x)
                val estimate = coefficient * ((1 - x) * .175f + x * .35000002f) + x * x * x
                if (abs(estimate - alpha) < .00001) {
                    samples[index] = coefficient * ((1 - x) * .5f + x) + x * x * x
                    break
                }
                if (estimate > alpha) upper = x else lower = x
            }
        }
    }
    fun value(density: Float, velocity: Float, time: Float): FloatArray {
        val coefficient = .015f * (386.0878f * density * 160f * .84f)
        val rate = (ln(.78) / ln(.9)).toFloat().toDouble()
        val logarithm = ln((.35f * abs(velocity)).toDouble() / coefficient)
        val duration = (1000 * exp(logarithm / (rate - 1))).toLong()
        val distance = (coefficient * exp(rate / (rate - 1) * logarithm)).toFloat()
        val t = time.coerceIn(0f, 1f)
        val index = (t * 100).toInt()
        val fraction = if (index >= 100) 1f else {
            val slope = (positions[index + 1] - positions[index]) / ((index + 1) / 100f - index / 100f)
            positions[index] + (t - index / 100f) * slope
        }
        return floatArrayOf(duration.toFloat(), distance, distance * sign(velocity) * fraction)
    }
}

class PanelScrollMatchTest {
    @Test fun splineDecayMatchesComposeReferenceAcrossDensitiesVelocitiesAndTimes() {
        for (density in listOf(.75f, 1f, 1.5f, 2.625f, 3f, 4f)) {
            for (velocity in listOf(-12000f, -4000f, -760f, -50f, 0f, 50f, 760f, 4000f, 12000f)) {
                for (step in -1..101) {
                    val time = step / 100f
                    val expected = ReferencePanelScroll.value(density, velocity, time)
                    val actual = PanelRules.scroll(floatArrayOf(density, velocity, time))
                    assertEquals(expected[0], actual[0])
                    for (i in 1..2) assertTrue(abs(expected[i] - actual[i]) <= .003f, "spline component $i")
                }
            }
        }
    }
}
