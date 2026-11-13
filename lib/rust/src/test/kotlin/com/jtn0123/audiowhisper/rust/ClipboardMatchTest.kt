package com.jtn0123.audiowhisper.rust

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals

class ClipboardMatchTest {
    @Test fun orderingGroupsDuplicatesExpiryLimitsAndPins() {
        val random = Random(7308)
        val extremes = listOf(Long.MIN_VALUE, -1L, 0L, 299_999L, 300_000L, 300_001L, Long.MAX_VALUE)
        val durations = listOf(Int.MIN_VALUE, -1, 0, 1, 5, Int.MAX_VALUE)
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
                durations.random(random).toLong(), (step shr 4 and 1).toLong())
            val expected = referenceClipboard(records, texts, config)
            val actual = ClipboardRules.plan(records, texts, config)
            for (i in expected.indices) assertContentEquals(expected[i], actual[i], "step=$step plan=$i")
        }
    }
    @Test fun mimeEligibilityAndPatterns() {
        val types = listOf("", "text/plain", "text/html", "image/gif", "image/png", "video/mp4", "*/*", "image/*", "IMAGE/*", "/x", "a/", "a/*", "\uD800/a", "a/b/c")
        for (concrete in types) for (desired in types) {
            assertEquals(referenceMime(concrete,desired), ClipboardRules.mimeMatches(concrete,desired))
            for (extra in types) {
                val input = arrayOf(concrete,extra)
                val editor = arrayOf(desired)
                val expected = "text/plain" in input || editor.any { e -> input.any { referenceMime(it,e) } }
                assertEquals(expected, ClipboardRules.eligible(input,editor))
            }
        }
        assertEquals(false, ClipboardRules.eligible(emptyArray(),arrayOf("*/*")))
        assertEquals(true, ClipboardRules.eligible(arrayOf("text/plain"),emptyArray()))
    }
}
