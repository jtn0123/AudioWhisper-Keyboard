/*
 * Copyright (C) 2020-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package dev.patrickgold.florisboard.ime.text.gestures

import android.content.res.Resources
import android.view.MotionEvent
import android.view.VelocityTracker
import com.jtn0123.audiowhisper.rust.Swipe
import com.jtn0123.audiowhisper.rust.SwipeDetector
import com.jtn0123.audiowhisper.rust.SwipeDirection
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.lib.Pointer

/**
 * Wrapper class which holds all enums, interfaces and classes for detecting a swipe gesture.
 */
abstract class SwipeGesture {
    /**
     * Class which detects swipes based on given [MotionEvent]s. Only supports single-finger swipes
     * and ignores additional pointers provided, if any.
     *
     * AudioWhisper: which way a finger swiped and by how many units is decided in Rust
     * (lib/rust/crates/gestures/src/swipe.rs); this reads the events, the velocity and the
     * preferences, and tells [listener].
     *
     * @property listener The listener to report detected swipes to.
     */
    class Detector(private val listener: Listener) {
        private val prefs by FlorisPreferenceStore

        var isEnabled: Boolean = true
        private val swipes = SwipeDetector()
        private val velocityTracker: VelocityTracker by lazy { VelocityTracker.obtain() }

        fun onTouchEvent(event: MotionEvent) {
            if (!isEnabled) return
            if (event.actionMasked == MotionEvent.ACTION_DOWN) {
                resetState()
            }
            velocityTracker.addMovement(event)
        }

        fun onTouchDown(event: MotionEvent, pointer: Pointer) {
            if (!isEnabled) return
            down(pointer.id, event.getX(pointer.index), event.getY(pointer.index), density())
        }

        fun onTouchMove(event: MotionEvent, pointer: Pointer, alwaysTriggerOnMove: Boolean): Boolean {
            if (!isEnabled) return false
            val x = event.getX(pointer.index)
            val y = event.getY(pointer.index)
            return move(pointer.id, x, y, density(), prefs.gestures.swipeDistanceThreshold.get(), alwaysTriggerOnMove)
        }

        fun onTouchUp(event: MotionEvent, pointer: Pointer): Boolean {
            if (!isEnabled) return false
            velocityTracker.computeCurrentVelocity(1000)
            val velocityX = velocityTracker.getXVelocity(pointer.id)
            val velocityY = velocityTracker.getYVelocity(pointer.id)
            val density = density()
            return up(
                pointer.id,
                event.getX(pointer.index),
                event.getY(pointer.index),
                velocityX,
                velocityY,
                density,
                prefs.gestures.swipeDistanceThreshold.get(),
                prefs.gestures.swipeVelocityThreshold.get(),
            )
        }

        fun onTouchCancel(event: MotionEvent, pointer: Pointer) {
            if (!isEnabled) return
            cancel(pointer.id)
        }

        /** Finger [id] went down at ([x], [y]) pixels. */
        internal fun down(id: Int, x: Float, y: Float, density: Float) = swipes.down(id, x, y, density)

        /** Finger [id] moved to ([x], [y]) pixels: what [listener] made of the swipe, if it was one. */
        internal fun move(id: Int, x: Float, y: Float, density: Float, distanceThreshold: Int, always: Boolean): Boolean {
            val swipe = swipes.move(id, x, y, density, distanceThreshold, always) ?: return false
            return listener.onSwipe(event(swipe, Type.TOUCH_MOVE, id))
        }

        /** Finger [id] was lifted at ([x], [y]) pixels: what [listener] made of the swipe, if it was one. */
        internal fun up(
            id: Int,
            x: Float,
            y: Float,
            velocityX: Float,
            velocityY: Float,
            density: Float,
            distanceThreshold: Int,
            velocityThreshold: Int,
        ): Boolean {
            val swipe = swipes.up(id, x, y, velocityX, velocityY, density, distanceThreshold, velocityThreshold)
                ?: return false
            return listener.onSwipe(event(swipe, Type.TOUCH_UP, id))
        }

        /** Finger [id]'s touch was cancelled. */
        internal fun cancel(id: Int) = swipes.cancel(id)

        /** A new touch began: forget every finger. */
        internal fun reset() = swipes.reset()

        private fun event(swipe: Swipe, type: Type, id: Int) = Event(
            direction = direction(swipe.direction),
            type = type,
            id,
            swipe.absX,
            swipe.absY,
            swipe.relX,
            swipe.relY,
        )

        private fun direction(direction: SwipeDirection) = when (direction) {
            SwipeDirection.UP_LEFT -> Direction.UP_LEFT
            SwipeDirection.UP -> Direction.UP
            SwipeDirection.UP_RIGHT -> Direction.UP_RIGHT
            SwipeDirection.RIGHT -> Direction.RIGHT
            SwipeDirection.DOWN_RIGHT -> Direction.DOWN_RIGHT
            SwipeDirection.DOWN -> Direction.DOWN
            SwipeDirection.DOWN_LEFT -> Direction.DOWN_LEFT
            SwipeDirection.LEFT -> Direction.LEFT
        }

        /** `ViewUtils.px2dp`'s density. */
        private fun density() = Resources.getSystem().displayMetrics.density

        /**
         * Resets the state.
         */
        private fun resetState() {
            reset()
            velocityTracker.clear()
        }
    }

    /**
     * An interface which provides an abstract callback function, which will be called for any
     * detected swipe event.
     */
    interface Listener {
        fun onSwipe(event: Event): Boolean
    }

    /**
     * Data class which describes a single gesture event.
     */
    data class Event(
        /** The direction of the swipe. */
        val direction: Direction,
        /** The type of the swipe. */
        val type: Type,
        /** The pointer ID of this event, corresponds to the value reported by the original MotionEvent. */
        val pointerId: Int,
        /** The unit count on the x-axis, measured from the first event (ACTION_DOWN). */
        val absUnitCountX: Int,
        /** The unit count on the y-axis, measured from the first event (ACTION_DOWN). */
        val absUnitCountY: Int,
        /** The unit count on the x-axis, measured from the last event (ACTION_MOVE). */
        val relUnitCountX: Int,
        /** The unit count on the y-axis, measured from the last event (ACTION_MOVE). */
        val relUnitCountY: Int,
    )

    /**
     * ENum which defines the direction of the detected swipe.
     */
    enum class Direction {
        UP_LEFT,
        UP,
        UP_RIGHT,
        RIGHT,
        DOWN_RIGHT,
        DOWN,
        DOWN_LEFT,
        LEFT;
    }

    /**
     * Enum which defines the type of the gesture.
     */
    enum class Type {
        TOUCH_UP,
        TOUCH_MOVE;
    }
}
