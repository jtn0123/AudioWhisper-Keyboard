package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Frozen one-line StartEllipsis contract for measured fixed-advance ASCII glyphs. */
class DictationPreviewMatchTest {
    private fun reference(text: String, width: Float, advance: Float): String {
        if (text.length*advance <= width) return text
        if (advance > width) return ""
        return "…" + text.takeLast(((width/advance).toInt()-1).coerceAtLeast(0))
    }
    @Test fun newestWordsRemainVisibleAndNarrowViewportsStayClipped() {
        val labels=arrayOf("", "a", "aw fixture dictation preview", "the newest words stay visible")
        for (advance in listOf(1f,3f,10f)) for (width in 0..100) {
            assertContentEquals(labels.map { reference(it,width.toFloat(),advance) }.toTypedArray(),
                ImeChromeRules.startEllipsis(labels,floatArrayOf(width.toFloat(),advance)))
        }
    }
}
