package dev.patrickgold.florisboard.ime.text.gestures

import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.ime.text.gestures.SwipeGesture.Direction
import dev.patrickgold.florisboard.ime.text.gestures.SwipeGesture.Event
import dev.patrickgold.florisboard.ime.text.gestures.SwipeGesture.Listener
import dev.patrickgold.florisboard.ime.text.gestures.SwipeGesture.Type
import dev.patrickgold.florisboard.lib.Pointer
import dev.patrickgold.florisboard.lib.PointerMap
import kotlin.math.abs
import kotlin.math.atan2

/**
 * The Kotlin `SwipeGesture.Detector` replaced by lib/rust/crates/gestures/src/swipe.rs, kept to
 * test against. Edited only to compile without Android: positions, velocities, the density and the
 * preferences are passed in (`ViewUtils.px2dp` is `px / density`, and the velocities are what
 * `VelocityTracker` gave), and a new touch's reset ([onTouchEvent]) takes the event's action.
 */
internal class ReferenceSwipeDetector(private val listener: Listener) {
    var isEnabled: Boolean = true
    private var pointerMap: PointerMap<GesturePointer> = PointerMap { GesturePointer() }

    private fun px2dp(px: Float, density: Float) = px / density

    fun onTouchEvent(actionMasked: Int) {
        if (!isEnabled) return
        if (actionMasked == ACTION_DOWN) {
            resetState()
        }
    }

    fun onTouchDown(id: Int, index: Int, x: Float, y: Float, density: Float) {
        if (!isEnabled) return
        pointerMap.add(id, index)?.let { gesturePointer ->
            gesturePointer.firstX = px2dp(x, density)
            gesturePointer.firstY = px2dp(y, density)
            gesturePointer.lastX = gesturePointer.firstX
            gesturePointer.lastY = gesturePointer.firstY
        }
    }

    fun onTouchMove(id: Int, index: Int, x: Float, y: Float, density: Float, swipeDistanceThreshold: Int, alwaysTriggerOnMove: Boolean): Boolean {
        if (!isEnabled) return false
        pointerMap.findById(id)?.let { gesturePointer ->
            gesturePointer.index = index
            val currentX = px2dp(x, density)
            val currentY = px2dp(y, density)
            val absDiffX = currentX - gesturePointer.firstX
            val absDiffY = currentY - gesturePointer.firstY
            val relDiffX = currentX - gesturePointer.lastX
            val relDiffY = currentY - gesturePointer.lastY
            val thresholdWidth = swipeDistanceThreshold.dp.value.toDouble()
            val unitWidth = thresholdWidth / 4.0
            return if (alwaysTriggerOnMove || abs(relDiffX) > (thresholdWidth / 2.0) || abs(relDiffY) > (thresholdWidth / 2.0)) {
                gesturePointer.lastX = currentX
                gesturePointer.lastY = currentY
                val direction = detectDirection(relDiffX.toDouble(), relDiffY.toDouble())
                val newAbsUnitCountX = (absDiffX / unitWidth).toInt()
                val newAbsUnitCountY = (absDiffY / unitWidth).toInt()
                val relUnitCountX = newAbsUnitCountX - gesturePointer.absUnitCountX
                val relUnitCountY = newAbsUnitCountY - gesturePointer.absUnitCountY
                gesturePointer.absUnitCountX = newAbsUnitCountX
                gesturePointer.absUnitCountY = newAbsUnitCountY
                listener.onSwipe(Event(
                    direction = direction,
                    type = Type.TOUCH_MOVE,
                    id,
                    gesturePointer.absUnitCountX,
                    gesturePointer.absUnitCountY,
                    relUnitCountX,
                    relUnitCountY,
                ))
            } else {
                false
            }
        }
        return false
    }

    fun onTouchUp(
        id: Int,
        x: Float,
        y: Float,
        xVelocity: Float,
        yVelocity: Float,
        density: Float,
        swipeDistanceThreshold: Int,
        swipeVelocityThreshold: Int,
    ): Boolean {
        if (!isEnabled) return false
        pointerMap.findById(id)?.let { gesturePointer ->
            val currentX = px2dp(x, density)
            val currentY = px2dp(y, density)
            val absDiffX = currentX - gesturePointer.firstX
            val absDiffY = currentY - gesturePointer.firstY
            val velocityX = px2dp(xVelocity, density)
            val velocityY = px2dp(yVelocity, density)
            pointerMap.removeById(id)
            val thresholdSpeed = swipeVelocityThreshold.toDouble()
            val thresholdWidth = swipeDistanceThreshold.dp.value.toDouble()
            val unitWidth = thresholdWidth / 4.0
            return if ((abs(absDiffX) > thresholdWidth || abs(absDiffY) > thresholdWidth) && (abs(velocityX) > thresholdSpeed || abs(velocityY) > thresholdSpeed)) {
                val direction = detectDirection(absDiffX.toDouble(), absDiffY.toDouble())
                gesturePointer.absUnitCountX = (absDiffX / unitWidth).toInt()
                gesturePointer.absUnitCountY = (absDiffY / unitWidth).toInt()
                listener.onSwipe(Event(
                    direction = direction,
                    type = Type.TOUCH_UP,
                    id,
                    gesturePointer.absUnitCountX,
                    gesturePointer.absUnitCountY,
                    gesturePointer.absUnitCountX,
                    gesturePointer.absUnitCountY,
                ))
            } else {
                false
            }
        }
        return false
    }

    fun onTouchCancel(id: Int) {
        if (!isEnabled) return
        pointerMap.removeById(id)
    }

    /**
     * Calculates the angle based on the given x any y lengths. The returned angle is in degree
     * and goes clockwise, beginning with 0° at +x, 90° at +y, 180° at -y and 270° at -y.
     */
    private fun angle(diffX: Double, diffY: Double): Double {
        return (Math.toDegrees(atan2(diffY, diffX)) + 360) % 360
    }

    /**
     * Detects the direction of a finger swipe by two given events.
     */
    private fun detectDirection(diffX: Double, diffY: Double): Direction {
        val diffAngle = angle(diffX, diffY) / 360.0
        return when {
            diffAngle >= (1/16.0) && diffAngle < (3/16.0) ->        Direction.DOWN_RIGHT
            diffAngle >= (3/16.0) && diffAngle < (5/16.0) ->        Direction.DOWN
            diffAngle >= (5/16.0) && diffAngle < (7/16.0) ->        Direction.DOWN_LEFT
            diffAngle >= (7/16.0) && diffAngle < (9/16.0) ->        Direction.LEFT
            diffAngle >= (9/16.0) && diffAngle < (11/16.0) ->       Direction.UP_LEFT
            diffAngle >= (11/16.0) && diffAngle < (13/16.0) ->      Direction.UP
            diffAngle >= (13/16.0) && diffAngle < (15/16.0) ->      Direction.UP_RIGHT
            else ->                                                 Direction.RIGHT
        }
    }

    /**
     * Resets the state.
     */
    private fun resetState() {
        pointerMap.clear()
    }

    class GesturePointer : Pointer() {
        var firstX: Float = 0.0f
        var firstY: Float = 0.0f
        var lastX: Float = 0.0f
        var lastY: Float = 0.0f
        var absUnitCountX: Int = 0
        var absUnitCountY: Int = 0

        override fun reset() {
            super.reset()
            firstX = 0.0f
            firstY = 0.0f
            lastX = 0.0f
            lastY = 0.0f
            absUnitCountX = 0
            absUnitCountY = 0
        }
    }

    private companion object {
        /** `MotionEvent.ACTION_DOWN`. */
        const val ACTION_DOWN = 0
    }
}
