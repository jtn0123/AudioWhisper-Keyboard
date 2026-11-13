package com.jtn0123.audiowhisper.rust

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Test-only StaggeredGridCells and shortest-lane reference from the Compose panel. */
class ClipboardPresentationMatchTest {
    private fun reference(c: FloatArray): FloatArray {
        val columns = if(c[2]==0f) (c[0]/(160*c[1])).toInt().coerceAtLeast(1) else c[2].toInt().coerceAtLeast(1)
        val lanes=FloatArray(columns)
        val out=mutableListOf(columns.toFloat())
        for(height in c.drop(3)) {
            val lane=lanes.indices.minBy { lanes[it] }
            out.add(lane.toFloat());out.add(lanes[lane]);lanes[lane]+=height
        }
        return out.toFloatArray()
    }
    @Test fun adaptiveFixedAndUnequalItemHeights() {
        val r=Random(7309)
        for(density in listOf(.75f,1f,2f,3f)) for(width in listOf(100f,160f,319f,320f,360f,1080f)) for(fixed in 0..5) {
            repeat(40) {
                val c=floatArrayOf(width,density,fixed.toFloat())+FloatArray(40){r.nextInt(1,500).toFloat()}
                assertContentEquals(reference(c),PanelRules.clipboardGrid(c))
            }
        }
    }
}
