package com.jtn0123.audiowhisper.suggest

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Policy tests against a fake decoder. Scores, kinds and probabilities are the ones the real
 * engine produced for these words with the bundled en_US dictionary.
 */
class LatinSuggesterTest {
    private val correction = RawSuggestion.KIND_CORRECTION or RawSuggestion.FLAG_APPROPRIATE_FOR_AUTO_CORRECTION
    private val exact = correction or RawSuggestion.FLAG_EXACT_MATCH
    private val whitelist = RawSuggestion.KIND_WHITELIST
    private val whitelistScore = Int.MAX_VALUE - 1

    private val decoder = FakeDecoder(
        results = mapOf(
            "teh" to listOf(RawSuggestion("the", 477_893, correction), RawSuggestion("ten", 400_787, correction)),
            "teh2" to listOf(RawSuggestion("the", 477_893, correction)),
            "im" to listOf(RawSuggestion("I'm", whitelistScore, whitelist), RawSuggestion("in", 571_761, correction)),
            "form" to listOf(RawSuggestion("form", 1_819_619, exact), RawSuggestion("from", 422_604, correction)),
            "fro" to listOf(RawSuggestion("fro", 1_536_913, exact), RawSuggestion("from", 471_045, correction)),
            "ok" to listOf(RawSuggestion("OK", 1_740_557, exact), RawSuggestion("on", 521_841, correction)),
            "london" to listOf(RawSuggestion("London", 1_801_319, exact)),
            "yeah" to listOf(RawSuggestion("yeah", 1_753_136, exact), RawSuggestion("year", 235_587, correction)),
            "lol" to listOf(RawSuggestion("oil", 227_445, correction)),
            // The real decoder ranks the case-only match first.
            "wor" to listOf(
                RawSuggestion("WOR", 1_600_000, exact),
                RawSuggestion("work", 600_000, correction),
                RawSuggestion("word", 580_000, correction),
                RawSuggestion("worn", 570_000, correction),
            ),
            "jello" to listOf(RawSuggestion("jello", 1_500_000, exact), RawSuggestion("Jello", 1_400_000, correction)),
            "jelo" to listOf(RawSuggestion("Jello", 600_000, correction), RawSuggestion("jello", 590_000, correction)),
            "u" to listOf(RawSuggestion("I", 484_164, correction)),
            "nit" to listOf(
                RawSuggestion("not", whitelistScore, whitelist),
                RawSuggestion("nit", 1_493_598, exact),
                RawSuggestion("not", 505_979, correction),
            ),
            "" to listOf(
                RawSuggestion("you", 90, RawSuggestion.KIND_PREDICTION),
                RawSuggestion("God", 80, RawSuggestion.KIND_PREDICTION),
                RawSuggestion("you", 70, RawSuggestion.KIND_PREDICTION),
            ),
        ),
        probabilities = mapOf("jello" to 120, "WOR" to 79, "work" to 200, "word" to 190, "worn" to 60, "OK" to 150, "on" to 220, "form" to 158, "from" to 191, "fro" to 73, "yeah" to 135, "nit" to 59),
        ngrams = setOf("came" to "from"),
    )
    private val suggester = LatinSuggester(decoder)

    private fun suggest(typed: String, before: String = "", after: String = "", max: Int = 8) =
        suggester.suggest(TypingState.of(before, typed, after), max, blockOffensiveWords = true)

    private fun autoCorrection(typed: String, before: String = "", after: String = "") =
        suggest(typed, before, after).firstOrNull { it.isAutoCorrection }?.text

    @Test
    fun correctsATypoAndKeepsTheTypedWordTappable() {
        val words = suggest("teh")
        assertEquals(listOf("the", "teh", "ten"), words.map { it.text })
        assertEquals(listOf(true, false, false), words.map { it.isAutoCorrection })
        assertTrue(words[1].isTypedWord)
    }

    @Test
    fun matchesTheCaseOfTheTypedWord() {
        assertEquals("The", autoCorrection("Teh"))
        assertEquals(listOf("THE", "TEH", "TEN").toSet(), suggest("TEH").map { it.text }.toSet())
    }

    @Test
    fun neverAutoCorrectsMostlyCapsWords() {
        assertEquals(null, autoCorrection("TEH"))
    }

    @Test
    fun whitelistedWordsAreCorrectedEvenWhenCapitalized() {
        assertEquals("I'm", autoCorrection("im"))
        assertEquals("I'm", autoCorrection("Im"))
    }

    @Test
    fun whitelistBeatsAValidTypedWord() {
        assertEquals("not", autoCorrection("nit"))
        assertEquals(listOf("not", "nit"), suggest("nit").map { it.text })
    }

    @Test
    fun leavesValidWordsAlone() {
        val words = suggest("form")
        assertEquals(listOf("form", "from"), words.map { it.text })
        assertFalse(words.any { it.isAutoCorrection })
    }

    @Test
    fun correctsAValidWordThatIsMuchRarerInContext() {
        assertEquals("from", autoCorrection("fro", before = "came "))
        // Without an n-gram backing the suggestion, the valid word stays.
        assertEquals(null, autoCorrection("fro", before = "and "))
        assertEquals(null, autoCorrection("fro"))
    }

    @Test
    fun capitalizesProperNounsButDoesNotTurnWordsIntoAcronyms() {
        assertEquals("London", autoCorrection("london"))
        assertEquals(null, autoCorrection("ok"))
        assertEquals(listOf("ok", "OK", "on"), suggest("ok").map { it.text })
    }

    @Test
    fun offersEachWordOnceInOneSpelling() {
        // A typed word that is itself a word isn't offered again capitalized.
        assertEquals(listOf("jello"), suggest("jello").map { it.text })
        // Two spellings of the same suggestion collapse to the lower-case one, in the better slot.
        val jelo = suggest("jelo").map { it.text }
        assertEquals(listOf("jello"), jelo.filter { it.equals("jello", ignoreCase = true) })
    }

    @Test
    fun anAcronymOfTheTypedLettersGoesAfterTheWordsBeingTyped() {
        // Behind the more frequent words that continue "wor", ahead of the rarer "worn".
        assertEquals(listOf("wor", "work", "word", "WOR", "worn"), suggest("wor").map { it.text })
        // Still not an auto-correction target, so "wor" isn't corrected either.
        assertEquals(null, autoCorrection("wor"))
        // "on" doesn't continue "ok", so "OK" keeps its slot.
        assertEquals(listOf("ok", "OK", "on"), suggest("ok").map { it.text })
    }

    @Test
    fun belowThresholdOffersButDoesNotCorrect() {
        assertEquals(null, autoCorrection("yeah"))
        assertEquals(listOf("lol", "oil"), suggest("lol").map { it.text })
        assertEquals(null, autoCorrection("lol"))
    }

    @Test
    fun singleLettersAreOnlyCorrectedByTheWhitelist() {
        assertEquals(null, autoCorrection("u"))
    }

    @Test
    fun noAutoCorrectionWithDigitsOrInsideAWord() {
        assertEquals(null, autoCorrection("teh2"))
        assertEquals(null, autoCorrection("teh", after = "re"))
        assertEquals("the", autoCorrection("teh", after = " and"))
    }

    @Test
    fun predictsTheNextWordWithoutAutoCorrecting() {
        val words = suggest("", before = "thank ")
        assertEquals(listOf("you", "God"), words.map { it.text })
        assertFalse(words.any { it.isAutoCorrection || it.isTypedWord })
        assertEquals(emptyList(), suggest("", before = "Hello, "))
    }

    @Test
    fun respectsTheCandidateLimit() {
        assertEquals(listOf("the"), suggest("teh", max = 1).map { it.text })
        assertEquals(listOf("form"), suggest("form", max = 1).map { it.text })
        assertEquals(emptyList(), suggest("teh", max = 0))
    }

    @Test
    fun spellCheckFlagsTyposWithConfidence() {
        assertEquals(SpellCheck(true, emptyList(), false), suggester.spell("form", 3, true))
        assertEquals(SpellCheck(false, listOf("the", "ten"), true), suggester.spell("teh", 3, true))
        assertEquals(SpellCheck(false, listOf("oil"), false), suggester.spell("lol", 3, true))
    }

    private val personal = PersonalWords(
        listOf(PersonalWords.Entry("teh", 200), PersonalWords.Entry("iPhone", 250), PersonalWords.Entry("formica", 120)),
    )
    private val personalSuggester = LatinSuggester(decoder, personalWords = { personal })

    private fun suggestPersonal(typed: String, after: String = "") =
        personalSuggester.suggest(TypingState.of("", typed, after), 8, blockOffensiveWords = true)

    @Test
    fun neverCorrectsAPersonalWordAway() {
        val words = suggestPersonal("teh")
        assertFalse(words.any { it.isAutoCorrection })
        assertEquals(listOf("teh", "the", "ten"), words.map { it.text })
        assertEquals(SpellCheck(true, emptyList(), false), personalSuggester.spell("teh", 3, true))
    }

    @Test
    fun givesAPersonalWordTheUsersSpelling() {
        // The fake decoder knows no "iphone", so only the personal dictionary has it.
        assertEquals("iPhone", suggestPersonal("iphone").first { it.isAutoCorrection }.text)
        assertFalse(suggestPersonal("iphone", after = "s").any { it.isAutoCorrection })
    }

    @Test
    fun offersPersonalCompletionsAfterTheTypedWord() {
        assertEquals(listOf("form", "formica", "from"), suggestPersonal("form").map { it.text })
        assertEquals(listOf("Form", "Formica", "From"), suggestPersonal("Form").map { it.text })
    }

    private val learned = LearnedWords(null)
    private val learningSuggester = LatinSuggester(decoder, history = { learned })

    private fun learn(word: String, previous: String? = null, times: Int = TypingHistory.KNOWN_AFTER) =
        repeat(times) { learned.add(word, previous) }

    @Test
    fun offersUsedWordsTheDictionaryLacks() {
        learn("wordle")
        learn("word")
        learn("wormhole", times = 1)
        // "word" is the dictionary's to rank; "wormhole" isn't the user's word yet.
        assertEquals(listOf("wo", "wordle"), learningSuggester.suggest(TypingState.of("", "wo", ""), 8, true).map { it.text })
    }

    @Test
    fun neverCorrectsAWordTheUserKeepsTyping() {
        learn("teh")
        val words = learningSuggester.suggest(TypingState.of("", "teh", ""), 8, true)
        assertFalse(words.any { it.isAutoCorrection })
        assertEquals(SpellCheck(true, emptyList(), false), learningSuggester.spell("teh", 3, true))
        // Typed once, it is still a typo.
        learned.clear()
        learn("teh", times = 1)
        assertEquals("the", learningSuggester.suggest(TypingState.of("", "teh", ""), 8, true).first().text)
    }

    @Test
    fun predictsTheUsersHabitsFirst() {
        learn("later", "you")
        learn("there", "you", times = 1)
        learn("You", "thank", times = 1)
        val words = learningSuggester.suggest(TypingState.of("see you ", "", ""), 8, true).map { it.text }
        assertEquals(listOf("later", "you", "God", "there"), words)
        // One spelling per word: the dictionary's "you" stands for the learned "You".
        assertEquals(listOf("you", "God"), learningSuggester.suggest(TypingState.of("thank ", "", ""), 8, true).map { it.text })
    }

    @Test
    fun passesWhereEachLetterWasTappedToTheDecoder() {
        val taps = listOf(TapOffset(0f, 0f), null, TapOffset(0.45f, -0.1f))
        suggester.suggest(TypingState.of("", "teh", "").copy(taps = taps), 8, true)
        assertEquals(taps, decoder.lastTaps)
    }

    /** Decoder results by input; probabilities by word; n-grams as (previous word, word) pairs. */
    private class FakeDecoder(
        private val results: Map<String, List<RawSuggestion>>,
        private val probabilities: Map<String, Int>,
        private val ngrams: Set<Pair<String, String>>,
    ) : TypingDecoder {
        /** The taps the last [decode] was given. */
        var lastTaps: List<TapOffset?> = emptyList()

        override fun decode(input: String, previous: PreviousWords, blockOffensiveWords: Boolean, taps: List<TapOffset?>): List<RawSuggestion> {
            lastTaps = taps
            return results[input].orEmpty().sortedByDescending { it.score }
        }

        override fun probability(word: String) = probabilities[word] ?: NOT_A_PROBABILITY

        override fun ngramProbability(previous: PreviousWords, word: String): Int {
            val previousWord = previous.words.firstOrNull() ?: return NOT_A_PROBABILITY
            return if (previousWord to word in ngrams) 194 else NOT_A_PROBABILITY
        }
    }
}
