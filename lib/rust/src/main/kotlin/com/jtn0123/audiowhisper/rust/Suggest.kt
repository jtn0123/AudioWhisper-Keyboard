package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * The typing suggestions' rules, in Rust (lib/rust/crates/suggest): how a typed word is
 * capitalized, the words before it, whether a correction is good enough to make by itself, how to
 * undo one, and where a tap lands on the decoder's QWERTY layout. The keyboard's `suggest` package
 * calls these; its classes document the rules. Text is cased as Locale.US cases it.
 */
object SuggestRules {
    /** How a typed word is capitalized, as the keyboard's `WordCase` names them. */
    enum class Case { AS_TYPED, CAPITALIZED, ALL_CAPS }

    /** The previous words before a word, most recent first, and whether a sentence starts there. */
    class Context(val words: List<String>, val isBeginningOfSentence: Boolean)

    /** Replace the [length] characters left of the cursor with [text]. */
    class Revert(val length: Int, val text: String, val phantomSpace: Boolean)

    /** AOSP's "modest" auto-correction threshold. */
    val modestThreshold: Float = SuggestNative.modestThreshold()

    /** The most words a [Context] holds. */
    val maxPreviousWords: Int = SuggestNative.maxPreviousWords()

    /** How far beside a key, in key sizes from its centre, a tap still counts for it. */
    val tapLimit: Float = SuggestNative.tapLimit()

    /** How many characters a [TapPositions] keeps unless told otherwise. */
    val tapTrailCapacity: Int = SuggestNative.tapTrailCapacity()

    /** Whether typing [codePoint] right after a word continues the word. */
    fun continuesWord(codePoint: Int): Boolean = SuggestNative.continuesWord(codePoint)

    /** How [typed] is capitalized. */
    fun case(typed: String): Case = Case.entries[SuggestNative.wordCase(typed)]

    /** [word] capitalized as [case] says. */
    fun applyCase(case: Case, word: String): String = SuggestNative.applyCase(case.ordinal, word)

    /** Whether [typed] has more than one capital. */
    fun isMostlyCaps(typed: String): Boolean = SuggestNative.isMostlyCaps(typed)

    /** [typed] as the decoder is given it. */
    fun decoderInput(typed: String): String = SuggestNative.decoderInput(typed)

    /** The context for a word that starts right after [textBeforeWord]. */
    fun previousWords(textBeforeWord: String): Context {
        val packed = SuggestNative.previousWords(textBeforeWord)
        return Context(packed.asList().subList(1, packed.size), packed[0].isNotEmpty())
    }

    /**
     * Whether auto-correcting [typed] to the decoder's [word], with its [score] and
     * [kindAndFlags], passes [threshold].
     */
    fun exceedsThreshold(word: String, score: Int, kindAndFlags: Int, typed: String, threshold: Float): Boolean =
        SuggestNative.exceedsThreshold(word, score, kindAndFlags, typed, threshold)

    /** The decoder's [score] for [after], weighted down by how far it is from [before]. */
    fun normalizedScore(before: String, after: String, score: Int): Float =
        SuggestNative.normalizedScore(before, after, score)

    /** The edits, by code point, from [before] to [after]. */
    fun editDistance(before: IntArray, after: IntArray): Int = SuggestNative.editDistance(before, after)

    /**
     * How to undo auto-correcting [typed] to [corrected] at [start], with the cursor at [cursor]
     * after [textBeforeCursor], or null if that is no longer the correction and its separator.
     */
    fun revertAt(typed: String, corrected: String, start: Int, cursor: Int, textBeforeCursor: String): Revert? {
        val packed = SuggestNative.revertAt(typed, corrected, start, cursor, textBeforeCursor) ?: return null
        val length = (packed[0].code shl 16) or packed[1].code
        return Revert(length, packed.substring(3), packed[2].code == 1)
    }

    /**
     * Where a tap at ([x], [y]) landed on a key centred at ([centerX], [centerY]): dx and dy in
     * key sizes, or null for a key with no size.
     */
    fun tapOffset(x: Float, y: Float, centerX: Float, centerY: Float, width: Float, height: Float): FloatArray? =
        SuggestNative.tapOffset(x, y, centerX, centerY, width, height)
}

/** The fixed QWERTY layout the decoder's spatial model sees, from Rust. */
object QwertyLayout {
    private val sizes = SuggestNative.qwertySizes()
    val keyWidth: Int = sizes[0]
    val keyHeight: Int = sizes[1]
    val keyboardWidth: Int = sizes[2]
    val keyboardHeight: Int = sizes[3]
    val gridWidth: Int = sizes[4]
    val gridHeight: Int = sizes[5]
    val maxProximityChars: Int = sizes[6]

    /** The keys, three values each: code point, left and top. */
    fun keys(): IntArray = SuggestNative.qwertyKeys()

    /** The centre (x, y) of the key that types [codePoint] in either case, or null. */
    fun keyCenter(codePoint: Int): IntArray? = SuggestNative.keyCenter(codePoint)

    /**
     * Where (x, y) a tap on the key for [codePoint] lands: at ([dx], [dy]) from its centre if
     * [hasOffset], else at the centre. Null if no key types it.
     */
    fun touchPoint(codePoint: Int, hasOffset: Boolean, dx: Float, dy: Float): IntArray? =
        SuggestNative.touchPoint(codePoint, hasOffset, dx, dy)

    /** The code points near each grid cell, [maxProximityChars] per cell, -1 padded. */
    fun proximityGrid(): IntArray = SuggestNative.proximityGrid()
}

/**
 * Where each of the last [capacity] typed characters was tapped, in Rust. Safe to use from any
 * thread. Its native memory is freed once nothing can reach it.
 */
class TapPositions(capacity: Int = SuggestRules.tapTrailCapacity) {
    private val handle = SuggestNative.newTapTrail(capacity)

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** A key for [codePoint] went down, at ([dx], [dy]) from its centre if [hasOffset]. */
    fun pressed(codePoint: Int, hasOffset: Boolean, dx: Float, dy: Float) {
        SuggestNative.tapPressed(handle, codePoint, hasOffset, dx, dy)
        Reference.reachabilityFence(this)
    }

    /** [text] went into the editor. */
    fun typed(text: String) {
        SuggestNative.tapTyped(handle, text)
        Reference.reachabilityFence(this)
    }

    /** The last typed character was deleted. */
    fun deleted() {
        SuggestNative.tapDeleted(handle)
        Reference.reachabilityFence(this)
    }

    /** Everything typed is forgotten. */
    fun clear() {
        SuggestNative.tapCleared(handle)
        Reference.reachabilityFence(this)
    }

    /**
     * Where each character of [word] was tapped: three values per character, 1 if it was tapped
     * and 0 if not, then dx and dy. Empty if the last characters typed don't spell [word].
     */
    fun offsetsFor(word: String): FloatArray =
        SuggestNative.tapOffsetsFor(handle, word).also { Reference.reachabilityFence(this) }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = SuggestNative.freeTapTrail(handle)
    }
}
