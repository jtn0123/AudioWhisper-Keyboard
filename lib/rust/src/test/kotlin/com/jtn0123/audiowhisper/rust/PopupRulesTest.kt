package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

/** The popup's entry points unpack what Rust packs (lib/rust/crates/awnative/src/popup.rs). */
class PopupRulesTest {
    private val key = PopupRect(400f, 200f, 500f, 280f)
    private val base = PopupRect(400f, 80f, 500f, 200f)

    @Test
    fun extendUnpacksTheLayout() {
        val layout = PopupGrid.extend(7, 1, key, base, 1000f)!!
        assertTrue(layout.anchorLeft)
        assertEquals(1, layout.anchorOffset)
        assertEquals(4, layout.row0Count)
        assertEquals(3, layout.row1Count)
        assertEquals(4, layout.initialIndex)
        assertEquals(7, layout.count)
        assertEquals(listOf(0, 1, 2, 3, -1, 4, 5), List(layout.count) { layout.popupIndex(it) })
        assertEquals(listOf(1, 1, 1, 0, 0, 0, 0), List(layout.count) { layout.row(it) })
        assertEquals(listOf(300f, 112f, 700f, 208f), listOf(layout.left, layout.top, layout.right, layout.bottom))
        assertNull(PopupGrid.extend(0, 0, key, base, 1000f))
    }

    @Test
    fun extendKeepsNanAndNegativeZero() {
        val layout = PopupGrid.extend(1, 0, PopupRect(Float.NaN, 0f, 1f, -0f), PopupRect(0f, -0f, 0f, 0f), 1f)!!
        assertTrue(layout.left.isNaN())
        assertEquals((-0f).toBits(), layout.top.toBits())
        assertEquals(0f.toBits(), layout.bottom.toBits())
    }

    @Test
    fun pointerFindsTheKeyOrSaysTheFingerLeft() {
        val index = { x: Float, y: Float ->
            PopupGrid.pointer(400f, 200f, 500f, 280f, 400f, 80f, 500f, 200f, true, 1, 4, 3, x, y)
        }
        assertEquals(1L, index(450f, 190f))
        assertEquals(4L, index(450f, 210f))
        assertEquals(PopupGrid.OFF_THE_POPUP, index(450f, 309f))
        assertEquals(PopupGrid.OFF_THE_POPUP, index(-1000f, 210f))
    }

    @Test
    fun elementAtCountsFromTheLastRow() {
        assertEquals(1L shl 32, PopupGrid.elementAt(intArrayOf(4, 3), 0))
        assertEquals(3L, PopupGrid.elementAt(intArrayOf(4, 3), 6))
        assertEquals(-1L, PopupGrid.elementAt(intArrayOf(4, 3), 7))
        assertEquals(-1L, PopupGrid.elementAt(intArrayOf(), -1))
    }

    @Test
    fun orderNumbersTheKeys() {
        val both = PopupOrder.order(
            main = false, relevant = 2, symbolHint = true, numberHint = true, symbolPopups = 1, numberPopups = 2,
            symbolMode = PopupOrder.Mode.ACCENT_PRIORITY, numberMode = PopupOrder.Mode.ACCENT_PRIORITY,
            mergeHintPopups = true,
        )
        assertEquals(PopupOrder.SYMBOL_HINT, both.hint)
        assertContentEquals(intArrayOf(3, PopupOrder.SYMBOL_HINT, PopupOrder.NUMBER_HINT), both.prioritized)
        assertContentEquals(intArrayOf(4, 5, 6, 7), both.other)
        assertEquals(PopupOrder.NUMBERED, both.ownList)
        val none = PopupOrder.order(
            main = true, relevant = 2, symbolHint = true, numberHint = false, symbolPopups = 0, numberPopups = 0,
            symbolMode = PopupOrder.Mode.DISABLED, numberMode = PopupOrder.Mode.HINT_PRIORITY,
            mergeHintPopups = false,
        )
        assertEquals(PopupOrder.NO_HINT, none.hint)
        assertContentEquals(intArrayOf(PopupOrder.MAIN), none.prioritized)
        assertContentEquals(intArrayOf(), none.other)
        assertEquals(PopupOrder.OWN_RELEVANT, none.ownList)
        val bare = PopupOrder.order(
            main = false, relevant = 0, symbolHint = false, numberHint = true, symbolPopups = 1, numberPopups = 2,
            symbolMode = PopupOrder.Mode.ACCENT_PRIORITY, numberMode = PopupOrder.Mode.ACCENT_PRIORITY,
            mergeHintPopups = true,
        )
        assertEquals(PopupOrder.OWN_NUMBER_POPUPS, bare.ownList)
    }

    @Test
    fun orderRefusesNegativeCounts() {
        assertFailsWith<IllegalArgumentException> {
            PopupOrder.order(
                main = false, relevant = -1, symbolHint = false, numberHint = false, symbolPopups = 0,
                numberPopups = 0, symbolMode = PopupOrder.Mode.DISABLED, numberMode = PopupOrder.Mode.DISABLED,
                mergeHintPopups = false,
            )
        }
    }

    @Test
    fun mergeSaysWhereEachKeyGoes() {
        assertContentEquals(intArrayOf(0, 2, PopupOrder.SET_MAIN), PopupOrder.merge(booleanArrayOf(true, false, true), true, true, false))
        assertContentEquals(intArrayOf(1), PopupOrder.merge(booleanArrayOf(false), true, false, false))
    }
}
