package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.dictation.Phrases.Typing
import com.jtn0123.audiowhisper.dictation.Spoken.Break
import com.jtn0123.audiowhisper.dictation.Spoken.Mark
import com.jtn0123.audiowhisper.dictation.Spoken.ScratchThat
import com.jtn0123.audiowhisper.dictation.Spoken.Words
import kotlin.test.Test
import kotlin.test.assertEquals

class SpokenTest {
    @Test
    fun `a command said on its own, as the model writes it`() {
        // Parakeet's output for synthesized speech of each command after a pause.
        assertEquals(listOf(Break("\n")), Spoken.parse("New line."))
        assertEquals(listOf(Break("\n\n")), Spoken.parse("New paragraph"))
        assertEquals(listOf(ScratchThat), Spoken.parse("Delete that."))
        assertEquals(listOf(ScratchThat), Spoken.parse("scratch that"))
        assertEquals(listOf(Mark('.')), Spoken.parse("Period."))
        assertEquals(listOf(Mark('.')), Spoken.parse("Full stop."))
        assertEquals(listOf(Mark('!')), Spoken.parse("Exclamation mark"))
        assertEquals(listOf(Mark('?')), Spoken.parse("Question  mark."))
    }

    @Test
    fun `a command between sentences of one phrase`() {
        assertEquals(
            listOf(Words("Hello there."), Break("\n"), Words("How are you? Fine.")),
            Spoken.parse("Hello there. New line. How are you? Fine."),
        )
    }

    @Test
    fun `the words of a command inside a sentence are just words`() {
        assertEquals(listOf(Words("I said New Line in the middle.")), Spoken.parse("I said New Line in the middle."))
        assertEquals(listOf(Words("Mr. Smith called.")), Spoken.parse("Mr. Smith called."))
        assertEquals(listOf(Words("A period of time.")), Spoken.parse("A period of time."))
    }

    @Test
    fun `a spoken mark replaces the punctuation the dictation just typed`() {
        assertEquals(Typing(1, "?"), Phrases.punctuate("Bye. See you soon.", " See you soon.", '?'))
        assertEquals(Typing(1, "."), Phrases.punctuate("Hello there,", "Hello there,", '.'))
        assertEquals(Typing(0, ","), Phrases.punctuate("Hello there", "Hello there", ','))
        assertEquals(Typing(0, "!"), Phrases.punctuate("typed by me.", null, '!'), "not the dictation's own")
    }

    @Test
    fun `a new line starts with a capital`() {
        assertEquals(Typing(0, "See you soon."), Phrases.join("Hello there\n", "\n", "see you soon.") { false })
        assertEquals(Typing(0, "See you."), Phrases.join("Hello,\n\n", "\n\n", "See you.") { false })
    }

    @Test
    fun `a full stop said out loud is never taken back`() {
        assertEquals(Typing(0, " And then home."), Phrases.join("we left.", ".", "and then home.") { false })
    }
}
