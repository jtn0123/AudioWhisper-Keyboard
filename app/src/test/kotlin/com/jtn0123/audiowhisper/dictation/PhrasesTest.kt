package com.jtn0123.audiowhisper.dictation

import kotlin.test.assertEquals
import kotlin.test.Test

class PhrasesTest {
    @Test
    fun midSentencePauseLowersTheCapital() {
        assertEquals("the park.", Phrases.continueSentence("meet me at", "The park."))
    }

    @Test
    fun afterACommaLowersTheCapital() {
        assertEquals("and then home.", Phrases.continueSentence("shops, ", "And then home."))
    }

    @Test
    fun afterASentenceKeepsTheCapital() {
        assertEquals("Then home.", Phrases.continueSentence("We left. ", "Then home."))
        assertEquals("Sure.", Phrases.continueSentence("Ready? ", "Sure."))
    }

    @Test
    fun keepsIAndAcronyms() {
        assertEquals("I think so.", Phrases.continueSentence("well", "I think so."))
        assertEquals("I'm in.", Phrases.continueSentence("well", "I'm in."))
        assertEquals("NASA called.", Phrases.continueSentence("then", "NASA called."))
    }

    @Test
    fun emptyFieldKeepsTheCapital() {
        assertEquals("Hello.", Phrases.continueSentence("", "Hello."))
        assertEquals("Hello.", Phrases.continueSentence(null, "Hello."))
    }
}
