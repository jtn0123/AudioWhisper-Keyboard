package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class QwertyGeometryTest {
    @Test
    fun hasTheTwentySixLetters() {
        assertEquals(26, QwertyGeometry.keys.size)
        assertEquals(('a'..'z').map { it.code }.toSet(), QwertyGeometry.keys.map { it.codePoint }.toSet())
    }

    @Test
    fun keyCentersFollowTheStaggeredRows() {
        assertEquals(Pair(50, 75), QwertyGeometry.keyCenter('q'.code))
        assertEquals(Pair(100, 225), QwertyGeometry.keyCenter('a'.code))
        assertEquals(Pair(200, 375), QwertyGeometry.keyCenter('z'.code))
        assertEquals(QwertyGeometry.keyCenter('t'.code), QwertyGeometry.keyCenter('T'.code))
        assertNull(QwertyGeometry.keyCenter('\''.code))
    }

    @Test
    fun aTapLandsAtTheSameSpotInTheKey() {
        assertEquals(QwertyGeometry.keyCenter('o'.code), QwertyGeometry.touchPoint('o'.code, null))
        // Half a key right of O's centre is the O/P border; a quarter key up is above centre.
        assertEquals(Pair(900, 38), QwertyGeometry.touchPoint('o'.code, TapOffset(0.5f, -0.25f)))
        // Never off the keyboard.
        assertEquals(Pair(999, 0), QwertyGeometry.touchPoint('p'.code, TapOffset(0.6f, -0.6f)))
        assertEquals(Pair(40, 449), QwertyGeometry.touchPoint('a'.code, TapOffset(-0.6f, 5f)))
        assertNull(QwertyGeometry.touchPoint('\''.code, TapOffset(0f, 0f)))
    }

    @Test
    fun proximityGridListsTheNeighborsOfEachCell() {
        val grid = QwertyGeometry.proximityGrid()
        assertEquals(QwertyGeometry.GRID_WIDTH * QwertyGeometry.GRID_HEIGHT * QwertyGeometry.MAX_PROXIMITY_CHARS, grid.size)
        val near = nearbyLetters(grid, QwertyGeometry.keyCenter('g'.code)!!)
        assertTrue(near.containsAll("gfhtyvb".toList()), "near g: $near")
        assertTrue('q' !in near && 'p' !in near, "near g: $near")
    }

    @Test
    fun unusedSlotsArePaddedWithMinusOne() {
        val grid = QwertyGeometry.proximityGrid()
        assertTrue(grid.all { it == -1 || it in 'a'.code..'z'.code })
        assertTrue(grid.any { it == -1 })
    }

    private fun nearbyLetters(grid: IntArray, point: Pair<Int, Int>): Set<Char> {
        val cellWidth = (QwertyGeometry.KEYBOARD_WIDTH + QwertyGeometry.GRID_WIDTH - 1) / QwertyGeometry.GRID_WIDTH
        val cellHeight = (QwertyGeometry.KEYBOARD_HEIGHT + QwertyGeometry.GRID_HEIGHT - 1) / QwertyGeometry.GRID_HEIGHT
        val cell = (point.second / cellHeight) * QwertyGeometry.GRID_WIDTH + point.first / cellWidth
        val start = cell * QwertyGeometry.MAX_PROXIMITY_CHARS
        return (start until start + QwertyGeometry.MAX_PROXIMITY_CHARS)
            .map { grid[it] }
            .filter { it >= 0 }
            .map { it.toChar() }
            .toSet()
    }
}
