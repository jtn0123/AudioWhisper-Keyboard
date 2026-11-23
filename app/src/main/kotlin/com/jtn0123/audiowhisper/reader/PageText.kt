package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ReaderRules
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
)

/** Where reading starts: a block, and a character within it. */
data class Start(val block: Int, val offset: Int = 0)

/**
 * The decisions behind reading a page aloud, kept apart from Android so they can be tested: where
 * to start, how to cut text for the speech engine, and where to carry on once a scroll has brought
 * in more text. The rules are in Rust ([ReaderRules]); this keeps what needs Java, the sentence
 * breaks and each block's [TextBlock.reveal].
 */
object PageText {
    /** Text-to-speech refuses anything longer than 4,000 characters; a margin under that. */
    val MAX_UTTERANCE: Int = ReaderRules.maxUtterance

    /** How many of the last blocks read are kept to find the place again after a scroll. */
    val RECENT: Int = ReaderRules.recent

    /** The first block that reaches below [visibleTop], so a line cut off at the top is read whole. */
    fun firstOnScreen(blocks: List<TextBlock>, visibleTop: Int, visibleBottom: Int): Start? {
        val index = ReaderRules.firstOnScreen(edges(blocks), visibleTop, visibleBottom)
        return if (index < 0) null else Start(index)
    }

    /** Whether every block shows between [visibleTop] and [visibleBottom]. */
    fun allShown(blocks: List<TextBlock>, visibleTop: Int, visibleBottom: Int): Boolean =
        ReaderRules.allShown(edges(blocks), visibleTop, visibleBottom)

    /**
     * The pieces of text a walk of the screen found, made into blocks to read: a paragraph that
     * holds a link or bold words arrives as several pieces, which are joined so it is spoken as
     * one; list bullets are dropped, and so is a piece with no words in it. A block scrolls into
     * view as its first piece does.
     */
    fun blocks(pieces: List<TextBlock>): List<TextBlock> {
        val scrolls = BooleanArray(pieces.size) { pieces[it].scrolls }
        return ReaderRules.join(pieces.map { it.text }, edges(pieces), scrolls).map { block ->
            val first = pieces[block.first]
            TextBlock(block.text, block.top, block.bottom, first.scrolls, first.reveal)
        }
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
        val found = ReaderRules.find(blocks.map { it.text }, edges(blocks), selected, visibleTop, visibleBottom)
        return found?.let { Start(it[0], it[1]) }
    }

    /**
     * Where to carry on in [blocks], the screen's text after a scroll, given what was [read] so far:
     * just past the last few blocks read, found in order. If none of them is there any more (the
     * scroll went further than the screen), the first block that wasn't read recently.
     * [blocks].size means there is nothing new.
     */
    fun resumeAfter(blocks: List<TextBlock>, read: List<String>): Int =
        ReaderRules.resumeAfter(blocks.map { it.text }, read)

    /**
     * [text] cut into pieces the speech engine takes: sentences, joined while they fit in [max],
     * and a sentence longer than that cut at a space. [max] must be positive.
     */
    fun utterances(text: String, max: Int = MAX_UTTERANCE): List<String> =
        ReaderRules.utterances(text, sentenceLengths(text), max)

    /** How long each of [text]'s sentences is, in order. */
    private fun sentenceLengths(text: String): IntArray {
        val iterator = BreakIterator.getSentenceInstance(Locale.US)
        iterator.setText(text)
        val lengths = ArrayList<Int>()
        var start = iterator.first()
        var end = iterator.next()
        while (end != BreakIterator.DONE) {
            lengths += end - start
            start = end
            end = iterator.next()
        }
        return lengths.toIntArray()
    }

    /** Each block's top, then its bottom. */
    fun edges(blocks: List<TextBlock>): IntArray =
        IntArray(2 * blocks.size) { if (it % 2 == 0) blocks[it / 2].top else blocks[it / 2].bottom }
}
