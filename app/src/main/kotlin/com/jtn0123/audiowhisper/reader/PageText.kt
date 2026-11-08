package com.jtn0123.audiowhisper.reader

import java.text.BreakIterator
import java.util.Locale

/**
 * One piece of text on the screen, in reading order: a paragraph, a message, a label. [top] and
 * [bottom] are its edges in screen pixels as the app reports them: a web page has its whole text
 * there, and a piece scrolled out of sight is squashed to no height at the edge it went past.
 * [scrolls] is whether it is in the list that scrolls rather than a bar fixed above or below it.
 * [reveal] scrolls it into view if it is out of sight and the app can.
 */
class TextBlock(
    val text: String,
    val top: Int,
    val bottom: Int,
    val scrolls: Boolean = false,
    val reveal: suspend () -> Unit = {},
) {
    internal fun isShown(visibleTop: Int, visibleBottom: Int) =
        bottom > top && bottom > visibleTop && top < visibleBottom
}

/** Where reading starts: a block, and a character within it. */
data class Start(val block: Int, val offset: Int = 0)

/**
 * The decisions behind reading a page aloud, kept apart from Android so they can be tested: where
 * to start, how to cut text for the speech engine, and where to carry on once a scroll has brought
 * in more text.
 */
object PageText {
    /** Text-to-speech refuses anything longer than 4,000 characters; a margin under that. */
    const val MAX_UTTERANCE = 3_900

    /** How many of the last blocks read are matched to find the place again after a scroll. */
    private const val ANCHOR = 3

    /** The first block that reaches below [visibleTop], so a line cut off at the top is read whole. */
    fun firstOnScreen(blocks: List<TextBlock>, visibleTop: Int, visibleBottom: Int): Start? {
        val index = blocks.indexOfFirst { it.isShown(visibleTop, visibleBottom) }
        return if (index < 0) null else Start(index)
    }

    /**
     * The pieces of text a walk of the screen found, made into blocks to read: a paragraph that
     * holds a link or bold words arrives as several pieces, which are joined so it is spoken as
     * one; list bullets are dropped, and so is a piece with no words in it.
     */
    fun blocks(pieces: List<TextBlock>): List<TextBlock> {
        val out = ArrayList<TextBlock>()
        var run = ArrayList<TextBlock>()
        for (piece in pieces) {
            if (run.isNotEmpty() && !continues(run.last(), piece)) {
                join(run)?.let(out::add)
                run = ArrayList()
            }
            run += piece
        }
        join(run)?.let(out::add)
        return out
    }

    /**
     * Where [selected] starts: the block that contains it, preferring one that is on the screen
     * (the same words can appear twice on a page). A selection across paragraphs is found by its
     * first line. Matching ignores case and runs of spaces, which apps report differently.
     *
     * Only the words reach "Read from here", not where they are: Chrome doesn't mark the selection
     * in the tree before Android 17. A word that is on the screen twice starts at the first.
     */
    fun find(blocks: List<TextBlock>, selected: String, visibleTop: Int, visibleBottom: Int): Start? {
        val needle = normalize(selected.lineSequence().map(String::trim).firstOrNull(String::isNotEmpty) ?: return null)
        val matches = blocks.indices.mapNotNull { index ->
            val offset = indexIn(blocks[index].text, needle)
            if (offset < 0) null else Start(index, offset)
        }
        return matches.firstOrNull { blocks[it.block].isShown(visibleTop, visibleBottom) } ?: matches.firstOrNull()
    }

    /**
     * Where to carry on in [blocks], the screen's text after a scroll, given what was [read] so far:
     * just past the last few blocks read, found in order. If none of them is there any more (the
     * scroll went further than the screen), the first block that wasn't read recently.
     * [blocks].size means there is nothing new.
     */
    fun resumeAfter(blocks: List<TextBlock>, read: List<String>): Int {
        if (read.isEmpty()) return 0
        for (length in minOf(ANCHOR, read.size) downTo 1) {
            val anchor = read.takeLast(length)
            val end = lastMatch(blocks, anchor)
            if (end >= 0) return end + 1
        }
        val recent = read.takeLast(RECENT).toHashSet()
        val fresh = blocks.indexOfFirst { it.text !in recent }
        return if (fresh < 0) blocks.size else fresh
    }

    /**
     * [text] cut into pieces the speech engine takes: sentences, joined while they fit in [max],
     * and a sentence longer than that cut at a space.
     */
    fun utterances(text: String, max: Int = MAX_UTTERANCE): List<String> {
        val pieces = ArrayList<String>()
        val current = StringBuilder()
        for (sentence in sentences(text)) {
            if (current.isNotEmpty() && current.length + sentence.length > max) {
                pieces += current.toString().trim()
                current.clear()
            }
            if (sentence.length > max) pieces += splitLong(sentence, max) else current.append(sentence)
        }
        if (current.isNotBlank()) pieces += current.toString().trim()
        return pieces.filter(String::isNotBlank)
    }

    /** How many of the last blocks read are kept to find the place again after a scroll. */
    const val RECENT = 50

    private const val BULLETS = "•◦▪▫‣⁃· \u00A0"
    private const val CLOSING = ",.;:!?)]}”’"

    /**
     * Whether [next] carries on [previous]'s sentence: a web page keeps the space at the edge of a
     * link or bold words ("has a " + "link"), and closing punctuation always belongs to what's before.
     */
    private fun continues(previous: TextBlock, next: TextBlock): Boolean =
        previous.scrolls == next.scrolls && next.text.isNotEmpty() && previous.text.isNotEmpty() &&
            (previous.text.last().isWhitespace() || next.text.first().isWhitespace() || next.text.first() in CLOSING)

    private fun join(run: List<TextBlock>): TextBlock? {
        val text = run.joinToString("") { it.text }.trimStart { it in BULLETS }.trim()
        if (text.none(Char::isLetterOrDigit)) return null
        val first = run.first()
        val shown = run.filter { it.bottom > it.top }
        val top = shown.minOfOrNull { it.top } ?: first.top
        val bottom = shown.maxOfOrNull { it.bottom } ?: first.bottom
        return TextBlock(text, top, bottom, first.scrolls, first.reveal)
    }

    private fun sentences(text: String): List<String> {
        val iterator = BreakIterator.getSentenceInstance(Locale.US)
        iterator.setText(text)
        val out = ArrayList<String>()
        var start = iterator.first()
        var end = iterator.next()
        while (end != BreakIterator.DONE) {
            out += text.substring(start, end)
            start = end
            end = iterator.next()
        }
        return out
    }

    private fun splitLong(sentence: String, max: Int): List<String> {
        val out = ArrayList<String>()
        var rest = sentence.trim()
        while (rest.length > max) {
            val cut = rest.lastIndexOf(' ', max).takeIf { it > 0 } ?: max
            out += rest.substring(0, cut).trim()
            rest = rest.substring(cut).trim()
        }
        if (rest.isNotEmpty()) out += rest
        return out
    }

    /** The index of the last block that ends a run equal to [anchor], or -1. */
    private fun lastMatch(blocks: List<TextBlock>, anchor: List<String>): Int {
        for (end in blocks.lastIndex downTo anchor.size - 1) {
            val start = end - anchor.size + 1
            if (anchor.indices.all { blocks[start + it].text == anchor[it] }) return end
        }
        return -1
    }

    private val SPACES = Regex("\\s+")

    private fun normalize(text: String) = text.replace(SPACES, " ").lowercase(Locale.ROOT)

    /** Where [needle] (normalised) starts in [text], as an index into [text] itself; -1 if absent. */
    private fun indexIn(text: String, needle: String): Int {
        val folded = StringBuilder(text.length)
        val origin = IntArray(text.length)
        var lastSpace = false
        for ((index, char) in text.withIndex()) {
            val space = char.isWhitespace()
            if (space && lastSpace) continue
            origin[folded.length] = index
            folded.append(if (space) ' ' else char.lowercaseChar())
            lastSpace = space
        }
        val at = folded.indexOf(needle)
        return if (at < 0) -1 else origin[at]
    }
}
