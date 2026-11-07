package com.jtn0123.audiowhisper.suggest

import kotlin.math.roundToInt

/**
 * A fixed QWERTY layout for the decoder's spatial model, in abstract units (a key is 100 x 150).
 * Each typed letter is fed to the decoder as a touch on its key here, at the same spot in the key
 * as the real tap ([touchPoint]), so the real on-screen layout and its size don't matter.
 *
 * The proximity grid follows AOSP `ProximityInfo.computeNearestNeighbors`: each grid cell lists
 * the keys whose edge lies within 1.2 key widths of the cell's center.
 */
object QwertyGeometry {
    const val KEY_WIDTH = 100
    const val KEY_HEIGHT = 150
    const val KEYBOARD_WIDTH = 10 * KEY_WIDTH
    const val KEYBOARD_HEIGHT = 3 * KEY_HEIGHT
    const val GRID_WIDTH = 32
    const val GRID_HEIGHT = 16

    /** Native MAX_PROXIMITY_CHARS_SIZE. */
    const val MAX_PROXIMITY_CHARS = 16

    private const val SEARCH_DISTANCE = 1.2f
    private val ROWS = listOf("qwertyuiop", "asdfghjkl", "zxcvbnm")
    private val ROW_OFFSETS = intArrayOf(0, KEY_WIDTH / 2, KEY_WIDTH * 3 / 2)

    /** A key's code point and its top-left corner. */
    data class Key(val codePoint: Int, val x: Int, val y: Int)

    val keys: List<Key> = ROWS.flatMapIndexed { row, letters ->
        letters.mapIndexed { column, letter ->
            Key(letter.code, ROW_OFFSETS[row] + column * KEY_WIDTH, row * KEY_HEIGHT)
        }
    }

    private val keysByCodePoint = keys.associateBy { it.codePoint }

    /** The center of the key that types [codePoint] (either case), or null if there is none. */
    fun keyCenter(codePoint: Int): Pair<Int, Int>? {
        val key = keysByCodePoint[Character.toLowerCase(codePoint)] ?: return null
        return Pair(key.x + KEY_WIDTH / 2, key.y + KEY_HEIGHT / 2)
    }

    /**
     * Where a tap on the key for [codePoint] lands here: [offset] from its centre ([TapOffset]),
     * or the centre when the tap's position is unknown. Null if no key types [codePoint].
     */
    fun touchPoint(codePoint: Int, offset: TapOffset?): Pair<Int, Int>? {
        val (x, y) = keyCenter(codePoint) ?: return null
        if (offset == null) return Pair(x, y)
        return Pair(
            (x + offset.dx * KEY_WIDTH).roundToInt().coerceIn(0, KEYBOARD_WIDTH - 1),
            (y + offset.dy * KEY_HEIGHT).roundToInt().coerceIn(0, KEYBOARD_HEIGHT - 1),
        )
    }

    /** `GRID_WIDTH * GRID_HEIGHT * MAX_PROXIMITY_CHARS` code points near each cell, -1 padded. */
    fun proximityGrid(): IntArray {
        val cellWidth = (KEYBOARD_WIDTH + GRID_WIDTH - 1) / GRID_WIDTH
        val cellHeight = (KEYBOARD_HEIGHT + GRID_HEIGHT - 1) / GRID_HEIGHT
        val threshold = (KEY_WIDTH * SEARCH_DISTANCE).toInt()
        val grid = IntArray(GRID_WIDTH * GRID_HEIGHT * MAX_PROXIMITY_CHARS) { -1 }
        for (cell in 0 until GRID_WIDTH * GRID_HEIGHT) {
            val centerX = (cell % GRID_WIDTH) * cellWidth + cellWidth / 2
            val centerY = (cell / GRID_WIDTH) * cellHeight + cellHeight / 2
            keys.filter { squaredDistanceToEdge(it, centerX, centerY) < threshold * threshold }
                .take(MAX_PROXIMITY_CHARS)
                .forEachIndexed { slot, key -> grid[cell * MAX_PROXIMITY_CHARS + slot] = key.codePoint }
        }
        return grid
    }

    private fun squaredDistanceToEdge(key: Key, x: Int, y: Int): Int {
        val dx = maxOf(0, key.x - x, x - (key.x + KEY_WIDTH))
        val dy = maxOf(0, key.y - y, y - (key.y + KEY_HEIGHT))
        return dx * dx + dy * dy
    }
}
