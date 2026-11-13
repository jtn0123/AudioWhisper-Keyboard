package com.jtn0123.audiowhisper.rust

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SmartbarMatchTest {
    @Test fun settingsAndSeededCandidateListsMatchComposeRules() {
        val random = Random(0x3b02)
        val samples = listOf("", "hi", "a very long candidate that must be ellipsized", "👨‍👩‍👧‍👦", "🇺🇸", "👍🏽", "1️⃣", "é")
        var cases = 0
        for (layout in 0..3) for (flip in 0..1) for (shared in 0..1) for (extended in 0..1)
            for (placement in 0..2) for (speaker in 0..1) for (sticky in 0..1) for (display in 0..2) {
                repeat(8) { n ->
                    val density = listOf(1f,2f,2.625f,3f)[n%4]
                    val config = floatArrayOf(if (n == 0) 0f else 1f,layout.toFloat(),flip.toFloat(),
                        shared.toFloat(),extended.toFloat(),placement.toFloat(),speaker.toFloat(),sticky.toFloat(),
                        listOf(0,1,2,3,6,19,30,50)[n].toFloat(),display.toFloat(),density,40*density)
                    val words = List(if (n == 1) 0 else if (n == 2) 1 else random.nextInt(2,25)) {
                        samples.random(random).repeat(random.nextInt(1,8))
                    }
                    // Independent deterministic text advances; geometry must agree for any measured font.
                    val widths = words.map { it.codePoints().count().toFloat()*7*density+24*density }.toFloatArray()
                    val width = listOf(1080f,720f,1440f,480f)[n%4]
                    val expected = ReferenceSmartbar.layout(config,width,widths)
                    val actual = SmartbarRules.layout(config,width,widths)
                    assertEquals(expected.size,actual.size,"packet size at case $cases")
                    expected.indices.forEach { i -> assertTrue(kotlin.math.abs(expected[i]-actual[i])<0.002f,"case $cases field $i expected ${expected[i]} actual ${actual[i]}") }
                    // Font-independent ellipsis selection, including empty and emoji prefixes.
                    val advances = FloatArray(words.size) { (it+1)*7*density+9*density }
                    val available = random.nextFloat()*300*density
                    assertEquals(ReferenceSmartbar.ellipsis(advances,available),SmartbarRules.ellipsis(advances,available))
                    cases++
                }
            }
        assertEquals(9216,cases)
    }
}
