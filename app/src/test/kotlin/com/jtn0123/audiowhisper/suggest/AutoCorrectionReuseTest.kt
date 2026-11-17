package com.jtn0123.audiowhisper.suggest

import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.Future
import java.util.concurrent.TimeUnit
import kotlin.concurrent.thread
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

/**
 * The auto-commit at a space reuses the candidate strip's decode only when it was for exactly
 * the input at the cursor now: the answer is always the one a fresh decode gives. On the real
 * engine with the shipped dictionary and the AOSP one, which pins "helo" → "help".
 */
class AutoCorrectionReuseTest {
    private val engine = requireNotNull(LatinDictionaryEngine.open(
        TestDictionaries.aosp.absolutePath, 0, TestDictionaries.aosp.length(), "en_US"))
    private val pool = Executors.newSingleThreadExecutor()

    @AfterTest
    fun close() {
        pool.shutdownNow()
        engine.close()
    }

    private fun state(typed: String, before: String = "") = TypingState.of(before, typed, "")

    /** What a suggester that has decoded nothing yet says: the answer the cache must match. */
    private fun fresh(
        state: TypingState,
        strength: AutoCorrectStrength = AutoCorrectStrength.NORMAL,
        personal: PersonalWords = PersonalWords.NONE,
        history: LearnedWords? = null,
    ) = LatinSuggester(engine, { strength }, { personal }, { history })
        .suggest(state, maxCount = 8, blockOffensiveWords = true).firstOrNull { it.isAutoCorrection }

    /** Runs [ask] while another thread holds the session, as a background decode does. */
    private fun <T> whileSessionBusy(ask: () -> T, check: (Future<T>) -> Unit) {
        val held = CountDownLatch(1)
        val release = CountDownLatch(1)
        val holder = thread { synchronized(engine.session) { held.countDown(); release.await() } }
        held.await()
        try {
            check(pool.submit(ask))
        } finally {
            release.countDown()
            holder.join()
        }
    }

    private fun <T> Future<T>.finishesWithin(millis: Long) = runCatching { get(millis, TimeUnit.MILLISECONDS) }

    @Test
    fun theStripsDecodeIsReusedWithoutWaitingForTheSession() {
        val suggester = LatinSuggester(engine)
        val typed = state("helo")
        val expected = fresh(typed)
        suggester.suggest(typed, maxCount = 8, blockOffensiveWords = true)
        whileSessionBusy({ suggester.autoCorrection(typed, blockOffensiveWords = true) }) { answer ->
            val reused = answer.finishesWithin(2_000)
            assertTrue(reused.isSuccess, "a matching strip must not decode again")
            assertEquals("help", reused.getOrNull()?.text)
            assertEquals(expected, reused.getOrNull())
        }
    }

    @Test
    fun aStripAKeystrokeBehindIsDecodedAfresh() {
        val suggester = LatinSuggester(engine)
        suggester.suggest(state("hel"), maxCount = 8, blockOffensiveWords = true)
        val now = state("helo")
        whileSessionBusy({ suggester.autoCorrection(now, blockOffensiveWords = true) }) { answer ->
            assertFalse(answer.finishesWithin(300).isSuccess, "a stale strip must not be reused")
        }
        assertEquals("help", suggester.autoCorrection(now, blockOffensiveWords = true)?.text)
        assertEquals(fresh(now), suggester.autoCorrection(now, blockOffensiveWords = true))
    }

    @Test
    fun aStripForOtherWordsBeforeIsNotReused() {
        val suggester = LatinSuggester(engine)
        suggester.suggest(state("helo", before = "I said "), maxCount = 8, blockOffensiveWords = true)
        val now = state("helo", before = "Say ")
        whileSessionBusy({ suggester.autoCorrection(now, blockOffensiveWords = true) }) { answer ->
            assertFalse(answer.finishesWithin(300).isSuccess, "the last decode was for other words before")
        }
        assertEquals(fresh(now), suggester.autoCorrection(now, blockOffensiveWords = true))
    }

    @Test
    fun aStripTappedElsewhereIsNotReused() {
        val suggester = LatinSuggester(engine)
        val centred = state("helo").copy(taps = List(4) { TapOffset(0f, 0f) })
        suggester.suggest(centred, maxCount = 8, blockOffensiveWords = true)
        val offCentre = centred.copy(taps = listOf(TapOffset(0.4f, 0f), null, null, null))
        whileSessionBusy({ suggester.autoCorrection(offCentre, blockOffensiveWords = true) }) { answer ->
            assertFalse(answer.finishesWithin(300).isSuccess, "taps that differ must decode again")
        }
    }

    @Test
    fun wordsLearnedSinceTheStripAreHonoured() {
        val learned = LearnedWords(file = null)
        val suggester = LatinSuggester(engine, history = { learned })
        val typed = state("helo")
        val before = suggester.suggest(typed, maxCount = 8, blockOffensiveWords = true).first { it.isAutoCorrection }
        repeat(TypingHistory.KNOWN_AFTER) { learned.add("helo", previous = null) }
        val now = fresh(typed, history = learned)
        assertNotEquals(before, now, "learning the word must change its auto-correction")
        assertEquals(now, suggester.autoCorrection(typed, blockOffensiveWords = true))
    }

    @Test
    fun personalWordsSwappedSinceTheStripAreHonoured() {
        var personal = PersonalWords.NONE
        val suggester = LatinSuggester(engine, personalWords = { personal })
        val typed = state("helo")
        suggester.suggest(typed, maxCount = 8, blockOffensiveWords = true)
        personal = PersonalWords(listOf(PersonalWords.Entry("helo", 200)))
        val now = fresh(typed, personal = personal)
        assertEquals(null, now)
        assertEquals(now, suggester.autoCorrection(typed, blockOffensiveWords = true))
    }

    @Test
    fun offNeverAutoCorrectsAndNeverWaitsForTheSession() {
        var strength = AutoCorrectStrength.NORMAL
        val suggester = LatinSuggester(engine, strength = { strength })
        val typed = state("helo")
        suggester.suggest(typed, maxCount = 8, blockOffensiveWords = true)
        strength = AutoCorrectStrength.OFF
        assertEquals(null, fresh(typed, strength = AutoCorrectStrength.OFF))
        whileSessionBusy({ suggester.autoCorrection(state("hepo"), blockOffensiveWords = true) }) { answer ->
            val off = answer.finishesWithin(2_000)
            assertTrue(off.isSuccess, "Off has nothing to decode")
            assertEquals(null, off.getOrNull())
        }
        assertEquals(null, suggester.autoCorrection(typed, blockOffensiveWords = true))
    }

    @Test
    fun aSentenceTypedLetterByLetterAlwaysGetsTheFreshAnswer() {
        val learned = LearnedWords(file = null)
        val suggester = LatinSuggester(engine, history = { learned })
        val text = "teh quick borwn fox jumsp over hte lazy dgo and helo agian"
        var before = ""
        for (word in text.split(' ')) {
            for (end in 1..word.length) {
                suggester.suggest(state(word.substring(0, end), before), maxCount = 8, blockOffensiveWords = true)
            }
            val typed = state(word, before)
            val expected = fresh(typed, history = learned)
            assertEquals(expected, suggester.autoCorrection(typed, blockOffensiveWords = true), word)
            learned.add(expected?.text ?: word, previous = before.trim().split(' ').last().ifEmpty { null })
            before += (expected?.text ?: word) + " "
        }
    }
}
