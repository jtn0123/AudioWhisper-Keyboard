package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * Reading a page aloud's rules, in Rust (lib/rust/crates/reader): where reading starts, joining
 * the pieces of text a walk of the screen found into blocks, finding a selection, carrying on
 * after a scroll, cutting text for the speech engine, and keeping the text read in sight. The
 * keyboard's `reader` package wraps these and documents them.
 *
 * Blocks' places on the screen go in as one array of edges: each block's top, then its bottom,
 * in screen pixels.
 */
object ReaderRules {
    /** The longest piece of text the speech engine is given, in UTF-16 units. */
    val maxUtterance: Int = ReaderNative.maxUtterance()

    /** How many of the last blocks read [resumeAfter] looks back over. */
    val recent: Int = ReaderNative.recent()

    /** A block made of pieces from the one at [first] on: their [text], [top] and [bottom]. */
    class Joined(val first: Int, val text: String, val top: Int, val bottom: Int)

    /** The first block whose [edges] show between [visibleTop] and [visibleBottom], or -1. */
    fun firstOnScreen(edges: IntArray, visibleTop: Int, visibleBottom: Int): Int =
        ReaderNative.firstOnScreen(edges, visibleTop, visibleBottom)

    /** Whether every block whose [edges] these are shows between [visibleTop] and [visibleBottom]. */
    fun allShown(edges: IntArray, visibleTop: Int, visibleBottom: Int): Boolean =
        ReaderNative.allShown(edges, visibleTop, visibleBottom)

    /** The pieces with these [texts], [edges] and whether each [scrolls], made into blocks to read. */
    fun join(texts: List<String>, edges: IntArray, scrolls: BooleanArray): List<Joined> {
        val packet = Utf16Batch(texts)
        val places = IntArray(1 + PLACE * texts.size)
        val joined = ReaderNative.join(packet.text, packet.lengths, edges, scrolls, places)
        var offset = 0
        return List(places[0]) { index ->
            val at = 1 + PLACE * index
            val length = places[at + 3]
            val text = joined.substring(offset, offset + length)
            offset += length
            Joined(places[at], text, places[at + 1], places[at + 2])
        }
    }

    /**
     * Where [selected] starts among the blocks with these [texts] and [edges]: the block, then the
     * offset in its text; null if it isn't there.
     */
    fun find(texts: List<String>, edges: IntArray, selected: String, visibleTop: Int, visibleBottom: Int): IntArray? {
        val packet = Utf16Batch(texts)
        return ReaderNative.find(packet.text, packet.lengths, edges, selected, visibleTop, visibleBottom)
    }

    /** Where to carry on among the blocks with these [texts] after a scroll, given the texts [read] so far. */
    fun resumeAfter(texts: List<String>, read: List<String>): Int {
        val packet = Utf16Batch(texts + read)
        return ReaderNative.resumeAfter(packet.text, packet.lengths, texts.size)
    }

    /**
     * [text], whose sentences are [sentenceLengths] long, cut into pieces of at most [max] UTF-16
     * units for the speech engine. [max] must be positive.
     */
    fun utterances(text: String, sentenceLengths: IntArray, max: Int): List<String> =
        ReaderNative.utterances(text, sentenceLengths, max).asList()

    /** Whether a piece from [first] down to [last] fits in a view from [top] to [bottom]. */
    fun fits(first: Int, last: Int, top: Int, bottom: Int): Boolean = ReaderNative.fits(first, last, top, bottom)

    /**
     * A text-to-speech voice as Android describes it: its locale's [language] and [country],
     * whether it [needsNetwork], whether it is [installed] (no "notInstalled" feature), and its
     * [quality] and [latency] (Voice.QUALITY_* and LATENCY_*).
     */
    class VoiceFacts(
        val language: String,
        val country: String,
        val needsNetwork: Boolean,
        val installed: Boolean,
        val quality: Int,
        val latency: Int,
    )

    /**
     * Which of [voices] may read text in [language]: the [preferred] one (an index, or -1) if it
     * works offline, else the offline one for [language] that best fits [country], quality and
     * latency; -1 if every voice for [language] needs the network or isn't installed.
     */
    fun offlineVoice(voices: List<VoiceFacts>, language: String, country: String, preferred: Int): Int {
        val locales = Array(voices.size * 2) { if (it % 2 == 0) voices[it / 2].language else voices[it / 2].country }
        val numbers = IntArray(voices.size * VOICE_NUMBERS)
        voices.forEachIndexed { index, voice ->
            val at = index * VOICE_NUMBERS
            numbers[at] = if (voice.needsNetwork) 1 else 0
            numbers[at + 1] = if (voice.installed) 1 else 0
            numbers[at + 2] = voice.quality
            numbers[at + 3] = voice.latency
        }
        return ReaderNative.offlineVoice(locales, numbers, language, country, preferred)
    }

    /** How many values [join] writes for each block. */
    private const val PLACE = 4

    /** How many numbers [offlineVoice] passes for each voice. */
    private const val VOICE_NUMBERS = 4
}

/**
 * The suggestion bar's speaker key between a press and its lift, in Rust: a tap reads from the
 * screen, a hold of [holdMs] or more from the top of the page, and a press while reading stops it.
 * Each press and lift says when it came, in milliseconds. Safe to use from any thread. Its native
 * memory is freed once nothing can reach it.
 */
class SpeakerKeyState {
    /** What a press or a lift asks for. */
    enum class Action { STOP, FROM_SCREEN, FROM_TOP }

    private val handle = ReaderNative.newSpeakerKey()

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** The press, at [now]; [Action.STOP] if [reading], and the lift that follows then asks nothing. */
    fun down(reading: Boolean, now: Long): Action? =
        action(ReaderNative.speakerKeyDown(handle, reading, now)).also { Reference.reachabilityFence(this) }

    /** The lift, at [now]. */
    fun up(now: Long): Action? = action(ReaderNative.speakerKeyUp(handle, now)).also { Reference.reachabilityFence(this) }

    /** The finger slid off: the lift asks nothing. */
    fun cancel() {
        ReaderNative.speakerKeyCancel(handle)
        Reference.reachabilityFence(this)
    }

    private fun action(number: Int): Action? = if (number < 0) null else Action.entries[number]

    private class Free(private val handle: Long) : Runnable {
        override fun run() = ReaderNative.freeSpeakerKey(handle)
    }

    companion object {
        /** A press this long, in milliseconds, is a hold. */
        val holdMs: Long = ReaderNative.holdMs()
    }
}
