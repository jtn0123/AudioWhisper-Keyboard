package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.util.VelocityTracker
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Velocity
import kotlin.math.abs
import kotlin.math.sign

/**
 * One finger dragging a Compose 1.12.1 `verticalScroll`, as its shipped bytecode does it:
 * TouchSlopDetector judges the slop at each event's final position and the first scroll is the
 * over-slop; DragGestureNode (isDraggableVelocityTrackerFixEnabled is false) feeds the tracker
 * the down, then every sample of the slop event and later moves, except that `awaitDragOrUp`
 * passes over an event whose pointer has not moved in x or y; Lsq2VelocityTracker records no
 * up but forgets a gesture whose up came over 40 ms after the last change; ScrollState keeps the
 * float position and shows round(position). The fit is the real `VelocityTracker` (Lsq2) and the
 * fling distance the real `FlingCalculator` at Android's scroll friction.
 */
internal class ReferenceSettingsDrag(private val density: Float, var position: Float, private val max: Float) {
    private val slop = (8 * density + .5f).toInt().toFloat()
    private val maximum = (8000 * density + .5f).toInt().toFloat()
    private val tracker = VelocityTracker()
    private var downAt = 0L
    private var downY = 0f
    private var lastX = 0f
    private var lastY = 0f
    private var lastChange = 0L
    private var dragging = false
    val value get() = Math.round(position)

    fun down(ms: Long, x: Float, y: Float) {
        downAt = ms; downY = y; lastX = x; lastY = y; dragging = false
    }

    fun move(samples: List<Pair<Long, Float>>, x: Float) {
        val y = samples.last().second
        if (dragging && x == lastX && y == lastY) return
        var delta = y - lastY
        if (!dragging) {
            val total = y - downY
            if (total == 0f || abs(total) < slop) { lastX = x; lastY = y; return }
            dragging = true
            tracker.resetTracking()
            tracker.addPosition(downAt, Offset(0f, downY))
            delta = total - sign(total) * slop
        }
        for ((ms, sample) in samples) tracker.addPosition(ms, Offset(0f, sample))
        lastChange = samples.last().first
        scrollBy(-delta)
        lastX = x; lastY = y
    }

    /** The fling velocity in scroll units per second, or null when the gesture never dragged. */
    fun up(ms: Long): Float? {
        if (!dragging) return null
        if (ms - lastChange > 40) tracker.resetTracking()
        val velocity = -tracker.calculateVelocity(Velocity(maximum, maximum)).y
        if (abs(velocity) > 1f) scrollBy(flingDistance(velocity) * sign(velocity))
        return velocity
    }

    private fun scrollBy(delta: Float) {
        position = (position + delta).coerceIn(0f, max)
    }

    private fun flingDistance(velocity: Float): Float {
        // ViewConfiguration.getScrollFriction() is a stub off-device; SCROLL_FRICTION is 0.015.
        val calculator = Class.forName("androidx.compose.animation.FlingCalculator")
            .getConstructor(Float::class.javaPrimitiveType, Density::class.java)
            .newInstance(.015f, Density(density))
        return calculator.javaClass.getMethod("flingDistance", Float::class.javaPrimitiveType)
            .invoke(calculator, velocity) as Float
    }
}
