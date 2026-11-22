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
 * NLP session's typing events then say what it typed, was deleted, kept or taken back (see
 * lib/rust/crates/nlp/src/typing.rs). A character with no press behind it (a long-press popup, a
 * hardware key) has no position.
 *
 * [offsetsFor] lines the trail up with the word being suggested for from its end, so the trail
 * needs no resetting as words come and go: a word that doesn't match it (the cursor moved, a
 * whole word was deleted) simply gets no positions, which is how every word was decoded before.
 *
 * The trail is kept in Rust (lib/rust/crates/suggest) behind a lock: it is written on the main
 * thread and read while suggesting. Holds positions only, never logged.
 */
class TapTrail(capacity: Int = SuggestRules.tapTrailCapacity) {
    /** The Rust side, which the Rust renderer's hit test reads where this user taps from. */
    val positions = TapPositions(capacity)

    /** A key for [codePoint] went down at [offset]. */
    fun pressed(codePoint: Int, offset: TapOffset?) =
        positions.pressed(codePoint, offset != null, offset?.dx ?: 0f, offset?.dy ?: 0f)

    /**
     * Where each character of [word] was tapped, if the last characters typed spell [word] (in
     * any case) and at least one of them was tapped; otherwise an empty list.
     */
    fun offsetsFor(word: String): List<TapOffset?> = unpack(positions.offsetsFor(word))

    /**
     * [offsetsFor], with each offset moved by where this user is known to tap that key if
     * [aimed], so a tap where they usually hit P reads as a centred P (see suggest/tap_aim.rs).
     */
    fun offsetsFor(word: String, aimed: Boolean): List<TapOffset?> = unpack(positions.offsetsFor(word, aimed))

    private fun unpack(packed: FloatArray): List<TapOffset?> = List(packed.size / 3) { i ->
        if (packed[i * 3] == 0f) null else TapOffset(packed[i * 3 + 1], packed[i * 3 + 2])
    }

    /** Taps are learned and aimed per orientation; this says which the screen is in. */
    fun landscape(landscape: Boolean) = positions.landscape(landscape)

    /**
     * The key a tap at ([x], [y]) types: [hit] unless it lies near a border that, if [aimed], both
     * keys' learned aims say the tap was meant across, or else that the neighbour's letter, being
     * likelier to come next, takes a little of [hit]'s room. [layout] and [bounds] are as
     * [TapPositions.retarget] takes them, less the hit.
     */
    fun retarget(hit: Int, layout: IntArray, bounds: FloatArray, x: Float, y: Float, aimed: Boolean): Int {
        layout[0] = hit
        return positions.retarget(layout, bounds, x, y, aimed).takeIf { it >= 0 } ?: hit
    }

    /** Where this user taps, as saved in the app's private files: numbers only. */
    fun aimText(): String = positions.aimText()

    fun loadAim(text: String) = positions.loadAim(text)

    fun forgetAim() = positions.forgetAim()
}
