package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

/** The reference's answers also go on golden tapes (docs/phase6-kotlin-references.md). */
class MediaPresentationMatchTest {
    @Test fun categoryAccessibilityScrollMatchesComposeViewportClamp() {
        for(offset in listOf(-100f,0f,100f,300f)) for(width in listOf(100f,360f)) for(extent in listOf(0f,100f,900f)) for(forward in listOf(false,true)) {
            val expected=(offset+if(forward)width else -width).coerceIn(0f,(extent-width).coerceAtLeast(0f))
            assertContentEquals(floatArrayOf(expected),MediaRules.scrollChips(floatArrayOf(offset,width,extent,if(forward)1f else 0f)))
        }
    }
    @Test fun staggeredGifGeometryIncludesAspectClamps() {
        val random=Random(7310)
        val tape=GoldenTape.ofReference("media-grids")
        for(density in listOf(.75f,1f,2f,3f)) for(width in listOf(200f,360f,1080f)) repeat(200) {
            val c=floatArrayOf(width,density)+FloatArray(30){random.nextInt(1,100)/10f}
            tape.answer("grid",referenceGifGrid(c).toList())
            assertContentEquals(referenceGifGrid(c),MediaRules.grid(c))
        }
        tape.finish()
    }
    @Test fun lruVictimIdentityIsObservable() {
        val c=intArrayOf(24,16)
        val ops=longArrayOf(0,1,0,100,1,1,1,0,0,2,0,100,1,1,1,0,0,3,0,100,1,1,1,0,
            1,2,0,0,1,1,0,0,0,4,0,100,1,1,1,0,1,1,0,0,1,1,0,0)
        GoldenTape.ofReference("media-lru").apply { answer("trace",referenceMedia(c,ops).toList()) }.finish()
        assertContentEquals(referenceMedia(c,ops),MediaRules.trace(c,ops))
    }
    @Test fun finiteAndInfiniteEncodedRepeatCounts() {
        val tape=GoldenTape.ofReference("media-repeats")
        for(repeat in listOf(-1,0,1,2,10)) {
            val ops=mutableListOf<Long>(0,1,0,100_000_000,1,1,0,0,0,1,1,200_000_000,1,1,1,0,
                4,1,repeat.toLong(),0,1,1,0,0,1,1,0,0,1,1,0,0)
            for(t in 1..50)ops.addAll(listOf(2,1,0,0,1,1,0,t*100_000_000L))
            ops.addAll(listOf(1,0,0,0,1,1,0,6_000_000_000L,1,1,0,0,1,1,0,7_000_000_000L))
            tape.answer("trace $repeat",referenceMedia(intArrayOf(1024,512),ops.toLongArray()).toList())
            assertContentEquals(referenceMedia(intArrayOf(1024,512),ops.toLongArray()),MediaRules.trace(intArrayOf(1024,512),ops.toLongArray()))
        }
        tape.finish()
    }
    @Test fun animationVisibilityDeadlinesAndBoundedLruAdmission() {
        val random=Random(7311)
        val tape=GoldenTape.ofReference("media-animations")
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
            tape.answer("trace $run",referenceMedia(c,ops.toLongArray()).toList())
            assertContentEquals(referenceMedia(c,ops.toLongArray()),MediaRules.trace(c,ops.toLongArray()),"run=$run")
        }
        tape.finish()
    }
}
