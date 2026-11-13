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
