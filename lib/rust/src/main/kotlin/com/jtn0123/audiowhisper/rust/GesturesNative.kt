package com.jtn0123.audiowhisper.rust

/**
 * The touch gestures' entry points in libawnative: lib/rust/crates/awnative/src/gestures.rs, which
 * documents each and how a swipe and a glide's outcome are packed. [SwipeDetector] and
 * [GlideDetector] wrap them.
 */
internal object GesturesNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun newSwipes(): Long

    @JvmStatic external fun freeSwipes(swipes: Long)

    @JvmStatic external fun swipesReset(swipes: Long)

    @JvmStatic external fun swipesDown(swipes: Long, id: Int, x: Float, y: Float, density: Float)

    @JvmStatic external fun swipesMove(
        swipes: Long,
        id: Int,
        x: Float,
        y: Float,
        density: Float,
        distanceThreshold: Int,
        always: Boolean,
        out: IntArray,
    ): Boolean

    @JvmStatic external fun swipesUp(
        swipes: Long,
        id: Int,
        x: Float,
        y: Float,
        velocityX: Float,
        velocityY: Float,
        density: Float,
        distanceThreshold: Int,
        velocityThreshold: Int,
        out: IntArray,
    ): Boolean

    @JvmStatic external fun swipesCancel(swipes: Long, id: Int)

    @JvmStatic external fun newGlideStart(keySize: Float): Long

    @JvmStatic external fun freeGlideStart(glide: Long)

    @JvmStatic external fun glideTouch(
        glide: Long,
        action: Int,
        id: Int,
        points: FloatArray,
        count: Int,
        now: Long,
        density: Float,
        hasKey: Boolean,
        keyCode: Int,
    ): Long
}
