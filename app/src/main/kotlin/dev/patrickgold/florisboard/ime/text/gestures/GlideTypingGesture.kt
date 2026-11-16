/*
 * Copyright (C) 2025 The FlorisBoard Contributors
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

import android.content.Context
import android.content.res.Resources
import android.view.MotionEvent
import com.jtn0123.audiowhisper.rust.GlideDetector
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.text.keyboard.TextKey
import dev.patrickgold.florisboard.lib.util.ViewUtils
import kotlin.math.pow
import kotlin.math.sqrt

/**
 * Wrapper class which holds all enums, interfaces and classes for detecting a gesture.
 */
class GlideTypingGesture {
    /**
     * Class which detects swipes based on given [MotionEvent]s. Only supports single-finger swipes
     * and ignores additional pointers provided, if any.
     *
     * AudioWhisper: whether a touch is a glide, and which of its points the listeners hear of, is
     * decided in Rust (lib/rust/crates/gestures/src/glide_start.rs); this reads the events, keeps
     * the points and calls the listeners. [clock] is in milliseconds, read once an event.
     */
    class Detector internal constructor(keySize: Float, private val clock: () -> Long) {
        constructor(context: Context) : this(
            ViewUtils.px2dp(context.resources.getDimension(R.dimen.key_width)),
            System::currentTimeMillis,
        )

        private val pointerData: PointerData = PointerData(mutableListOf(), 0)
        private val listeners: ArrayList<Listener> = arrayListOf()
        private val detector = GlideDetector(keySize)

        /** The event's points for the detector: x, y, x, y, … */
        private var points = FloatArray(POINTS)

        /**
         * Method which evaluates if a given [event] is a gesture.
         *
         * @return whether or not the event was interpreted as part of a gesture.
         */
        fun onTouchEvent(event: MotionEvent, initialKey: TextKey?): Boolean {
            val index = event.actionIndex
            val count = when (event.actionMasked) {
                MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> point(0, event.getX(index), event.getY(index))
                MotionEvent.ACTION_MOVE -> {
                    val history = event.historySize
                    for (i in 0 until history) point(i, event.getHistoricalX(index, i), event.getHistoricalY(index, i))
                    point(history, event.getX(index), event.getY(index))
                }
                else -> 0
            }
            val density = Resources.getSystem().displayMetrics.density
            return touch(event.actionMasked, event.getPointerId(index), points, count, density, initialKey?.computedData?.code)
        }

        /** Puts the [i]th point in [points], growing it if needed: how many floats it now holds. */
        private fun point(i: Int, x: Float, y: Float): Int {
            if (points.size < 2 * i + 2) points = points.copyOf(2 * points.size.coerceAtLeast(i + 1))
            points[2 * i] = x
            points[2 * i + 1] = y
            return 2 * i + 2
        }

        /**
         * A touch event: its masked [action], the pointer [id] at its action index and that
         * pointer's first [count] floats of [points] (pixels), from a touch that began on the key
         * with [initialKeyCode]. Whether it is part of a glide.
         */
        internal fun touch(action: Int, id: Int, points: FloatArray, count: Int, density: Float, initialKeyCode: Int?): Boolean {
            val now = clock()
            val outcome = detector.touch(action, id, points, count, now, density, initialKeyCode)
            val data = pointerData
            if (outcome.clearedBefore) resetState()
            if (outcome.took) {
                for (i in 0 until count / 2) data.positions.add(Position(points[2 * i], points[2 * i + 1]))
                if (outcome.started) data.startTime = now
            }
            if (!outcome.clearedAfter) data.isActuallyGesture = outcome.decision
            if (outcome.emitFrom >= 0) {
                for (i in outcome.emitFrom until data.positions.size) {
                    val point = data.positions[i]
                    listeners.forEach { it.onGlideAddPoint(point) }
                }
            }
            if (outcome.complete) listeners.forEach { listener -> listener.onGlideComplete(data) }
            if (outcome.cancelled) listeners.forEach { it.onGlideCancelled() }
            if (outcome.clearedAfter) resetState()
            return outcome.gesture
        }

        fun registerListener(listener: Listener) {
            listeners.add(listener)
        }

        fun unregisterListener(listener: Listener) {
            listeners.remove(listener)
        }

        private fun resetState() {
            pointerData.apply {
                positions.clear()
                startTime = 0
                isActuallyGesture = null
            }
        }

        private companion object {
            /** Room for this many floats at first: a move's history is seldom longer. */
            const val POINTS = 32
        }

        data class PointerData(
            val positions: MutableList<Position>,
            var startTime: Long,
            var isActuallyGesture: Boolean? = null,
        )

        /**
         * A point of a gesture. AudioWhisper: with its [time] in microseconds after the touch went
         * down, where the renderer gives it (RendererGestureHost), and 0 where nothing does.
         */
        data class Position(val x: Float, val y: Float, val time: Int = 0) {
            fun dist(p2: Position): Float {
                return sqrt((p2.x - x).pow(2) + (p2.y - y).pow(2))
            }
        }
    }

    interface Listener {
        /**
         * Called when a gesture is complete.
         */
        fun onGlideComplete(data: Detector.PointerData) {}

        /**
         * Called when a point is added to a gesture.
         * Will not be called before a series of events is detected as a gesture.
         */
        fun onGlideAddPoint(point: Detector.Position) {}

        /**
         * Called to cancel a gesture.
         */
        fun onGlideCancelled() {}
    }
}
