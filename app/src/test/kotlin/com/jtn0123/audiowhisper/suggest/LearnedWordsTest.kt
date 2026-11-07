package com.jtn0123.audiowhisper.suggest

import java.io.File
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class LearnedWordsTest {
    private val sentenceStart = PreviousWords(emptyList(), isBeginningOfSentence = true)
    private fun after(word: String) = PreviousWords(listOf(word), isBeginningOfSentence = false)

    private fun tempFile(): File = Files.createTempDirectory("learned").toFile().resolve("learned-words.txt")

    @Test
    fun aWordBecomesTheUsersOnceUsedTwice() {
        val learned = LearnedWords(null)
        learned.add("Zorblax", previous = null)
        assertNull(learned.spellingOf("zorblax"))
        assertEquals(emptyList(), learned.completions("zor", 2))
        learned.add("Zorblax", previous = null)
        assertEquals("Zorblax", learned.spellingOf("zorblax"))
        assertEquals(listOf("Zorblax"), learned.completions("zor", 2))
        assertEquals(emptyList(), learned.completions("zorblax", 2), "only longer words complete")
    }

    @Test
    fun completionsGoMostUsedFirstThenMostRecent() {
        val learned = LearnedWords(null)
        repeat(3) { learned.add("formica", null) }
        repeat(2) { learned.add("formal", null) }
        repeat(2) { learned.add("formula", null) }
        assertEquals(listOf("formica", "formula", "formal"), learned.completions("form", 3))
        assertEquals(listOf("formica", "formal"), learned.completions("form", 3) { it != "formula" })
    }

    @Test
    fun followersComeFromPairsWithTheUsersSpelling() {
        val learned = LearnedWords(null)
        repeat(2) { learned.add("later", "you") }
        learned.add("there", "you")
        learned.add("Hey", LearnedWords.SENTENCE_START)
        assertEquals(listOf("later"), learned.followers(after("You"), minCount = 2, maxCount = 3))
        assertEquals(listOf("later", "there"), learned.followers(after("you"), minCount = 1, maxCount = 3))
        assertEquals(listOf("Hey"), learned.followers(sentenceStart, minCount = 1, maxCount = 3))
        assertEquals(2, learned.pairCount(after("YOU"), "Later"))
        assertEquals(0, learned.pairCount(PreviousWords.NONE, "later"))
    }

    @Test
    fun removeTakesBackOneUse() {
        val learned = LearnedWords(null)
        repeat(2) { learned.add("later", "you") }
        learned.remove("later", "you")
        assertNull(learned.spellingOf("later"))
        assertEquals(1, learned.pairCount(after("you"), "later"))
        learned.remove("later", "you")
        assertEquals(0, learned.pairCount(after("you"), "later"))
        learned.remove("never", "seen")
    }

    @Test
    fun keepSpellingLeavesAnEarlierSpellingAlone() {
        val learned = LearnedWords(null)
        learned.add("zorblax", null)
        learned.add("Zorblax", null, keepSpelling = true)
        assertEquals("zorblax", learned.spellingOf("Zorblax"))
        learned.add("Zorblax", null)
        assertEquals("Zorblax", learned.spellingOf("zorblax"))
    }

    @Test
    fun onlyWordsAreStored() {
        val learned = LearnedWords(null)
        for (word in listOf("mp3", "a b", "tab\there", "x".repeat(LearnedWords.MAX_WORD_LENGTH + 1), "--", "")) {
            repeat(2) { learned.add(word, "previous") }
            assertNull(learned.spellingOf(word), word)
        }
        repeat(2) { learned.add("don't", "mp3") }
        assertEquals("don't", learned.spellingOf("don't"))
        assertEquals(0, learned.pairCount(after("mp3"), "don't"))
    }

    @Test
    fun theLeastRecentlyUsedGoFirst() {
        val learned = LearnedWords(null, wordCapacity = 2, pairCapacity = 1)
        repeat(2) { learned.add("alpha", "one") }
        repeat(2) { learned.add("bravo", "two") }
        repeat(2) { learned.add("alpha", null) }
        repeat(2) { learned.add("charlie", null) }
        assertEquals("alpha", learned.spellingOf("alpha"))
        assertNull(learned.spellingOf("bravo"))
        assertEquals(0, learned.pairCount(after("one"), "alpha"))
        assertEquals(2, learned.pairCount(after("two"), "bravo"))
    }

    @Test
    fun savesAndLoadsEverything() {
        val file = tempFile()
        val learned = LearnedWords(file)
        repeat(2) { learned.add("Zorblax", LearnedWords.SENTENCE_START) }
        repeat(3) { learned.add("later", "you") }
        assertTrue(learned.save())
        val reloaded = LearnedWords(file).apply { load() }
        assertEquals("Zorblax", reloaded.spellingOf("zorblax"))
        assertEquals(2, reloaded.pairCount(sentenceStart, "zorblax"))
        assertEquals(3, reloaded.pairCount(after("you"), "later"))
    }

    @Test
    fun loadKeepsWhatWasLearnedBeforeItFinished() {
        val file = tempFile()
        LearnedWords(file).apply { add("later", "you"); save() }
        val learned = LearnedWords(file)
        learned.add("later", "you")
        learned.load()
        assertEquals("later", learned.spellingOf("later"))
        assertEquals(2, learned.pairCount(after("you"), "later"))
    }

    @Test
    fun ignoresDamagedLinesAndMissingFiles() {
        val file = tempFile()
        file.writeText("# header\nw\tx\tword\nw\t2\tgood\np\t1\tonly-three\nq\t1\ta\tb\n\n")
        val learned = LearnedWords(file).apply { load() }
        assertEquals("good", learned.spellingOf("good"))
        LearnedWords(File(file.parentFile, "missing.txt")).load()
    }
}
