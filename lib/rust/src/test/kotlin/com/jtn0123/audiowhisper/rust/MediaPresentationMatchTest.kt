package com.jtn0123.audiowhisper.rust

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

class MediaPresentationMatchTest {
    @Test fun staggeredGifGeometryIncludesAspectClamps() {
        val random=Random(7310)
        for(density in listOf(.75f,1f,2f,3f)) for(width in listOf(200f,360f,1080f)) repeat(200) {
            val c=floatArrayOf(width,density)+FloatArray(30){random.nextInt(1,100)/10f}
            assertContentEquals(referenceGifGrid(c),MediaRules.grid(c))
        }
    }
    @Test fun animationVisibilityDeadlinesAndBoundedLruAdmission() {
        val random=Random(7311)
        repeat(400) { run ->
            val c=intArrayOf(listOf(64,256,1024,4096)[run%4],listOf(32,128,512)[run%3])
            val ops=mutableListOf<Long>()
            var time=0L
            repeat(200) {
                time+=random.nextLong(1,200_000_000)
                ops.addAll(listOf(random.nextLong(0,3),random.nextLong(0,6),random.nextLong(0,66),
                    random.nextLong(0,300_000_000),random.nextLong(1,9),random.nextLong(1,9),random.nextLong(2),time))
            }
            // Guaranteed multi-frame playback, hidden pause and visible resume, including exact boundary.
            ops.addAll(listOf(0,9,0,100_000_000,1,1,0,time,0,9,1,200_000_000,1,1,1,time,
                1,9,0,0,1,1,0,time,2,9,0,0,1,1,0,time+100_000_000,1,0,0,0,1,1,0,time+200_000_000))
            assertContentEquals(referenceMedia(c,ops.toLongArray()),MediaRules.trace(c,ops.toLongArray()),"run=$run")
        }
    }
}
