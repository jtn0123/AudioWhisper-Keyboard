package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.QwertyLayout

/**
 * A fixed QWERTY layout for the decoder's spatial model, in abstract units (a key is 100 x 150).
 * Each typed letter is fed to the decoder as a touch on its key here, at the same spot in the key
 * as the real tap ([touchPoint]), so the real on-screen layout and its size don't matter.
 *
 * The proximity grid follows AOSP `ProximityInfo.computeNearestNeighbors`: each grid cell lists
 * the keys whose edge lies within 1.2 key widths of the cell's center. The layout and its grid are
 * in Rust (lib/rust/crates/suggest).
 */
object QwertyGeometry {
    val KEY_WIDTH: Int = QwertyLayout.keyWidth
    val KEY_HEIGHT: Int = QwertyLayout.keyHeight
    val KEYBOARD_WIDTH: Int = QwertyLayout.keyboardWidth
    val KEYBOARD_HEIGHT: Int = QwertyLayout.keyboardHeight
    val GRID_WIDTH: Int = QwertyLayout.gridWidth
    val GRID_HEIGHT: Int = QwertyLayout.gridHeight

    /** Native MAX_PROXIMITY_CHARS_SIZE. */
    val MAX_PROXIMITY_CHARS: Int = QwertyLayout.maxProximityChars

    /** A key's code point and its top-left corner. */
    data class Key(val codePoint: Int, val x: Int, val y: Int)

    /** The keys, row by row from the top, each row left to right. */
    val keys: List<Key> = QwertyLayout.keys().let { packed ->
        List(packed.size / 3) { Key(packed[it * 3], packed[it * 3 + 1], packed[it * 3 + 2]) }
    }

    /** The center of the key that types [codePoint] (either case), or null if there is none. */
    fun keyCenter(codePoint: Int): Pair<Int, Int>? = QwertyLayout.keyCenter(codePoint)?.let { Pair(it[0], it[1]) }

    /**
     * Where a tap on the key for [codePoint] lands here: [offset] from its centre ([TapOffset]),
     * rounded and kept on the keyboard, or the centre when the tap's position is unknown. Null if
     * no key types [codePoint].
     */
    fun touchPoint(codePoint: Int, offset: TapOffset?): Pair<Int, Int>? =
        QwertyLayout.touchPoint(codePoint, offset != null, offset?.dx ?: 0f, offset?.dy ?: 0f)
            ?.let { Pair(it[0], it[1]) }

    /** `GRID_WIDTH * GRID_HEIGHT * MAX_PROXIMITY_CHARS` code points near each cell, -1 padded. */
    fun proximityGrid(): IntArray = QwertyLayout.proximityGrid()
}
