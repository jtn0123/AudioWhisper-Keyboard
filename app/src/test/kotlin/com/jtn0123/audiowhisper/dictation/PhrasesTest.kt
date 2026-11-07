package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.dictation.Phrases.Typing
import kotlin.test.Test
import kotlin.test.assertEquals

class PhrasesTest {
    private val names = setOf("John", "Paris")

    private fun join(before: String?, phrase: String, lastPhrase: String? = null) =
        Phrases.join(before, lastPhrase, phrase) { it in names }

    @Test
    fun `a mid-sentence pause lowers the capital`() {
        assertEquals(Typing(0, " the park."), join("meet me at", "The park."))
        assertEquals(Typing(0, "and then home."), join("shops, ", "And then home."))
    }

    @Test
    fun `the first phrase of a dictation is joined too`() {
        assertEquals(Typing(0, " maybe later."), join("I think", "Maybe later.", lastPhrase = null))
    }

    @Test
    fun `after a sentence the phrase starts one`() {
        assertEquals(Typing(0, "Then home."), join("We left. ", "Then home."))
        assertEquals(Typing(0, " Sure."), join("Ready?", "Sure."))
        assertEquals(Typing(0, " Because we ran out."), join("Hello.", "because we ran out."), "the user's own full stop stays")
    }

    @Test
    fun `keeps I, acronyms and names`() {
        assertEquals(Typing(0, " I think so."), join("well", "I think so."))
        assertEquals(Typing(0, " I'm in."), join("well", "I'm in."))
        assertEquals(Typing(0, " NASA called."), join("then", "NASA called."))
        assertEquals(Typing(0, " John about it."), join("talk to", "John about it."))
    }

    @Test
    fun `an empty field keeps the model's text`() {
        assertEquals(Typing(0, "Hello."), join("", "Hello."))
        assertEquals(Typing(0, "Hello."), join(null, "Hello."))
    }

    @Test
    fun `takes back a full stop typed where the speaker only paused`() {
        // From synthesized speech through the keyboard's VAD and Parakeet.
        assertEquals(Typing(1, " John about it."), join("to talk to.", "John about it.", lastPhrase = "I need to talk to."))
        assertEquals(Typing(1, " because we ran out."), join("some milk.", "because we ran out.", lastPhrase = " some milk."))
        assertEquals(Typing(1, " and then I bought milk."), join("e store.", "And then I bought milk.", lastPhrase = "the store."))
    }

    @Test
    fun `keeps a full stop that ends a sentence`() {
        assertEquals(Typing(0, " Maybe tomorrow."), join("about it.", "Maybe tomorrow.", lastPhrase = " John about it."))
        assertEquals(Typing(0, " I need to go."), join("ran out.", "I need to go.", lastPhrase = " because we ran out."))
        assertEquals(Typing(0, " And then?"), join("Is it?", "and then?", lastPhrase = "Is it?"), "only a full stop is taken back")
    }

    @Test
    fun `never takes back a full stop the dictation didn't type`() {
        assertEquals(Typing(0, " And more."), join("talk to.", "and more.", lastPhrase = null))
        assertEquals(Typing(0, " And more."), join("typed this.", "and more.", lastPhrase = "talk to."), "the cursor moved")
    }
}
