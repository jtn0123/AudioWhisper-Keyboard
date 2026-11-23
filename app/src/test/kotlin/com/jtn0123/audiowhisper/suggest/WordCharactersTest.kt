package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.SuggestRules
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class WordCharactersTest {
    @Test
    fun connectorsDigitsAndAtContinueAWord() {
        for (char in "'’-@0123456789") {
            assertTrue(SuggestRules.continuesWord(char.code), "'$char' should continue a word")
        }
    }

    @Test
    fun punctuationEndsAWord() {
        for (char in ".,!?;:)\"/") {
            assertFalse(SuggestRules.continuesWord(char.code), "'$char' should end a word")
        }
    }
}
