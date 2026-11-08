package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals

/** [splitWords] reads [com.android.inputmethod.latin.BinaryDictionary.getWords]'s text. */
class SplitWordsTest {
    @Test
    fun eachWordEndsWithANewline() {
        assertEquals(listOf("the", "I'm", "e-mail"), splitWords("the\nI'm\ne-mail\n"))
    }

    @Test
    fun anEmptyDictionaryHasNoWords() {
        assertEquals(emptyList(), splitWords(""))
    }

    @Test
    fun wordsOutsideTheBasicPlaneStayWhole() {
        assertEquals(listOf("café", "😀ok"), splitWords("café\n😀ok\n"))
    }

    @Test
    fun aMissingLastNewlineKeepsTheLastWord() {
        assertEquals(listOf("one", "two"), splitWords("one\ntwo"))
    }
}
