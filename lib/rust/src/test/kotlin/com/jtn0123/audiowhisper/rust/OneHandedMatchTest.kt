package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Frozen FlorisImeUi Row weights and OneHandedPanel aspectRatio + weighted remainder. */
private object ReferenceOneHanded {
    fun weights(enabled: Boolean, landscape: Boolean, scale: Int, end: Boolean): FloatArray {
        val keyboard = if (!enabled || landscape) 1f else scale / 100f
        return floatArrayOf(keyboard, if (enabled && !landscape && end) 1f - keyboard else 0f,
            if (enabled && !landscape && !end) 1f - keyboard else 0f)
    }
    fun buttons(width: Float, height: Float): FloatArray {
        val close = minOf(width, height)
        return floatArrayOf(0f,0f,width,close,0f,close,width,height)
    }
}
class OneHandedMatchTest {
    @Test fun portraitLandscapePhysicalSidesAndScale() {
        for (enabled in 0..1) for (landscape in 0..1) for (scale in 50..100) for (end in 0..1) {
            assertContentEquals(ReferenceOneHanded.weights(enabled != 0, landscape != 0, scale, end != 0),
                ImeChromeRules.oneHanded(intArrayOf(enabled, landscape, scale, end)))
        }
    }
    @Test fun squareCloseAndWeightedMoveTargets() {
        for (density in listOf(.75f,1f,2f,2.625f,3f)) for (width in listOf(0f,24f,72f,120f)) for (height in listOf(20f,100f,280f)) {
            assertContentEquals(ReferenceOneHanded.buttons(width*density,height*density),
                ImeChromeRules.sideButtons(floatArrayOf(width*density,height*density)))
        }
    }
}
