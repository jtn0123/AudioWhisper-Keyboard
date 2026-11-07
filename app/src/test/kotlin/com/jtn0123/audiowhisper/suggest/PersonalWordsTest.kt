package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class PersonalWordsTest {
    private val words = PersonalWords(
        listOf(
            PersonalWords.Entry("Justin", 200),
            PersonalWords.Entry("iPhone", 250),
            PersonalWords.Entry("justinian", 100),
            PersonalWords.Entry("on my way", 150, shortcut = "omw"),
            PersonalWords.Entry(" ", 255),
        ),
    )

    @Test
    fun spellsAWordTheUsersWay() {
        assertEquals("Justin", words.spellingOf("Justin"))
        assertEquals("Justin", words.spellingOf("justin"))
        assertEquals("iPhone", words.spellingOf("IPHONE"))
        assertNull(words.spellingOf("just"))
    }

    @Test
    fun completesLongerWordsMostFrequentFirst() {
        assertEquals(listOf("Justin", "justinian"), words.completions("ju", 5))
        assertEquals(listOf("Justin"), words.completions("ju", 1))
        // The typed word itself is not a completion; only longer ones are.
        assertEquals(listOf("justinian"), words.completions("Justin", 5))
        assertEquals(emptyList(), words.completions("", 5))
    }

    @Test
    fun expandsAShortcutFirst() {
        assertEquals(listOf("on my way"), words.completions("omw", 5))
        assertEquals(listOf("on my way"), words.completions("OMW", 5))
    }

    @Test
    fun appliesToItsLanguageOrToAll() {
        assertTrue(PersonalWords.appliesTo(null, "en"))
        assertTrue(PersonalWords.appliesTo("en_US", "en"))
        assertTrue(PersonalWords.appliesTo("en-GB", "en"))
        assertFalse(PersonalWords.appliesTo("de_DE", "en"))
        assertTrue(PersonalWords.NONE.isEmpty)
        assertFalse(words.isEmpty)
    }
}
