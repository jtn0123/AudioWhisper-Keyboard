package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class WordCharactersTest {
    @Test
    fun connectorsDigitsAndAtContinueAWord() {
        for (char in "'’-@0123456789") {
            assertTrue(WordCharacters.continuesWord(char.code), "'$char' should continue a word")
        }
    }

    @Test
    fun punctuationEndsAWord() {
        for (char in ".,!?;:)\"/") {
            assertFalse(WordCharacters.continuesWord(char.code), "'$char' should end a word")
        }
    }
}
