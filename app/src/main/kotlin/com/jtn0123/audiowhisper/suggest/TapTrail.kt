package com.jtn0123.audiowhisper.suggest

/**
 * Where a tap landed in its key, from the key's centre, in key widths ([dx], right is positive)
 * and key heights ([dy], down is positive). A tap on the key's right edge is dx = 0.5.
 */
data class TapOffset(val dx: Float, val dy: Float) {
    companion object {
        /** A tap in the gap beside a key still counts for it, but no further out than this. */
        const val LIMIT = 0.6f

        /** The offset of a tap at ([x], [y]) on a key centred at ([centerX], [centerY]). */
        fun of(x: Float, y: Float, centerX: Float, centerY: Float, width: Float, height: Float): TapOffset? {
            if (width <= 0f || height <= 0f) return null
            return TapOffset(
                ((x - centerX) / width).coerceIn(-LIMIT, LIMIT),
                ((y - centerY) / height).coerceIn(-LIMIT, LIMIT),
            )
        }
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
 * Written on the main thread, read while suggesting. Holds positions only, never logged.
 */
class TapTrail(private val capacity: Int = CAPACITY) {
    private class Typed(val codePoint: Int, val offset: TapOffset?)

    private var pressedCodePoint = NONE
    private var pressedOffset: TapOffset? = null
    private val trail = ArrayDeque<Typed>()

    /** A key for [codePoint] went down at [offset]. */
    @Synchronized
    fun pressed(codePoint: Int, offset: TapOffset?) {
        pressedCodePoint = codePoint
        pressedOffset = offset
    }

    /** [text] went into the editor; its first character came from the last press if they match. */
    @Synchronized
    fun typed(text: String) {
        var first = true
        text.codePoints().forEach { codePoint ->
            val fromPress = first && sameKey(codePoint, pressedCodePoint)
            add(Typed(codePoint, if (fromPress) pressedOffset else null))
            first = false
        }
        pressedCodePoint = NONE
        pressedOffset = null
    }

    /** The last typed character was deleted. */
    @Synchronized
    fun deleted() {
        trail.removeLastOrNull()
    }

    @Synchronized
    fun clear() {
        trail.clear()
        pressedCodePoint = NONE
        pressedOffset = null
    }

    /**
     * Where each character of [word] was tapped, if the last characters typed spell [word] (in
     * any case); otherwise an empty list.
     */
    @Synchronized
    fun offsetsFor(word: String): List<TapOffset?> {
        val codePoints = word.codePoints().toArray()
        if (codePoints.isEmpty() || codePoints.size > trail.size) return emptyList()
        val start = trail.size - codePoints.size
        val matches = codePoints.indices.all { sameKey(trail[start + it].codePoint, codePoints[it]) }
        if (!matches) return emptyList()
        val offsets = List(codePoints.size) { trail[start + it].offset }
        return if (offsets.all { it == null }) emptyList() else offsets
    }

    private fun add(typed: Typed) {
        trail.addLast(typed)
        while (trail.size > capacity) trail.removeFirst()
    }

    private fun sameKey(a: Int, b: Int) = a == b || Character.toLowerCase(a) == Character.toLowerCase(b)

    private companion object {
        /** Longer than any word worth correcting. */
        const val CAPACITY = 64
        const val NONE = -1
    }
}
