package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Frozen extract SnyggRow: weighted field, 8dp action margins, vertically centered action. */
class ExtractChromeMatchTest {
    private fun reference(c: FloatArray): FloatArray {
        val width = minOf(c[3], maxOf(0f, c[0] - 16f * c[2])); val height = minOf(c[4], c[1])
        return floatArrayOf(0f,0f,maxOf(0f,c[0]-width-16f*c[2]),c[1],
            c[0]-width-8f*c[2],(c[1]-height)/2,c[0]-8f*c[2],(c[1]+height)/2)
    }
    @Test fun naturalActionAndWeightedExtractField() {
        for (density in listOf(.75f,1f,2f,2.625f,3f)) for (width in listOf(160f,360f,800f))
            for (height in listOf(80f,160f,300f)) for (button in listOf(40f,80f,200f)) {
                val c = floatArrayOf(width*density,height*density,density,button*density,36f*density)
                assertContentEquals(reference(c), ImeChromeRules.extract(c))
            }
    }
}
