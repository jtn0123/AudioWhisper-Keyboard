package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * The word stores through JNI, as the keyboard calls them: nulls passed both ways, text passed
 * through unit for unit, bad input thrown as Kotlin threw it, and the stores' native memory. The
 * stores themselves are tested in Rust and, against the Kotlin they replace, in the app.
 */
class WordStoresTest {
    @Test
    fun learnedWordsCrossWhole() {
        val store = LearnedWordStore()
        repeat(2) { store.add("Zorblax", WordStoreRules.sentenceStart, keepSpelling = false) }
        repeat(2) { store.add("later", "you", keepSpelling = false) }
        store.add("unknown", null, keepSpelling = false)
        assertEquals(listOf("Zorblax", "later"), store.knownSpellings())
        assertEquals("Zorblax", store.spellingOf("ZORBLAX"))
        assertNull(store.spellingOf("unknown"))
        assertEquals(listOf("Zorblax"), store.completions("zor", 1))
        assertEquals(listOf("Zorblax"), store.followers(null, true, 1, 3))
        assertEquals(listOf("later"), store.followers("YOU", false, 1, 3))
        assertEquals(emptyList(), store.followers(null, false, 1, 3))
        assertEquals(2, store.pairCount("you", true, "Later"))
        store.remove("later", "you")
        assertEquals(1, store.pairCount("you", false, "later"))
        store.clear()
        assertEquals(emptyList(), store.knownSpellings())
    }

    @Test
    fun swipeBoostsGrowWithUseAndGoWithTheWords() {
        val store = LearnedWordStore()
        repeat(4) { store.add("Zorblax", null, keepSpelling = false) }
        repeat(2) { store.add("later", null, keepSpelling = false) }
        store.add("once", null, keepSpelling = false)
        val words = listOf("Zorblax", "zorblax", "later", "once", "never")
        assertContentEquals(intArrayOf(10, 0, 5, 0, 0), store.swipeBoosts(words))
        store.clear()
        assertContentEquals(IntArray(words.size), store.swipeBoosts(words))
        assertContentEquals(intArrayOf(), store.swipeBoosts(emptyList()))
    }

    @Test
    fun savedTextReadsBack() {
        val store = LearnedWordStore()
        repeat(3) { store.add("later", "you", keepSpelling = false) }
        val text = store.savedText()
        assertEquals("# AudioWhisper learned words, v1\nw\t3\tlater\np\t3\tyou\tlater\n", text)
        val reloaded = LearnedWordStore().apply { merge(text) }
        assertEquals(text, reloaded.savedText())
        val kept = KeptWordStore()
        assertTrue(kept.add("Teh", remember = true))
        assertFalse(kept.add("secret", remember = false))
        assertEquals("teh\n", kept.savedText())
        val restarted = KeptWordStore().apply { merge(" TEH \r\n\n") }
        assertTrue("teh" in restarted)
        assertFalse("secret" in restarted)
    }

    @Test
    fun oddUtf16PassesThroughUnchanged() {
        // A lone surrogate isn't a letter, so it can't be learned, but it can be kept and listed.
        val odd = "a\uD800b😀\u0000"
        assertFalse(WordStoreRules.isStorable(odd))
        val kept = KeptWordStore()
        assertTrue(kept.add(odd, remember = true))
        assertTrue(odd in kept)
        assertEquals("$odd\n", kept.savedText())
        val personal = PersonalWordStore(listOf(odd, "x$odd"), intArrayOf(1, 2), listOf(null, odd))
        assertEquals(odd, personal.spellingOf(odd))
        assertEquals(listOf("x$odd"), personal.completions(odd, 5))
    }

    @Test
    fun personalWordsUnpackTheirOrder() {
        val personal = PersonalWordStore(listOf("Justin", " ", "iPhone", "on my way"), intArrayOf(200, 255, 250, 150), listOf(null, null, null, "omw"))
        assertContentEquals(intArrayOf(2, 0, 3), personal.order)
        assertEquals("iPhone", personal.spellingOf("IPHONE"))
        assertNull(personal.spellingOf("just"))
        assertEquals(listOf("on my way"), personal.completions("OMW", 2))
        assertEquals(listOf("Justin"), personal.completions("ju", 2))
    }

    @Test
    fun badInputThrowsAsKotlinDid() {
        val store = LearnedWordStore()
        val negative = assertFailsWith<IllegalArgumentException> { store.followers("you", false, 1, -1) }
        assertEquals("Requested element count -1 is less than zero.", negative.message)
        // With no previous word there is nothing to take, so nothing throws.
        assertEquals(emptyList(), store.followers(null, false, 1, -1))
        assertFailsWith<IllegalArgumentException> { PersonalWordStore(listOf("a"), intArrayOf(), listOf(null)) }
    }

    @Test
    fun manyStoresAreFreed() {
        // Each store's native memory goes once it is unreachable; making many must not run out.
        repeat(30_000) {
            LearnedWordStore(4, 4).add("word", "previous", keepSpelling = false)
            KeptWordStore(4).add("word", remember = true)
            PersonalWordStore(listOf("word"), intArrayOf(1), listOf(null)).spellingOf("word")
        }
        System.gc()
    }

    @Test
    fun theRulesAndNumbersComeFromRust() {
        assertEquals("", WordStoreRules.sentenceStart)
        assertEquals(2, WordStoreRules.knownAfter)
        assertEquals(listOf(5000, 20_000, 48, 1000), with(WordStoreRules) {
            listOf(learnedWordCapacity, learnedPairCapacity, maxWordLength, keptCapacity)
        })
        assertTrue(WordStoreRules.isStorable("rock’n’roll"))
        assertFalse(WordStoreRules.isStorable("mp3"))
        assertEquals("you", WordStoreRules.previousKey("you", true))
        assertEquals("", WordStoreRules.previousKey(null, true))
        assertNull(WordStoreRules.previousKey(null, false))
        assertTrue(WordStoreRules.appliesTo(null, "en"))
        assertTrue(WordStoreRules.appliesTo("EN-gb", "en"))
        assertFalse(WordStoreRules.appliesTo("de_DE", "en"))
    }
}
