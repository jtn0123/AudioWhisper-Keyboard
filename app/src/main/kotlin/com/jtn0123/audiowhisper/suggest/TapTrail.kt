package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules
import com.jtn0123.audiowhisper.rust.TapPositions

/**
 * Where a tap landed in its key, from the key's centre, in key widths ([dx], right is positive)
 * and key heights ([dy], down is positive). A tap on the key's right edge is dx = 0.5.
 */
data class TapOffset(val dx: Float, val dy: Float) {
    companion object {
        /** A tap in the gap beside a key still counts for it, but no further out than this. */
        val LIMIT: Float = SuggestRules.tapLimit

        /**
         * The offset of a tap at ([x], [y]) on a key centred at ([centerX], [centerY]), each
         * coordinate held to [LIMIT]; null for a key with no width or height.
         */
        fun of(x: Float, y: Float, centerX: Float, centerY: Float, width: Float, height: Float): TapOffset? =
            SuggestRules.tapOffset(x, y, centerX, centerY, width, height)?.let { TapOffset(it[0], it[1]) }
    }
}

/**
 * Where each recently typed character was tapped, so the decoder hears that a tap on the O/P
 * border was nearly a P, not a clean O. The keyboard view reports each key press ([pressed]); the
 * character that press types is then [typed]. A character with no press behind it (a long-press
 * popup, a hardware key) has no position.
 *
 * [offsetsFor] lines the trail up with the word being suggested for from its end, so the trail
 * needs no resetting as words come and go: a word that doesn't match it (the cursor moved, a
 * whole word was deleted) simply gets no positions, which is how every word was decoded before.
 *
 * The trail is kept in Rust (lib/rust/crates/suggest) behind a lock: it is written on the main
 * thread and read while suggesting. Holds positions only, never logged.
 */
class TapTrail(capacity: Int = SuggestRules.tapTrailCapacity) {
    private val positions = TapPositions(capacity)

    /** A key for [codePoint] went down at [offset]. */
    fun pressed(codePoint: Int, offset: TapOffset?) =
        positions.pressed(codePoint, offset != null, offset?.dx ?: 0f, offset?.dy ?: 0f)

    /**
     * [text] went into the editor; its first character came from the last press if they are the
     * same key (the same character in either case).
     */
    fun typed(text: String) = positions.typed(text)

    /** The last typed character was deleted. */
    fun deleted() = positions.deleted()

    fun clear() = positions.clear()

    /**
     * Where each character of [word] was tapped, if the last characters typed spell [word] (in
     * any case) and at least one of them was tapped; otherwise an empty list.
     */
    fun offsetsFor(word: String): List<TapOffset?> {
        val packed = positions.offsetsFor(word)
        return List(packed.size / 3) { i ->
            if (packed[i * 3] == 0f) null else TapOffset(packed[i * 3 + 1], packed[i * 3 + 2])
        }
    }
}
