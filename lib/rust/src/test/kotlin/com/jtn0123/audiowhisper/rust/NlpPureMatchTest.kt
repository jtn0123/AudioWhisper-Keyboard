package com.jtn0123.audiowhisper.rust

import java.io.File
import java.util.Locale
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

/** Remaining pre-A1 provider/store decisions as independent Kotlin references. */
class NlpPureMatchTest {
    @Test fun composingRangesMatchTheOriginalProviderForIcuBoundaryMetadata() {
        val random = Random(16)
        NlpSession().use { nlp ->
            repeat(50_000) {
                val text = listOf(":smile", "word", "x:smile", "", "😀word", "\uD800:word").random(random)
                val start = random.nextInt(-1, text.length + 1)
                val end = text.length
                val status = listOf(0, 100, 200, 300).random(random)
                val expected = ReferenceNlpComposing.range(status, start, end, text)
                assertEquals(expected, nlp.call("composing", status.toString(), start.toString(), end.toString(), text).toList())
            }
        }
    }
    @Test fun namesAndUnknownSwipeWordsMatchOverNativeDictionaryAndPersonalStore() {
        val entries = listOf("Zorblax", "glorp")
        val personal = PersonalWordStore(entries, intArrayOf(255, 200), listOf(null, null))
        for (file in TestDictionaries.both) namesAndUnknownSwipeWordsMatch(file, entries, personal)
    }

    private fun namesAndUnknownSwipeWordsMatch(file: File, entries: List<String>, personal: PersonalWordStore) {
        val dictionary = requireNotNull(LatinSession.open(file.readBytes()))
        dictionary.use { session ->
            val words = listOf("John", "Maybe", "HELLO", "USA", "NASA", "hello", "", "Zorblax", "ZORBLAX", "Glorp", "Zorbaxq", "Teh", "a1", "café")
            repeat(20_000) { i ->
                val word = words[i % words.size]
                val spelling = entries.firstOrNull { it == word } ?: entries.firstOrNull { it.equals(word, ignoreCase = true) }
                fun expected(loaded: Boolean): Boolean {
                    val lower = word.lowercase(Locale.US)
                    if (lower == word) return false
                    if (spelling != null) return spelling.first().isUpperCase()
                    if (!loaded) return true
                    val written = session.probability(word); val lowered = session.probability(lower)
                    return written == -1 && lowered == -1 || written > lowered
                }
                assertEquals(expected(true), session.isName(personal, word))
                assertEquals(expected(false), NlpNames.withoutDictionary(personal, word))
            }
            assertEquals(words.filter { session.probability(it) == -1 }, session.unknownWords(words))
        }
        assertFailsWith<IllegalStateException> { dictionary.isName(personal, "Zorb") }
    }
}
