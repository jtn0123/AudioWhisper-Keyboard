package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test

class GlideTrailMatchTest {
    @Test fun recordedAndSeededGeometryAgeAndAcceleratedFadeMatchCompose() {
        val random=Random(0x3b0303)
        // As Compose's drawGlideTrail drew them: a golden digest of its frozen copy's answers
        // (docs/phase6-kotlin-references.md).
        val tape=GoldenTape("glide-trails")
        repeat(2400) {case ->
            val count=case%65
            var time=1_000_000_000L
            val times=LongArray(count) {time+=random.nextInt(0,41)*1_000_000;time}
            var x=100f;var y=100f
            val points=FloatArray(count*2) {i ->if(i%2==0) {x+=random.nextInt(-40,41);x} else {y+=random.nextInt(-40,41);y}}
            val duration=listOf(0L,1L,50L,200L,500L)[case%5]*1_000_000
            val finish=if(case%2==0) time else -1L
            val now=time+listOf(0L,1L,49L,100L,199L,200L,201L,500L)[case%8]*1_000_000
            val strip=listOf(0f,40f,112.5f)[case%3]
            tape.answer("circles",GlideTrailRules.circles(times,points,finish,now,duration,strip).toList())
        }
        tape.finish()
    }
}
