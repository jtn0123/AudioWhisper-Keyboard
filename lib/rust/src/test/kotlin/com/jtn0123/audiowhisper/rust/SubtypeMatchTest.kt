package com.jtn0123.audiowhisper.rust

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

/** SelectSubtypePanel's frozen items(subtypes), displayName and currentlySelected == id rule.
 * Android Locale supplies localized display names; Rust must preserve them without sorting.
 */
private object ReferenceSubtypePanel {
    fun rows(selected: Long, ids: LongArray, labels: Array<String>): Array<String> =
        ids.indices.flatMap { listOf(ids[it].toString(), labels[it], (selected == ids[it]).toString()) }.toTypedArray()
}
class SubtypeMatchTest {
    /** Frozen bottom-aligned sheet Column plus the unchanged bottom-aligned input-view spacer. */
    @Test fun sheetAndUnderlyingKeyboardRemainBottomAligned() {
        for (height in listOf(100f,400f,800f)) for (header in listOf(40f,48f,60f))
            for (row in listOf(48f,56f,72f)) for (count in 0..20) for (inset in listOf(0f,24f,48f)) {
                val c=floatArrayOf(height,header,row,count.toFloat(),48f,200f,inset)
                val expected=floatArrayOf(maxOf(0f,height-inset-header-row*count),maxOf(0f,height-48f-200f))
                assertContentEquals(expected,SubtypeRules.geometry(c))
            }
    }
    @Test fun configuredOrderLocalizedNamesAndSelectedIdentity() {
        val random = Random(80)
        val names = listOf("English (United States)", "English (United Kingdom)", "español", "Tiếng Việt", "français", "😀", "")
        repeat(2000) {
            val ids = LongArray(random.nextInt(0, 50)) { random.nextLong() }
            val labels = Array(ids.size) { names.random(random) }
            val selected = ids.randomOrNull(random) ?: Long.MIN_VALUE
            assertContentEquals(ReferenceSubtypePanel.rows(selected, ids, labels), SubtypeRules.rows(selected, ids, labels))
            assertContentEquals(ReferenceSubtypePanel.rows(Long.MAX_VALUE, ids, labels), SubtypeRules.rows(Long.MAX_VALUE, ids, labels))
        }
    }
}
