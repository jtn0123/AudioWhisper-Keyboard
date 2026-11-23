package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The clipboard history's plan and MIME rules, as FlorisBoard's ClipboardManager and ClipboardHistory
 * decided them: what its frozen copy answered is a golden digest now (docs/phase6-kotlin-references.md).
 */
class ClipboardMatchTest {
    @Test fun orderingGroupsDuplicatesExpiryLimitsAndPins() {
        val random = Random(7308)
        val extremes = listOf(Long.MIN_VALUE, -1L, 0L, 299_999L, 300_000L, 300_001L, Long.MAX_VALUE)
        val durations = listOf(Int.MIN_VALUE, -1, 0, 1, 5, Int.MAX_VALUE)
        val tape = GoldenTape("clipboard-plans")
        repeat(12_000) { step ->
            val count = random.nextInt(0, 30)
            val now = if (step % 2 == 0) extremes.random(random) else random.nextLong()
            val records = (0 until count).flatMap {
                listOf(if (it % 2 == 0) extremes.random(random) else now - random.nextLong(400_000),
                    random.nextLong(1,4), random.nextLong(2), random.nextLong(2), random.nextLong(2))
            }.toLongArray()
            val texts = Array(count + 1) { listOf("", "a", "aw-test-1", "\uD800", "\uDFFF", "📋").random(random) }
            val config = longArrayOf(now, (step and 1).toLong(), (step shr 1 and 1).toLong(), durations.random(random).toLong(),
                (step shr 2 and 1).toLong(), durations.random(random).toLong(), (step shr 3 and 1).toLong(),
                durations.random(random).toLong(), (step shr 4 and 1).toLong(),(step shr 5 and 1).toLong())
            tape.answer("plan", ClipboardRules.plan(records, texts, config).map { it.toList() })
        }
        tape.finish()
    }
    @Test fun mimeEligibilityAndPatterns() {
        val types = listOf("", "text/plain", "text/html", "image/gif", "image/png", "video/mp4", "*/*", "image/*", "IMAGE/*", "/x", "a/", "a/*", "\uD800/a", "a/b/c")
        val tape = GoldenTape("clipboard-mimes")
        for (concrete in types) for (desired in types) {
            tape.answer("matches", ClipboardRules.mimeMatches(concrete,desired))
            for (extra in types) tape.answer("eligible", ClipboardRules.eligible(arrayOf(concrete,extra),arrayOf(desired)))
        }
        tape.finish()
        assertEquals(false, ClipboardRules.eligible(emptyArray(),arrayOf("*/*")))
        assertEquals(true, ClipboardRules.eligible(arrayOf("text/plain"),emptyArray()))
    }
}
