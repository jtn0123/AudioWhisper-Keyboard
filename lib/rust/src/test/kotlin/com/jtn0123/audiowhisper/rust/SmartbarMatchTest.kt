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
                    val density = listOf(1f,2f,2.625f,3f,1.5f,2.5f,2.625f,3.5f)[n]
                    val config = floatArrayOf(if (n == 0) 0f else 1f,layout.toFloat(),flip.toFloat(),
                        shared.toFloat(),extended.toFloat(),placement.toFloat(),speaker.toFloat(),sticky.toFloat(),
                        listOf(0,1,2,3,6,19,30,50)[n].toFloat(),display.toFloat(),density,if (n%4 == 2) 43.028f*density else 40*density)
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

    @Test fun inlineReservationMatchesComposeForEveryMainRowAndPreviewState() {
        for (enabled in 0..1) for (layout in 0..3) for (flip in 0..1) for (shared in 0..1)
            for (extended in 0..1) for (placement in 0..2) for (speaker in 0..1)
                for (sticky in 0..1) for (inline in 0..1) for (preview in 0..1) {
                    val config = floatArrayOf(enabled.toFloat(), layout.toFloat(), flip.toFloat(),
                        shared.toFloat(), extended.toFloat(), placement.toFloat(), speaker.toFloat(),
                        sticky.toFloat(), 19f, 0f, 2.625f, 113f, inline.toFloat(), preview.toFloat())
                    for (width in listOf(720f, 1080f, 1440f)) {
                        val candidates = floatArrayOf(70f, 200f, 500f)
                        val expected = ReferenceSmartbar.layout(config, width, candidates)
                        val actual = SmartbarRules.layout(config, width, candidates)
                        assertEquals(expected.toList(), actual.toList(), config.toList().toString())
                    }
                }
    }

    @Test fun wordsFillTheMainRowForEveryArrangementAndException() {
        var roomy = 0
        var cases = 0
        for (enabled in 0..1) for (layout in 0..3) for (flip in 0..1) for (shared in 0..1)
            for (extended in 0..1) for (placement in 0..2) for (speaker in 0..1) for (sticky in 0..1)
                for (inline in 0..1) for (preview in 0..1) for (words in 0..1) for (dictating in 0..1)
                    for (reading in 0..1) {
                        val config = floatArrayOf(enabled.toFloat(), layout.toFloat(), flip.toFloat(),
                            shared.toFloat(), extended.toFloat(), placement.toFloat(), speaker.toFloat(),
                            sticky.toFloat(), 19f, 0f, 3.75f, 150f, inline.toFloat(), preview.toFloat(),
                            words.toFloat(), dictating.toFloat(), reading.toFloat())
                        assertEquals(ReferenceSmartbar.roomy(config), SmartbarRules.roomy(config), config.toList().toString())
                        if (SmartbarRules.roomy(config)) roomy++
                        for (width in listOf(1080f, 1440f)) {
                            val candidates = floatArrayOf(160f, 340f, 490f)
                            val expected = ReferenceSmartbar.layout(config, width, candidates)
                            val actual = SmartbarRules.layout(config, width, candidates)
                            assertEquals(expected.toList(), actual.toList(), config.toList().toString())
                        }
                        cases++
                    }
        assertEquals(24576, cases)
        // Enabled with words and no inline chips, preview or dictation: flip, extended, placement and
        // sticky are free (24); shared collapsed or extended either way (3); speaker and reading not both (3).
        assertEquals(24 * 3 * 3, roomy)
    }
}
