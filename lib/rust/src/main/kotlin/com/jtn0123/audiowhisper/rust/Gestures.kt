package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/** The direction of a swipe: eighths of the circle, clockwise from up-left. */
enum class SwipeDirection { UP_LEFT, UP, UP_RIGHT, RIGHT, DOWN_RIGHT, DOWN, DOWN_LEFT, LEFT }

/**
 * A swipe: its [direction], and how many units (a quarter of the swipe distance each) the finger
 * moved since it went down ([absX], [absY]) and since the last swipe reported ([relX], [relY]).
 */
data class Swipe(val direction: SwipeDirection, val absX: Int, val absY: Int, val relX: Int, val relY: Int)

/**
 * Finds swipes, in Rust (lib/rust/crates/gestures/src/swipe.rs): up to four fingers at once, each
 * from where it went down. Positions and velocities are in pixels, with the screen's density;
 * the swipe distance is in dp and the velocity in dp a second, as the gesture preferences hold
 * them. Not for more than one thread at a time. Its native memory is freed once nothing can
 * reach it.
 */
class SwipeDetector {
    private val handle = GesturesNative.newSwipes()
    private val out = IntArray(SWIPE_INTS)

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** A new touch began: forget every finger. */
    fun reset() {
        GesturesNative.swipesReset(handle)
        Reference.reachabilityFence(this)
    }

    /** Finger [id] went down at ([x], [y]). A fifth finger is ignored. */
    fun down(id: Int, x: Float, y: Float, density: Float) {
        GesturesNative.swipesDown(handle, id, x, y, density)
        Reference.reachabilityFence(this)
    }

    /**
     * Finger [id] moved to ([x], [y]): a swipe once it has moved more than half [distanceThreshold]
     * along either axis since the last one, or at every move if [always].
     */
    fun move(id: Int, x: Float, y: Float, density: Float, distanceThreshold: Int, always: Boolean): Swipe? {
        val found = GesturesNative.swipesMove(handle, id, x, y, density, distanceThreshold, always, out)
        Reference.reachabilityFence(this)
        return if (found) swipe() else null
    }

    /**
     * Finger [id] was lifted at ([x], [y]), moving at ([velocityX], [velocityY]): a swipe if it went
     * further than [distanceThreshold] and faster than [velocityThreshold] along either axis.
     */
    fun up(
        id: Int,
        x: Float,
        y: Float,
        velocityX: Float,
        velocityY: Float,
        density: Float,
        distanceThreshold: Int,
        velocityThreshold: Int,
    ): Swipe? {
        val found = GesturesNative.swipesUp(
            handle, id, x, y, velocityX, velocityY, density, distanceThreshold, velocityThreshold, out,
        )
        Reference.reachabilityFence(this)
        return if (found) swipe() else null
    }

    /** Finger [id]'s touch was cancelled: forget it. */
    fun cancel(id: Int) {
        GesturesNative.swipesCancel(handle, id)
        Reference.reachabilityFence(this)
    }

    private fun swipe() = Swipe(SwipeDirection.entries[out[0]], out[1], out[2], out[3], out[4])

    private class Free(private val handle: Long) : Runnable {
        override fun run() = GesturesNative.freeSwipes(handle)
    }

    private companion object {
        const val SWIPE_INTS = 5
    }
}

/**
 * What a touch event made of a glide, in the order to act on it: clear the points kept if
 * [clearedBefore], add the event's if [took] (and start the clock if [started]), hand the kept
 * points from [emitFrom] on to the listeners, then report the glide [complete] or [cancelled], and
 * clear the points if [clearedAfter]. [gesture] is whether the event is part of a glide, and
 * [decision] whether the touch is one, once decided.
 */
@JvmInline
value class GlideOutcome(private val bits: Long) {
    val gesture: Boolean get() = flag(0)
    val clearedBefore: Boolean get() = flag(1)
    val took: Boolean get() = flag(2)
    val started: Boolean get() = flag(3)
    val complete: Boolean get() = flag(4)
    val cancelled: Boolean get() = flag(5)
    val clearedAfter: Boolean get() = flag(6)

    val decision: Boolean?
        get() = when ((bits shr 8).toInt() and 3) {
            1 -> true
            2 -> false
            else -> null
        }

    /** The first kept point to hand on, or -1 for none. */
    val emitFrom: Int get() = (bits ushr 32).toInt() - 1

    private fun flag(bit: Int) = (bits shr bit) and 1L == 1L
}

/**
 * Decides whether a touch is the start of a glide across keys [keySize] dp wide, in Rust
 * (lib/rust/crates/gestures/src/glide_start.rs). Not for more than one thread at a time. Its
 * native memory is freed once nothing can reach it.
 */
class GlideDetector(keySize: Float) {
    private val handle = GesturesNative.newGlideStart(keySize)

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /**
     * A touch event at [now] milliseconds: its masked [action], the pointer [id] at its action
     * index, and that pointer's first [count] floats of [points] (x, y, … in pixels): where it went
     * down, or its history then where it is now. The touch began on the key with [initialKeyCode],
     * if any.
     */
    fun touch(
        action: Int,
        id: Int,
        points: FloatArray,
        count: Int,
        now: Long,
        density: Float,
        initialKeyCode: Int?,
    ): GlideOutcome {
        val bits = GesturesNative.glideTouch(
            handle, action, id, points, count, now, density, initialKeyCode != null, initialKeyCode ?: 0,
        )
        Reference.reachabilityFence(this)
        return GlideOutcome(bits)
    }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = GesturesNative.freeGlideStart(handle)
    }
}
