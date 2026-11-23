package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Smartbar.kt's, QuickActionsRow.kt's and CandidatesRow.kt's rows and AudioWhisper's word fit, as
 * their frozen transcription answered: golden digests now, exact, as every layout field matched it to
 * the bit when they were written (docs/phase6-kotlin-references.md).
 */
class SmartbarMatchTest {
    @Test fun settingsAndSeededCandidateListsMatchComposeRules() {
        val random = Random(0x3b02)
        val samples = listOf("", "hi", "a very long candidate that must be ellipsized", "👨‍👩‍👧‍👦", "🇺🇸", "👍🏽", "1️⃣", "é")
        var cases = 0
        val tape = GoldenTape("smartbar-layouts")
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
                    tape.answer("layout", SmartbarRules.layout(config,width,widths).toList())
                    // Font-independent ellipsis selection, including empty and emoji prefixes.
                    val advances = FloatArray(words.size) { (it+1)*7*density+9*density }
                    val available = random.nextFloat()*300*density
                    tape.answer("ellipsis", SmartbarRules.ellipsis(advances,available))
                    cases++
                }
            }
        tape.finish()
        assertEquals(9216,cases)
    }

    @Test fun inlineReservationMatchesComposeForEveryMainRowAndPreviewState() {
        val tape = GoldenTape("smartbar-inline")
        for (enabled in 0..1) for (layout in 0..3) for (flip in 0..1) for (shared in 0..1)
            for (extended in 0..1) for (placement in 0..2) for (speaker in 0..1)
                for (sticky in 0..1) for (inline in 0..1) for (preview in 0..1) {
                    val config = floatArrayOf(enabled.toFloat(), layout.toFloat(), flip.toFloat(),
                        shared.toFloat(), extended.toFloat(), placement.toFloat(), speaker.toFloat(),
                        sticky.toFloat(), 19f, 0f, 2.625f, 113f, inline.toFloat(), preview.toFloat())
                    for (width in listOf(720f, 1080f, 1440f)) {
                        val candidates = floatArrayOf(70f, 200f, 500f)
                        tape.answer("layout", SmartbarRules.layout(config, width, candidates).toList())
                    }
                }
        tape.finish()
    }

    @Test fun wordsShowInGboardsRowForEveryArrangementAndException() {
        var roomy = 0
        var cases = 0
        val tape = GoldenTape("smartbar-roomy")
        for (enabled in 0..1) for (layout in 0..3) for (flip in 0..1) for (shared in 0..1)
            for (extended in 0..1) for (placement in 0..2) for (speaker in 0..1) for (sticky in 0..1)
                for (inline in 0..1) for (preview in 0..1) for (words in 0..1) for (dictating in 0..1)
                    for (reading in 0..1) {
                        val config = floatArrayOf(enabled.toFloat(), layout.toFloat(), flip.toFloat(),
                            shared.toFloat(), extended.toFloat(), placement.toFloat(), speaker.toFloat(),
                            sticky.toFloat(), 19f, 0f, 3.75f, 150f, inline.toFloat(), preview.toFloat(),
                            words.toFloat(), dictating.toFloat(), reading.toFloat())
                        tape.answer("roomy", SmartbarRules.roomy(config))
                        tape.answer("toggle", SmartbarRules.toggleWidth(config))
                        if (SmartbarRules.roomy(config)) roomy++
                        for (width in listOf(1080f, 1440f)) {
                            val candidates = floatArrayOf(160f, 340f, 490f)
                            tape.answer("layout", SmartbarRules.layout(config, width, candidates).toList())
                        }
                        cases++
                    }
        tape.finish()
        assertEquals(24576, cases)
        // Enabled with words and no inline chips, preview or dictation: flip, extended, placement and
        // sticky are free (24); shared collapsed or extended either way (3); speaker and reading not both (3).
        assertEquals(24 * 3 * 3, roomy)
    }

    @Test fun wordsSqueezeThenShrinkStepForStep() {
        val random = Random(0x3b05)
        val tape = GoldenTape("smartbar-fit")
        repeat(20_000) {
            val squeeze = listOf(1f, 0.9f, 0.7f, 0.7f, 0.83f, random.nextFloat() * 0.3f + 0.7f)[it % 6]
            val scale = listOf(1f, 1f, 0.9f, 0.75f, 0.6f, random.nextFloat() + 0.01f)[it % 6]
            val available = random.nextFloat() * 600f
            val measured = random.nextFloat() * 900f
            val fit = SmartbarRules.fitStep(TextFit(squeeze, scale), measured, available)
            tape.answer("step", listOf(fit.squeeze, fit.scale))
        }
        tape.finish()
        // The fitting loop: squeeze alone while it can, one measure; then a smaller font.
        var measures = 0
        assertEquals(TextFit(0.8f, 1f), SmartbarRules.fitText(100f) { measures++; 125f * it })
        assertEquals(1, measures)
        assertEquals(TextFit(0.7f, 0.95f), SmartbarRules.fitText(100f) { 150f * it })
        assertEquals(TextFit(0.7f, 0.91f), SmartbarRules.fitText(100f) { 150f * it + 5f })
        assertEquals(TextFit(0.7f, 0.75f), SmartbarRules.fitText(100f) { 400f * it }, "both floors: ellipsis")
        assertEquals(TextFit.NATURAL, SmartbarRules.fitText(100f) { 90f * it })
        measures = 0
        SmartbarRules.fitText(100f) { measures++; 1000f * kotlin.math.sqrt(it) }
        assertTrue(measures <= SmartbarRules.TEXT_FIT_STEPS)
    }
}
