package com.jtn0123.audiowhisper.rust

/**
 * Reading a page aloud's entry points in libawnative: lib/rust/crates/awnative/src/reader.rs,
 * which documents each. [ReaderRules] and [SpeakerKeyState] wrap them.
 */
internal object ReaderNative {
    init {
        RustLibrary.load()
    }

    @JvmStatic external fun maxUtterance(): Int

    @JvmStatic external fun recent(): Int

    @JvmStatic external fun holdMs(): Long

    @JvmStatic external fun firstOnScreen(edges: IntArray, visibleTop: Int, visibleBottom: Int): Int

    @JvmStatic external fun allShown(edges: IntArray, visibleTop: Int, visibleBottom: Int): Boolean

    @JvmStatic external fun join(texts: Array<String>, edges: IntArray, scrolls: BooleanArray, places: IntArray): Array<String>

    @JvmStatic external fun find(
        texts: Array<String>,
        edges: IntArray,
        selected: String,
        visibleTop: Int,
        visibleBottom: Int,
    ): IntArray?

    @JvmStatic external fun resumeAfter(texts: Array<String>, read: Array<String>): Int

    @JvmStatic external fun utterances(text: String, sentenceLengths: IntArray, max: Int): Array<String>

    @JvmStatic external fun fits(first: Int, last: Int, top: Int, bottom: Int): Boolean

    @JvmStatic external fun newSpeakerKey(): Long

    @JvmStatic external fun freeSpeakerKey(key: Long)

    @JvmStatic external fun speakerKeyDown(key: Long, reading: Boolean, now: Long): Int

    @JvmStatic external fun speakerKeyUp(key: Long, now: Long): Int

    @JvmStatic external fun speakerKeyCancel(key: Long)
}
