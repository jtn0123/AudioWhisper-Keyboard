package dev.patrickgold.florisboard.ime.text.gestures

import dev.patrickgold.florisboard.ime.text.gestures.GlideTypingGesture.Detector.PointerData
import dev.patrickgold.florisboard.ime.text.gestures.GlideTypingGesture.Detector.Position
import dev.patrickgold.florisboard.ime.text.gestures.GlideTypingGesture.Listener
import dev.patrickgold.florisboard.ime.text.key.KeyCode

/**
 * The parts of a `MotionEvent` that [ReferenceGlideDetector] reads: its masked action, its action
 * index, and each pointer's id and positions (its history, oldest first, then where it is now).
 */
internal class ReferenceMotionEvent(
    val actionMasked: Int,
    val actionIndex: Int,
    private val ids: IntArray,
    private val xs: List<FloatArray>,
    private val ys: List<FloatArray>,
) {
    val historySize: Int get() = xs[0].size - 1

    fun getPointerId(index: Int) = ids[index]

    fun findPointerIndex(id: Int) = ids.indexOf(id)

    fun getX(index: Int) = xs[index][historySize]

    fun getY(index: Int) = ys[index][historySize]

    fun getHistoricalX(index: Int, pos: Int) = xs[index][pos]

    fun getHistoricalY(index: Int, pos: Int) = ys[index][pos]

    companion object {
        const val ACTION_DOWN = 0
        const val ACTION_UP = 1
        const val ACTION_MOVE = 2
        const val ACTION_CANCEL = 3
        const val ACTION_POINTER_DOWN = 5
        const val ACTION_POINTER_UP = 6
    }
}

/**
 * The Kotlin `GlideTypingGesture.Detector` replaced by lib/rust/crates/gestures/src/glide_start.rs,
 * kept to test against. Edited only to compile without Android: it reads a [ReferenceMotionEvent],
 * takes the key's size in dp and the initial key's code rather than a `Context` and a `TextKey`,
 * `ViewUtils.px2dp` is `px / density`, and `System.currentTimeMillis` is [clock]. Its
 * [pointerData] is visible, for the comparison to see what it decided.
 */
internal class ReferenceGlideDetector(private val keySize: Float, private val clock: () -> Long) {
    val pointerData: PointerData = PointerData(mutableListOf(), 0)
    private val listeners: ArrayList<Listener> = arrayListOf()
    private var pointerId: Int = -1

    companion object {
        private const val MAX_DETECT_TIME = 500
        private const val VELOCITY_THRESHOLD = 0.10 // dp per ms
        private val SWIPE_GESTURE_KEYS = arrayOf(KeyCode.DELETE, KeyCode.SHIFT, KeyCode.SPACE, KeyCode.CJK_SPACE)
    }

    private fun px2dp(px: Float, density: Float) = px / density

    /**
     * Method which evaluates if a given [event] is a gesture.
     *
     * @return whether or not the event was interpreted as part of a gesture.
     */
    fun onTouchEvent(event: ReferenceMotionEvent, initialKeyCode: Int?, density: Float): Boolean {
        when (event.actionMasked) {
            ReferenceMotionEvent.ACTION_DOWN,
            ReferenceMotionEvent.ACTION_POINTER_DOWN -> {
                if (event.actionMasked == ReferenceMotionEvent.ACTION_DOWN) {
                    resetState()
                }
                if (pointerId != -1) {
                    // if we already have another pointer, we don't care
                    return false
                }
                val pointerIndex = event.actionIndex
                pointerId = event.getPointerId(pointerIndex)
                pointerData.apply {
                    positions.add(Position(event.getX(pointerIndex), event.getY(pointerIndex)))
                    startTime = clock()
                }
                return false
            }
            ReferenceMotionEvent.ACTION_MOVE -> {
                if (pointerId != event.getPointerId(event.actionIndex)) {
                    // not our pointer.
                    return false
                }

                val pointerIndex = event.findPointerIndex(pointerId)
                for (i in 0..event.historySize) {
                    val pos = when (i) {
                        event.historySize -> Position(event.getX(pointerIndex), event.getY(pointerIndex))
                        else -> Position(event.getHistoricalX(pointerIndex, i), event.getHistoricalY(pointerIndex, i))
                    }
                    pointerData.positions.add(pos)
                    if (pointerData.isActuallyGesture == null) {
                        // evaluate whether is actually a gesture
                        val dist = px2dp(pointerData.positions[0].dist(pos), density)
                        val time = (clock() - pointerData.startTime) + 1
                        if (dist > keySize && (dist / time) > VELOCITY_THRESHOLD && (initialKeyCode !in SWIPE_GESTURE_KEYS)) {
                            pointerData.isActuallyGesture = true
                            // Let listener know all those points need to be added.
                            pointerData.positions.take(pointerData.positions.size - 1).forEach { point ->
                                listeners.forEach {
                                    it.onGlideAddPoint(point)
                                }
                            }
                        } else if (time > MAX_DETECT_TIME) {
                            pointerData.isActuallyGesture = false
                        }

                    }

                    if (pointerData.isActuallyGesture == true) {
                        pointerData.positions.last()
                            .let { point -> listeners.forEach { it.onGlideAddPoint(point) } }
                    }
                }
                return pointerData.isActuallyGesture ?: false
            }
            ReferenceMotionEvent.ACTION_UP,
            ReferenceMotionEvent.ACTION_POINTER_UP -> {
                if (pointerId != event.getPointerId(event.actionIndex)) {
                    // not our pointer.
                    return false
                }
                if (pointerData.isActuallyGesture == true) {
                    listeners.forEach { listener -> listener.onGlideComplete(pointerData) }
                }
                resetState()
                return false
            }
            ReferenceMotionEvent.ACTION_CANCEL -> {
                if (pointerData.isActuallyGesture == true) {
                    listeners.forEach { it.onGlideCancelled() }
                }
                resetState()
            }
            else -> return false
        }
        return false
    }

    fun registerListener(listener: Listener) {
        listeners.add(listener)
    }

    private fun resetState() {
        pointerData.apply {
            positions.clear()
            startTime = 0
            isActuallyGesture = null
        }
        pointerId = -1
    }
}
