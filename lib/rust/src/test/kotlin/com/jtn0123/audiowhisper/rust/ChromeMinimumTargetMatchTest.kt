package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Frozen Compose minimumInteractiveComponentSize, with the next subtype row's clip. */
private fun referenceTarget(density: Float, width: Float, top: Float, height: Float, next: Float, header: Boolean): FloatArray {
    val center = top + height / 2
    val half = maxOf(height, 48f * density) / 2
    return floatArrayOf(0f, center-half, width, if (header) minOf(next,center+half) else center+half)
}
class ChromeMinimumTargetMatchTest {
    @Test fun disabledHeaderAndExtractActionMatchComposeMinimumTargets() {
        for (density in listOf(1f,2f,2.625f,3f)) for (height in listOf(16f,36f,48f,80f)) for (header in listOf(true,false)) {
            val width=400f*density;val top=100f*density;val h=height*density;val next=top+h
            assertContentEquals(referenceTarget(density,width,top,h,next,header),
                ImeChromeRules.minimumTarget(floatArrayOf(density,width,top,h,next,if(header) 0f else 1f)))
        }
    }
}
