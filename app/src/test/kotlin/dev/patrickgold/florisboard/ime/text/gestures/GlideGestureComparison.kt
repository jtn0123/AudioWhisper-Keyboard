package dev.patrickgold.florisboard.ime.text.gestures

import dev.patrickgold.florisboard.ime.text.gestures.GlideTypingGesture.Detector.PointerData
import dev.patrickgold.florisboard.ime.text.gestures.GlideTypingGesture.Detector.Position
import dev.patrickgold.florisboard.ime.text.gestures.ReferenceMotionEvent.Companion.ACTION_CANCEL
import dev.patrickgold.florisboard.ime.text.gestures.ReferenceMotionEvent.Companion.ACTION_DOWN
import dev.patrickgold.florisboard.ime.text.gestures.ReferenceMotionEvent.Companion.ACTION_MOVE
import dev.patrickgold.florisboard.ime.text.gestures.ReferenceMotionEvent.Companion.ACTION_POINTER_DOWN
import dev.patrickgold.florisboard.ime.text.gestures.ReferenceMotionEvent.Companion.ACTION_POINTER_UP
import dev.patrickgold.florisboard.ime.text.gestures.ReferenceMotionEvent.Companion.ACTION_UP
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin
import kotlin.random.Random

/**
 * Puts the glide detector in Rust ([GlideTypingGesture.Detector]) up against the Kotlin it
 * replaced, [ReferenceGlideDetector], wherever this runs: the JVM's tests and, copied in for a
 * check, the phone. Both hear the same events at the same clock, and two listeners on each record
 * every call. It records each difference and how often each outcome was seen.
 */
class GlideGestureComparison {
    val differences = mutableListOf<String>()
    val seen = mutableMapOf<String, Int>()

    private fun check(what: String, input: String, kotlin: Any?, rust: Any?) {
        if (kotlin != rust && differences.size < MAX_DIFFERENCES) differences += "$what($input): kotlin $kotlin, rust $rust"
    }

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    /** Every call a listener hears, with what a complete glide held when it was reported. */
    private class Recorder : GlideTypingGesture.Listener {
        val calls = mutableListOf<String>()

        override fun onGlideComplete(data: PointerData) {
            calls += "complete ${data.positions.toList()} from ${data.startTime}, ${data.isActuallyGesture}"
        }

        override fun onGlideAddPoint(point: Position) {
            calls += "add $point"
        }

        override fun onGlideCancelled() {
            calls += "cancelled"
        }
    }

    /** A finger on the screen: its id and where it is, in pixels. */
    private class Finger(val id: Int, var x: Float, var y: Float)

    /**
     * Random touches: fingers going down and lifting in any order, moves with history, around a
     * key's width and the glide's speed, from every kind of key, cancels, other actions, and a
     * clock that jumps, stands still, goes back and wraps.
     */
    fun glides(random: Random, runs: Int, steps: Int) = repeat(runs) { run ->
        var now = TIMES.random(random)
        val keySize = KEY_SIZES.random(random)
        val kotlin = ReferenceGlideDetector(keySize) { now }
        val rust = GlideTypingGesture.Detector(keySize) { now }
        val kotlinCalls = List(2) { Recorder().also { kotlin.registerListener(it) } }
        val rustCalls = List(2) { Recorder().also { rust.registerListener(it) } }
        var density = DENSITIES.random(random)
        var key = KEYS.random(random)
        val fingers = mutableListOf<Finger>()
        repeat(steps) { step ->
            now += GAPS.random(random).takeIf { it != 0L } ?: random.nextLong(-1_000, 1_000)
            if (random.nextInt(30) == 0) density = DENSITIES.random(random)
            if (random.nextInt(30) == 0) key = KEYS.random(random)
            val event = event(random, fingers, kotlin.pointerData, now, density, keySize)
            val input = "run $run step $step: action ${event.actionMasked} at $now, key $key, density $density"
            val before = kotlin.pointerData.positions.size to kotlin.pointerData.isActuallyGesture
            val kotlinAnswer = kotlin.onTouchEvent(event, key, density)
            val rustAnswer = touch(rust, event, key, density)
            check("touch", input, kotlinAnswer, rustAnswer)
            for (i in kotlinCalls.indices) {
                check("calls ${i + 1}", input, kotlinCalls[i].calls.size, rustCalls[i].calls.size)
                check("last call ${i + 1}", input, kotlinCalls[i].calls.lastOrNull(), rustCalls[i].calls.lastOrNull())
            }
            see(outcome(event, before, kotlin.pointerData, key))
        }
        for (i in kotlinCalls.indices) check("calls ${i + 1}", "run $run", kotlinCalls[i].calls, rustCalls[i].calls)
    }

    /** What the keyboard's wrapper hands its detector from a `MotionEvent`. */
    private fun touch(rust: GlideTypingGesture.Detector, event: ReferenceMotionEvent, key: Int?, density: Float): Boolean {
        val index = event.actionIndex
        val points = when (event.actionMasked) {
            ACTION_DOWN, ACTION_POINTER_DOWN -> floatArrayOf(event.getX(index), event.getY(index))
            ACTION_MOVE -> FloatArray(2 * event.historySize + 2) { i ->
                if (i % 2 == 0) event.getHistoricalX(index, i / 2) else event.getHistoricalY(index, i / 2)
            }
            else -> FloatArray(0)
        }
        return rust.touch(event.actionMasked, event.getPointerId(index), points, points.size, density, key)
    }

    private fun outcome(event: ReferenceMotionEvent, before: Pair<Int, Boolean?>, after: PointerData, key: Int?): String {
        val (count, decided) = before
        val grew = after.positions.size > count
        return when (event.actionMasked) {
            ACTION_DOWN -> "down: first"
            ACTION_POINTER_DOWN -> if (grew) "down: followed" else "down: ignored"
            ACTION_MOVE -> when {
                !grew -> "move: not followed"
                decided == null && after.isActuallyGesture == true -> "move: a glide"
                decided == null && after.isActuallyGesture == false -> "move: too slow"
                decided == null && key in SWIPE_KEYS -> "move: undecided from a swipe key"
                else -> "move: ${if (decided == null) "undecided" else "decided $decided"}"
            }
            ACTION_UP, ACTION_POINTER_UP -> when {
                after.positions.isNotEmpty() || count == 0 && decided == null -> "up: not followed"
                decided == true -> "up: glide complete"
                else -> "up: no glide"
            }
            ACTION_CANCEL -> if (decided == true) "cancel: glide" else "cancel: no glide"
            else -> "other"
        }
    }

    /** The next event of a touch, its [fingers] moved, added or lifted as it says. */
    private fun event(
        random: Random,
        fingers: MutableList<Finger>,
        followed: PointerData,
        now: Long,
        density: Float,
        keySize: Float,
    ): ReferenceMotionEvent {
        val action = when (random.nextInt(24)) {
            0, 1 -> ACTION_DOWN
            // At most six fingers, so there is always an id left for another.
            2, 3 -> if (fingers.size < 6) ACTION_POINTER_DOWN else ACTION_POINTER_UP
            4 -> ACTION_UP
            5 -> ACTION_POINTER_UP
            6 -> ACTION_CANCEL
            7 -> OTHER_ACTIONS.random(random)
            else -> ACTION_MOVE
        }
        if (action == ACTION_DOWN) fingers.clear()
        if (action == ACTION_DOWN || action == ACTION_POINTER_DOWN || fingers.isEmpty()) {
            val id = generateSequence { ID.random(random) }.first { id -> fingers.none { it.id == id } }
            val finger = Finger(id, random.nextFloat() * 1_200f, random.nextFloat() * 600f)
            fingers.add(random.nextInt(fingers.size + 1), finger)
            if (action == ACTION_DOWN || action == ACTION_POINTER_DOWN) {
                return snapshot(action, fingers.indexOf(finger), fingers, 0, random, followed, now, density, keySize)
            }
        }
        val index = if (action == ACTION_MOVE && random.nextInt(5) != 0) 0 else random.nextInt(fingers.size)
        val history = if (action == ACTION_MOVE) HISTORY.random(random) else 0
        val event = snapshot(action, index, fingers, history, random, followed, now, density, keySize)
        when (action) {
            ACTION_UP, ACTION_CANCEL -> fingers.clear()
            ACTION_POINTER_UP -> fingers.removeAt(index)
        }
        return event
    }

    /** An event with the fingers where they are, each moved through [history] points first if it moves. */
    private fun snapshot(
        action: Int,
        actionIndex: Int,
        fingers: List<Finger>,
        history: Int,
        random: Random,
        followed: PointerData,
        now: Long,
        density: Float,
        keySize: Float,
    ): ReferenceMotionEvent {
        val xs = fingers.map { FloatArray(history + 1) }
        val ys = fingers.map { FloatArray(history + 1) }
        for ((f, finger) in fingers.withIndex()) {
            for (h in 0..history) {
                if (action == ACTION_MOVE) step(random, finger, followed, now, density, keySize)
                xs[f][h] = finger.x
                ys[f][h] = finger.y
            }
        }
        return ReferenceMotionEvent(action, actionIndex, fingers.map { it.id }.toIntArray(), xs, ys)
    }

    /**
     * Moves [finger]: often to exactly a key's width from where the glide began, or exactly as far
     * as the glide's speed takes it by [now], a hair either side, and otherwise anywhere near.
     */
    private fun step(random: Random, finger: Finger, followed: PointerData, now: Long, density: Float, keySize: Float) {
        val first = followed.positions.firstOrNull()
        val angle = if (random.nextBoolean()) random.nextInt(4) * PI / 2 else random.nextDouble() * 2 * PI
        val reach = when (random.nextInt(8)) {
            0, 1 -> nudge(random, keySize)
            2, 3 -> nudge(random, (0.1 * (now - followed.startTime + 1)).toFloat())
            4 -> ODD.random(random)
            else -> null
        }
        if (first != null && reach != null) {
            finger.x = first.x + (reach * density * cos(angle)).toFloat()
            finger.y = first.y + (reach * density * sin(angle)).toFloat()
        } else {
            finger.x += (random.nextFloat() - 0.5f) * 4f * keySize * density
            finger.y += (random.nextFloat() - 0.5f) * 4f * keySize * density
        }
    }

    /** [value], or a few units of the last place either side of it. */
    private fun nudge(random: Random, value: Float): Float {
        var nudged = value
        repeat(random.nextInt(4)) { nudged = if (random.nextBoolean()) Math.nextUp(nudged) else Math.nextDown(nudged) }
        return nudged
    }

    private companion object {
        const val MAX_DIFFERENCES = 20
        val SWIPE_KEYS = listOf(KeyCode.DELETE, KeyCode.SHIFT, KeyCode.SPACE, KeyCode.CJK_SPACE)
        val KEYS = SWIPE_KEYS + listOf(null, null, null, 'a'.code, 'q'.code, KeyCode.UNSPECIFIED, KeyCode.ENTER, -1, 12_289)
        val KEY_SIZES = listOf(36f, 36f, 32f, 48f, 0f, 1f, -1f, Float.NaN)
        val DENSITIES = listOf(1f, 1f, 2f, 2f, 4f, 0.75f, 1.5f, 2.625f, 3f, 3.5f, 0f, Float.NaN)
        val ID = listOf(0, 0, 1, 1, 2, 3, 4, 7, -1)
        val HISTORY = listOf(0, 0, 0, 1, 1, 2, 3, 5, 9, 20)
        val OTHER_ACTIONS = listOf(4, 7, 8, 9, 10, 11, 12, -1, 255)
        val TIMES = listOf(0L, 1_000L, 1_731_000_000_000L, Long.MAX_VALUE - 300, Long.MAX_VALUE, Long.MIN_VALUE, -500L)
        val GAPS = listOf(0L, 1L, 1L, 2L, 5L, 8L, 16L, 16L, 17L, 50L, 100L, 250L, 499L, 500L, 501L, -1L, -2L, -499L, 5_000L)
        val ODD = listOf(Float.NaN, Float.POSITIVE_INFINITY, -0f, 1e30f, Float.MAX_VALUE, Float.MIN_VALUE)
    }
}
