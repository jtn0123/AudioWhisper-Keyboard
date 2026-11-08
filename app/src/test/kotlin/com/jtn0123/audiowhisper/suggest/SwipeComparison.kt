package com.jtn0123.audiowhisper.suggest

import kotlin.math.abs
import kotlin.random.Random

/**
 * Holds swipe typing's ranking and word list in Rust (lib/rust/crates/suggest, behind
 * [GlideRanker] and [SwipeWords]) up against the Kotlin they replaced, [ReferenceSwipe], wherever
 * this runs: every bonus a context can give over wide ranges of counts and probabilities, random
 * guesses ranked in random contexts, and random dictionaries with the user's words. Floats are
 * compared by their bits. Self-contained, so a device check can copy it into the app and run it on
 * a phone, whose Java judges some characters differently from a computer's.
 *
 * [exactMath] says whether a frequency, which comes from `exp`, must match to the bit. On a phone
 * Java's `Math.exp` and Rust's are both bionic's, so it must. On a computer the JVM has its own,
 * which can differ from the system's in the last bit; there a frequency may be one unit in the last
 * place off, as long as the Float the glide classifier takes of it is the same.
 */
class SwipeComparison(private val exactMath: Boolean) {
    /** How many comparisons failed, and the first few, described. */
    var failures = 0
        private set
    val examples = mutableListOf<String>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    private fun check(what: String, input: Any?, kotlin: Any?, rust: Any?) = record(what, input, kotlin == rust, kotlin, rust)

    /** A frequency: see [exactMath]. */
    private fun checkFrequency(what: String, input: Any?, kotlin: Double, rust: Double) {
        val (kotlinBits, rustBits) = kotlin.toRawBits() to rust.toRawBits()
        val same = kotlinBits == rustBits || !exactMath && abs(kotlinBits - rustBits) == 1L &&
            kotlin.toFloat().toRawBits() == rust.toFloat().toRawBits()
        record(what, input, same, kotlin, rust)
    }

    private fun record(what: String, input: Any?, same: Boolean, kotlin: Any?, rust: Any?) {
        checks++
        if (same) return
        failures++
        if (examples.size < MAX_EXAMPLES) examples += "$what($input): Kotlin $kotlin, Rust $rust"
    }

    /** A history that knows one count for every pair, and a dictionary one n-gram for every word. */
    private class Fixed : TypingHistory, WordLookup {
        var pairCount = 0
        var ngram = NOT_A_PROBABILITY
        var probability = NOT_A_PROBABILITY

        override fun spellingOf(word: String): String? = null
        override fun completions(typed: String, maxCount: Int, accept: (String) -> Boolean) = emptyList<String>()
        override fun followers(previous: PreviousWords, minCount: Int, maxCount: Int) = emptyList<String>()
        override fun pairCount(previous: PreviousWords, word: String) = pairCount
        override fun probability(word: String) = probability
        override fun ngramProbability(previous: PreviousWords, word: String) = ngram
    }

    /** A history and a dictionary that know each word's numbers, and nothing without a previous word. */
    private class ByWord(
        val pairCounts: Map<String, Int>,
        val ngrams: Map<String, Int>,
        val probabilities: Map<String, Int>,
    ) : TypingHistory, WordLookup {
        override fun spellingOf(word: String): String? = null
        override fun completions(typed: String, maxCount: Int, accept: (String) -> Boolean) = emptyList<String>()
        override fun followers(previous: PreviousWords, minCount: Int, maxCount: Int) = emptyList<String>()
        override fun pairCount(previous: PreviousWords, word: String) = pairCounts[word] ?: 0
        override fun probability(word: String) = probabilities[word] ?: NOT_A_PROBABILITY
        override fun ngramProbability(previous: PreviousWords, word: String) =
            if (previous.words.isEmpty()) NOT_A_PROBABILITY else ngrams[word] ?: NOT_A_PROBABILITY
    }

    /**
     * Every bonus for pair counts from 0 to [maxCount] and the Int extremes, for every n-gram and
     * probability between -[span] and 2 x [span], and [randomCount] random mixes of the three.
     */
    fun bonuses(random: Random, maxCount: Int, span: Int, randomCount: Int) {
        val fixed = Fixed()
        val reference = ReferenceSwipe.GlideRanker(fixed, fixed)
        val rust = GlideRanker(fixed, fixed)
        val previous = PreviousWords.from("me ")
        fun compare() = check(
            "bonus",
            "${fixed.pairCount}, ${fixed.ngram}, ${fixed.probability}",
            reference.bonus("word", previous).toRawBits(),
            rust.bonus("word", previous).toRawBits(),
        )
        for (count in (0..maxCount) + EXTREMES) {
            fixed.pairCount = count
            compare()
        }
        fixed.pairCount = 0
        for (ngram in (-span..2 * span) + EXTREMES) {
            fixed.ngram = ngram
            for (probability in (-span..2 * span) + EXTREMES) {
                fixed.probability = probability
                compare()
            }
        }
        repeat(randomCount) {
            fixed.pairCount = if (random.nextBoolean()) random.nextInt(-5, 50) else random.nextInt()
            fixed.ngram = if (random.nextBoolean()) random.nextInt(-2, 300) else random.nextInt()
            fixed.probability = if (random.nextBoolean()) random.nextInt(-2, 300) else random.nextInt()
            compare()
        }
    }

    /**
     * [count] random lists of guesses ranked in random contexts, with and without a dictionary.
     * Now and then a list longer than the classifier gives (8), where a sort that isn't stable
     * would show: Rust's unstable sort is an insertion sort up to 20.
     */
    fun rankings(random: Random, count: Int) = repeat(count) {
        val size = if (random.nextInt(10) == 0) random.nextInt(21, 60) else random.nextInt(0, 9)
        val words = List(size) { WORDS.random(random) }
        val guesses = words.map { GlideGuess(it, cost(random)) }
        val known = ByWord(
            pairCounts = WORDS.associateWith { if (random.nextInt(3) == 0) random.nextInt(0, 30) else 0 },
            ngrams = WORDS.associateWith { if (random.nextInt(3) == 0) random.nextInt(-1, 256) else NOT_A_PROBABILITY },
            probabilities = WORDS.associateWith { if (random.nextInt(4) == 0) NOT_A_PROBABILITY else random.nextInt(0, 256) },
        )
        val lookup = if (random.nextInt(4) == 0) null else known
        val history = if (random.nextInt(4) == 0) TypingHistory.NONE else known
        val previous = PreviousWords.from(CONTEXTS.random(random))
        check(
            "rank",
            guesses.joinToString { "${it.word} ${it.cost}" } + " after ${previous.words}",
            ReferenceSwipe.GlideRanker(history, lookup).rank(guesses, previous),
            GlideRanker(history, lookup).rank(guesses, previous),
        )
    }

    /** A guess's cost: usually a clean swipe's, sometimes a tie or an odd float. */
    private fun cost(random: Random): Float = when (random.nextInt(10)) {
        0 -> ODD_COSTS.random(random)
        1, 2 -> 3f + random.nextInt(0, 4) * 0.25f
        else -> 2.5f + random.nextFloat() * 3f
    }

    /** Every frequency a probability can give, and the Int extremes. */
    fun frequencies() {
        for (probability in (-1000..1000) + EXTREMES) {
            checkFrequency(
                "frequencyOf",
                probability,
                ReferenceSwipe.SwipeWords.frequencyOf(probability),
                SwipeWords.frequencyOf(probability),
            )
        }
    }

    /** Every UTF-16 unit alone and beside a letter or a connector, and [count] random strings. */
    fun swipeable(random: Random, count: Int) {
        fun compare(word: String) = check(
            "isSwipeable",
            hex(word),
            ReferenceSwipe.SwipeWords.isSwipeable(word),
            SwipeWords.isSwipeable(word),
        )
        for (unit in 0..0xFFFF) {
            val char = unit.toChar().toString()
            for (word in listOf(char, "a$char", "${char}b", "'$char", "$char-")) compare(word)
        }
        repeat(count) { compare(text(random)) }
    }

    /**
     * [runs] random dictionaries with random user words: which words a swipe can produce, the
     * user's words in order, and every word's frequency.
     */
    fun wordLists(random: Random, runs: Int) = repeat(runs) {
        val dictionary = List(random.nextInt(0, 40)) { text(random) }
        val probabilities = (dictionary + WORDS).associateWith { probability(random) }
        val personal = List(random.nextInt(0, 6)) {
            PersonalWords.Entry(userWord(random, dictionary), random.nextInt(-50, 400))
        }
        val learned = List(random.nextInt(0, 6)) { userWord(random, dictionary) }
        val learnedCollection: Collection<String> = if (random.nextBoolean()) learned else learned.toCollection(LinkedHashSet())
        val lookup = { word: String -> probabilities[word] ?: NOT_A_PROBABILITY }
        val input = "${dictionary.size} words, personal $personal, learned $learned"
        val referenceUser = ReferenceSwipe.SwipeWords.userWords(personal, learnedCollection)
        val rustUser = SwipeWords.userWords(personal, learnedCollection)
        check("userWords", input, referenceUser.toList(), rustUser.toList())
        val referenceWords = ReferenceSwipe.SwipeWords.fromDictionary(dictionary, lookup)
        val rustWords = SwipeWords.fromDictionary(dictionary, lookup)
        check("fromDictionary", input, referenceWords.words, rustWords.words)
        val referenceAll = referenceWords.withUserWords(referenceUser)
        val rustAll = rustWords.withUserWords(rustUser)
        check("words", input, referenceAll.words, rustAll.words)
        check("same", input, referenceAll.withUserWords(referenceUser) === referenceAll, rustAll.withUserWords(rustUser) === rustAll)
        for (word in dictionary + WORDS + learned + personal.map { it.word } + "unknown") {
            checkFrequency("frequency", "$word in $input", referenceAll.frequency(word), rustAll.frequency(word))
        }
    }

    private fun probability(random: Random): Int = when (random.nextInt(6)) {
        0 -> NOT_A_PROBABILITY
        1 -> random.nextInt(-300, 600)
        else -> random.nextInt(0, 256)
    }

    /** A word for the user: one of the dictionary's or a known word, maybe recased, or random. */
    private fun userWord(random: Random, dictionary: List<String>): String = when (random.nextInt(4)) {
        0 -> if (dictionary.isEmpty()) text(random) else dictionary.random(random)
        1 -> WORDS.random(random).replaceFirstChar { it.uppercaseChar() }
        2 -> WORDS.random(random)
        else -> text(random)
    }

    private fun text(random: Random): String = buildString {
        repeat(random.nextInt(0, 6)) { append(PIECES.random(random)) }
    }

    private fun hex(text: String) = text.map { "%04X".format(it.code) }.joinToString(" ")

    private companion object {
        const val MAX_EXAMPLES = 20

        val EXTREMES = listOf(Int.MIN_VALUE, Int.MIN_VALUE + 1, -1, 16_777_216, 16_777_217, Int.MAX_VALUE - 1, Int.MAX_VALUE)
        val WORDS = listOf("to", "too", "top", "of", "off", "odd", "you", "thou", "Tupi", "an", "am", "Sam", "don't", "e-mail")
        val CONTEXTS = listOf("", "me ", "I ", "turn ", "thank ", "Hello. ", "going ", "Hi, ")
        val ODD_COSTS = listOf(Float.NaN, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, 0f, -0f, Float.MIN_VALUE, Float.MAX_VALUE, 3.28f)
        val PIECES = listOf(
            "a", "b", "z", "é", "ß", "Ω", "я", "ん", "中", "ﬁ", "ǅ", "ª", "ʰ", "́", "ः",
            "'", "’", "-", "‐", " ", "\t", "1", "٣", "$", "@", ".", "𝐀", "\uD800", "\uDC00", "\u0000",
        )
    }
}
