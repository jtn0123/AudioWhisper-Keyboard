package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals

class PreviousWordsTest {
    private fun context(vararg words: String, sentenceStart: Boolean) = PreviousWords(words.toList(), sentenceStart)

    @Test
    fun emptyFieldIsASentenceStart() {
        assertEquals(context(sentenceStart = true), PreviousWords.from(""))
    }

    @Test
    fun takesUpToThreeWordsMostRecentFirst() {
        assertEquals(context("the", "to", "went", sentenceStart = false), PreviousWords.from("I went to the "))
    }

    @Test
    fun startOfTextEndsTheContextWithASentenceStart() {
        assertEquals(context("thank", sentenceStart = true), PreviousWords.from("thank "))
        assertEquals(context("you", "thank", sentenceStart = true), PreviousWords.from("  thank you "))
    }

    @Test
    fun sentenceTerminatorEndsTheContextWithASentenceStart() {
        assertEquals(context("I", sentenceStart = true), PreviousWords.from("Hi there. I "))
        assertEquals(context(sentenceStart = true), PreviousWords.from("Really? "))
        assertEquals(context(sentenceStart = true), PreviousWords.from("Wow! "))
    }

    @Test
    fun otherPunctuationLeavesTheContextUnknown() {
        assertEquals(PreviousWords.NONE, PreviousWords.from("Hello, "))
        assertEquals(context("and", sentenceStart = false), PreviousWords.from("eggs; and "))
    }

    @Test
    fun onlyTheCurrentLineCounts() {
        assertEquals(context("second", sentenceStart = true), PreviousWords.from("first line\nsecond "))
        assertEquals(context(sentenceStart = true), PreviousWords.from("first line\n"))
    }

    @Test
    fun aPrefixGluedToTheWordStaysWithIt() {
        // "(word": the bracket belongs to the word being typed, "said" is still the previous word.
        assertEquals(context("said", sentenceStart = true), PreviousWords.from("said ("))
        // "e-mail" typed as "e-" + "mail": the glued "e-" is part of the word, "send" precedes it.
        assertEquals(context("send", sentenceStart = true), PreviousWords.from("send e-"))
        // A word starting with a connector ("-word") leaves the context unknown, as in AOSP.
        assertEquals(PreviousWords.NONE, PreviousWords.from("rock -"))
    }

    @Test
    fun slotCountIsCappedAtThree() {
        assertEquals(2, context("thank", sentenceStart = true).slotCount)
        assertEquals(2, context("you", "thank", sentenceStart = false).slotCount)
        assertEquals(3, context("a", "b", "c", sentenceStart = true).slotCount)
        assertEquals(0, PreviousWords.NONE.slotCount)
    }
}
