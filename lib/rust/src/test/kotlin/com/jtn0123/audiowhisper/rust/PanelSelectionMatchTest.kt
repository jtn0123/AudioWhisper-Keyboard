package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Frozen KeyboardEditing.handleClipboardSelect and handleArrow manual-selection flag rules. */
private fun referenceSelection(c: IntArray): IntArray {
    var manual = c[0]; var start = c[1]; var end = c[2]; var collapse = -1
    val selected = c[3] >= 0 && c[4] >= 0 && c[3] != c[4]
    if (c[5] == -34) {
        if (selected) {
            collapse = if (manual != 0 && start != 0) c[3] else c[4]
            manual = 0
        } else manual = if (manual == 0) 1 else 0
    } else if (manual != 0 && !selected) {
        if (c[5] in listOf(-21, -23, -25, -27)) { start = 1; end = 0 }
        else { start = 0; end = 1 }
    }
    return intArrayOf(manual, start, end, collapse)
}

class PanelSelectionMatchTest {
    @Test fun manualSelectionToggleCollapseAndArrowEndpoints() {
        for (manual in 0..1) for (start in 0..1) for (end in 0..1) for (left in -2..30) for (right in -2..30) {
            for (code in listOf(-34, -21, -22, -23, -24, -25, -26, -27, -28)) {
                val c = intArrayOf(manual, start, end, left, right, code)
                assertContentEquals(referenceSelection(c), PanelRules.selection(c))
            }
        }
    }
}
