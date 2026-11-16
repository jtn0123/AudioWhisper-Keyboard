package com.jtn0123.audiowhisper.rust

/**
 * The typing suggestions' entry points in libawnative: lib/rust/crates/awnative/src/suggest.rs and
 * suggest/tap.rs, which document each and how results are packed. [SuggestRules] and [TapPositions]
 * unpack them.
 */
internal object SuggestNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun continuesWord(codePoint: Int): Boolean

    @JvmStatic external fun wordCase(typed: String): Int

    @JvmStatic external fun applyCase(case: Int, word: String): String

    @JvmStatic external fun isMostlyCaps(typed: String): Boolean

    @JvmStatic external fun decoderInput(typed: String): String

    @JvmStatic external fun maxPreviousWords(): Int

    @JvmStatic external fun previousSlotCount(wordCount: Int, sentenceStart: Boolean): Int

    @JvmStatic external fun isCursorMidWord(textAfterCursor: String): Boolean

    @JvmStatic external fun previousWords(text: String): Array<String>

    @JvmStatic external fun modestThreshold(): Float

    @JvmStatic external fun defaultStrength(): Int

    @JvmStatic external fun strengthNames(): Array<String>

    @JvmStatic external fun exceedsThreshold(word: String, score: Int, kindAndFlags: Int, typed: String, threshold: Float): Boolean

    @JvmStatic external fun normalizedScore(before: String, after: String, score: Int): Float

    @JvmStatic external fun editDistance(before: IntArray, after: IntArray): Int

    @JvmStatic external fun revertAt(typed: String, corrected: String, start: Int, cursor: Int, textBeforeCursor: String): String?

    @JvmStatic external fun tapLimit(): Float

    @JvmStatic external fun tapTrailCapacity(): Int

    @JvmStatic external fun tapOffset(x: Float, y: Float, centerX: Float, centerY: Float, width: Float, height: Float): FloatArray?

    @JvmStatic external fun newTapTrail(capacity: Int): Long

    @JvmStatic external fun freeTapTrail(trail: Long)

    @JvmStatic external fun tapPressed(trail: Long, codePoint: Int, hasOffset: Boolean, dx: Float, dy: Float)

    @JvmStatic external fun tapTyped(trail: Long, text: String)

    @JvmStatic external fun tapDeleted(trail: Long)

    @JvmStatic external fun tapCleared(trail: Long)

    @JvmStatic external fun tapOffsetsFor(trail: Long, word: String): FloatArray

    @JvmStatic external fun tapLandscape(trail: Long, landscape: Boolean)

    @JvmStatic external fun tapKept(trail: Long, word: String)

    @JvmStatic external fun tapEvent(trail: Long, event: Int)

    @JvmStatic external fun tapAimedOffsetsFor(trail: Long, word: String, aimed: Boolean): FloatArray

    @JvmStatic external fun tapRetarget(trail: Long, layout: IntArray, bounds: FloatArray, x: Float, y: Float): Int

    @JvmStatic external fun tapAimText(trail: Long): String

    @JvmStatic external fun tapAimLoad(trail: Long, text: String)

    @JvmStatic external fun tapAimForget(trail: Long)

    @JvmStatic external fun tapAimChanged(trail: Long): Boolean

    @JvmStatic external fun qwertySizes(): IntArray

    @JvmStatic external fun qwertyKeys(): IntArray

    @JvmStatic external fun keyCenter(codePoint: Int): IntArray?

    @JvmStatic external fun touchPoint(codePoint: Int, hasOffset: Boolean, dx: Float, dy: Float): IntArray?

    @JvmStatic external fun proximityGrid(): IntArray
}
