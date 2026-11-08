package com.jtn0123.audiowhisper.reader

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class PageTextTest {
    /** Blocks 100 px tall, one under the other from [firstTop]. */
    private fun page(vararg texts: String, firstTop: Int = 0) =
        texts.mapIndexed { index, text -> TextBlock(text, firstTop + index * 100, firstTop + index * 100 + 100) }

    @Test
    fun readingStartsAtTheFirstBlockOnScreenEvenIfItsTopIsCutOff() {
        val blocks = page("above", "cut off", "whole", firstTop = -150)
        assertEquals(Start(1), PageText.firstOnScreen(blocks, visibleTop = 0, visibleBottom = 1000))
        assertNull(PageText.firstOnScreen(blocks, visibleTop = 2000, visibleBottom = 3000))
    }

    @Test
    fun aBlockScrolledOutOfSightIsNotOnScreen() {
        // A web page squashes what's scrolled away to no height at the edge it went past.
        val blocks = listOf(TextBlock("gone above", 210, 210), TextBlock("shown", 210, 400), TextBlock("below", 2339, 2339))
        assertEquals(Start(1), PageText.firstOnScreen(blocks, visibleTop = 0, visibleBottom = 2339))
        assertEquals(Start(2), PageText.find(blocks, "below", 0, 2339))
    }

    @Test
    fun aParagraphWithALinkInItIsReadAsOne() {
        val pieces = listOf(
            TextBlock("First has a ", 0, 50), TextBlock("link", 0, 50), TextBlock(" in it.", 0, 100),
            TextBlock("Next paragraph", 120, 170), TextBlock("bold", 120, 170), TextBlock(", then more.", 120, 170),
        )
        assertEquals(listOf("First has a link in it.", "Next paragraph"), PageText.blocks(pieces).take(2).map { it.text })
        assertEquals("bold, then more.", PageText.blocks(pieces)[2].text)
    }

    @Test
    fun bulletsAndPiecesWithNoWordsAreDropped() {
        val pieces = listOf(
            TextBlock("• ", 0, 50), TextBlock("Item one", 0, 50),
            TextBlock("• ", 60, 100), TextBlock("Item two", 60, 100), TextBlock("|", 110, 130),
        )
        assertEquals(listOf("Item one", "Item two"), PageText.blocks(pieces).map { it.text })
    }

    @Test
    fun aJoinedBlockSpansItsPiecesThatAreInSight() {
        val pieces = listOf(TextBlock("Cut ", 300, 300), TextBlock("off", 300, 380, scrolls = true))
        assertEquals(2, PageText.blocks(pieces).size, "a fixed bar and the list are never joined")
        val block = PageText.blocks(listOf(TextBlock("Cut ", 300, 300), TextBlock("off", 300, 380))).single()
        assertEquals(300 to 380, block.top to block.bottom)
    }

    @Test
    fun aSelectionStartsAtItsWordInTheBlockThatHoldsIt() {
        val blocks = page("Hello there.", "The quick brown fox jumps.", "Bye.")
        assertEquals(Start(1, 10), PageText.find(blocks, "brown", 0, 1000))
    }

    @Test
    fun aSelectionMatchesWhateverTheCaseAndSpacing() {
        val blocks = page("Line one\n  and   Two")
        assertEquals(Start(0, 11), PageText.find(blocks, "AND two", 0, 1000))
    }

    @Test
    fun aSelectionAcrossParagraphsIsFoundByItsFirstLine() {
        val blocks = page("First para.", "Second para.")
        assertEquals(Start(0, 6), PageText.find(blocks, "para.\nSecond", 0, 1000))
    }

    @Test
    fun theSameWordsOnScreenWinOverACopyOffIt() {
        val blocks = page("ok", "intro", "ok", firstTop = -100)
        assertEquals(Start(2), PageText.find(blocks, "ok", visibleTop = 150, visibleBottom = 1000))
    }

    @Test
    fun textThatIsntThereIsNotFound() {
        assertNull(PageText.find(page("abc"), "xyz", 0, 1000))
        assertNull(PageText.find(page("abc"), "  \n ", 0, 1000))
    }

    @Test
    fun afterAScrollReadingCarriesOnPastTheLastBlocksRead() {
        val read = listOf("a", "b", "c", "d")
        assertEquals(3, PageText.resumeAfter(page("b", "c", "d", "e", "f"), read))
    }

    @Test
    fun aRepeatedLabelDoesntSendReadingBack() {
        // "Copy" follows every message; the run of the last three blocks read pins the place.
        val read = listOf("one", "Copy", "two", "Copy")
        val after = page("one", "Copy", "two", "Copy", "three", "Copy")
        assertEquals(4, PageText.resumeAfter(after, read))
    }

    @Test
    fun aScrollPastEverythingReadCarriesOnFromTheFirstNewBlock() {
        assertEquals(0, PageText.resumeAfter(page("x", "y"), listOf("a", "b")))
        assertEquals(1, PageText.resumeAfter(page("b", "x"), listOf("z", "b", "q")))
    }

    @Test
    fun nothingNewAfterAScrollIsTheEnd() {
        assertEquals(2, PageText.resumeAfter(page("a", "b"), listOf("a", "b")))
    }

    @Test
    fun textIsCutIntoSentencesTheSpeechEngineTakes() {
        val text = "First one. Second one! Third?"
        assertEquals(listOf(text), PageText.utterances(text))
        assertEquals(listOf("First one.", "Second one!", "Third?"), PageText.utterances(text, max = 12))
    }

    @Test
    fun aSentenceTooLongForTheEngineIsCutAtSpaces() {
        val long = List(10) { "word$it" }.joinToString(" ")
        val pieces = PageText.utterances(long, max = 20)
        assertTrue(pieces.all { it.length <= 20 }, pieces.toString())
        assertEquals(long, pieces.joinToString(" "))
    }

    @Test
    fun blankTextIsNothingToSay() {
        assertEquals(emptyList(), PageText.utterances("  \n "))
    }
}
