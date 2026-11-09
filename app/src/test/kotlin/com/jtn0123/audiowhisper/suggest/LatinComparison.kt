package com.jtn0123.audiowhisper.suggest

import java.util.Locale
import kotlin.random.Random

/**
 * Holds the suggester and the word learner in Rust (lib/rust/crates/suggest, behind
 * [LatinSuggester] and [WordLearner]) up against the Kotlin they replaced, [ReferenceLatin],
 * wherever this runs. Both ask the same made-up dictionary ([Fake]), whose every answer is a pure
 * function of the question, so how often and in what order each asks doesn't matter, only what
 * they make of the answers: random strips and spell checks over random personal and learned
 * words, and random runs of finished, swiped and undone words on a pair of learners, comparing
 * what each stored. Self-contained, so a device check can copy it into the app and run it on a
 * phone, whose Java judges some characters differently from a computer's.
 *
 * [sigmaExact] says whether words may hold Σ, which lower-cases to ς at the end of a word and to σ
 * elsewhere. Rust decides that by ICU's rule, as Android does; the JVM uses its own, so off a
 * phone Σ is left out.
 */
class LatinComparison(private val sigmaExact: Boolean) {
    /** How many comparisons failed, and the first few, described. */
    var failures = 0
        private set
    val examples = mutableListOf<String>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    /** How often each outcome worth reaching was reached, so a run that tests nothing shows. */
    val seen = sortedMapOf<String, Int>()

    private val pieces = if (sigmaExact) PIECES else PIECES - "$CAPITAL_SIGMA"
    private val letters = if (sigmaExact) LETTERS else LETTERS - "$CAPITAL_SIGMA"

    private fun check(what: String, input: String, kotlin: Any?, rust: Any?) {
        checks++
        if (kotlin == rust) return
        failures++
        if (examples.size < MAX_EXAMPLES) examples += "$what($input): Kotlin ${show(kotlin)}, Rust ${show(rust)}"
    }

    private fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    /** What [body] gives, or the exception it throws, so both sides must throw alike. */
    private inline fun outcome(body: () -> Any?): Any? = try {
        body()
    } catch (e: IllegalArgumentException) {
        "threw ${e.javaClass.name}: ${e.message}"
    }

    /**
     * A made-up dictionary over [vocabulary]: each answer is drawn from a random generator seeded
     * with [seed] and the question, so asking again gives the same answer. One word in
     * [unknownOneIn] is unknown.
     */
    private inner class Fake(
        private val seed: Long,
        private val vocabulary: List<String>,
        private val unknownOneIn: Int,
    ) : TypingDecoder {
        override fun probability(word: String): Int {
            val random = Random(hash("p", word))
            return if (random.nextInt(unknownOneIn) == 0) NOT_A_PROBABILITY else PROBABILITIES.random(random)
        }

        override fun ngramProbability(previous: PreviousWords, word: String): Int {
            val random = Random(hash("n", "$previous", word))
            return if (random.nextInt(3) == 0) NOT_A_PROBABILITY else PROBABILITIES.random(random)
        }

        override fun decode(
            input: String,
            previous: PreviousWords,
            blockOffensiveWords: Boolean,
            taps: List<TapOffset?>,
        ): List<RawSuggestion> {
            val random = Random(hash("d", input, "$previous", "$blockOffensiveWords", "$taps"))
            val results = List(random.nextInt(9)) {
                RawSuggestion(random.result(input, vocabulary), random.score(), random.kindAndFlags())
            }
            // The decoder sorts by score; the rules shouldn't depend on it.
            return if (random.nextInt(4) == 0) results else results.sortedByDescending { it.score }
        }

        /** FNV-1a over [parts], each ended by a unit no string here holds, from [seed]. */
        private fun hash(vararg parts: String): Long {
            var hash = seed xor FNV_OFFSET
            for (part in parts) {
                for (unit in part + '￿') hash = (hash xor unit.code.toLong()) * FNV_PRIME
            }
            return hash
        }
    }

    /** [count] random requests for a strip and a spell check, made of each suggester. */
    fun suggester(random: Random, count: Int) = repeat(count) {
        val vocabulary = random.vocabulary()
        val decoder = Fake(random.nextLong(), vocabulary, random.nextInt(2, 6))
        val state = random.state(vocabulary)
        val personal = random.personal(vocabulary)
        val learned = random.learned(vocabulary, state.previousWords)
        val threshold = THRESHOLDS.random(random)
        val reference = ReferenceLatin.LatinSuggester(
            decoder, Locale.US, threshold, { personal }, { learned ?: TypingHistory.NONE },
        )
        val rust = LatinSuggester(decoder, threshold, { personal }, { learned })
        val maxCount = random.maxCount()
        val block = random.nextBoolean()
        val input = describe(state, maxCount, threshold, learned != null)
        val strip = outcome { reference.suggest(state, maxCount, block) }
        check("suggest", input, strip, outcome { rust.suggest(state, maxCount, block) })
        noteStrip(state, strip)
        val word = if (random.nextInt(8) == 0) random.text(3) else state.typed
        val spelled = outcome { reference.spell(word, maxCount, block) }
        check("spell ${hex(word)}", input, spelled, outcome { rust.spell(word, maxCount, block) })
        noteSpell(spelled)
    }

    private fun noteStrip(state: TypingState, strip: Any?) {
        val words = strip as? List<*> ?: return see("strip threw")
        val arranged = words.filterIsInstance<ArrangedWord>()
        if (state.typed.isEmpty()) return see("predictions", arranged.isNotEmpty())
        see("auto-correction", arranged.any { it.isAutoCorrection })
        see("no auto-correction", arranged.isNotEmpty() && arranged.none { it.isAutoCorrection })
        see("more than the typed word", arranged.size > 2)
    }

    private fun noteSpell(spelled: Any?) {
        val check = spelled as? SpellCheck ?: return see("spell threw")
        see(if (check.isValid) "valid" else "misspelled")
        see("confident", check.isHighConfidence)
    }

    /** [runs] runs of [steps] random steps each on two learners, each with its own store. */
    fun learner(random: Random, runs: Int, steps: Int) = repeat(runs) {
        val vocabulary = random.vocabulary()
        val decoder = Fake(random.nextLong(), vocabulary, random.nextInt(2, 6))
        val (words, pairs) = CAPACITIES.random(random) to CAPACITIES.random(random)
        val stores = LearnedWords(null, words, pairs) to LearnedWords(null, words, pairs)
        var lookup: WordLookup? = decoder
        val learners = ReferenceLatin.WordLearner(stores.first, { lookup }) to WordLearner(stores.second, { lookup })
        repeat(steps) {
            // The dictionary closes and opens.
            if (random.nextInt(10) == 0) lookup = if (lookup == null) decoder else null
            learnerStep(random, learners, vocabulary)
            check("saved", "", stores.first.store.savedText(), stores.second.store.savedText())
        }
    }

    private fun learnerStep(random: Random, learners: Pair<ReferenceLatin.WordLearner, WordLearner>, vocabulary: List<String>) {
        val (reference, rust) = learners
        val word = random.learnable(vocabulary)
        val textBefore = random.textBefore(vocabulary)
        val start = STARTS.random(random)
        val input = "${hex(word)}|${hex(textBefore)}|$start"
        when (random.nextInt(20)) {
            in 0..9 -> {
                val changed = reference.finished(word, textBefore, start)
                check("finished", input, changed, rust.finished(word, textBefore, start))
                see("finished", changed)
            }
            in 10..13 -> {
                val changed = reference.swiped(word, textBefore)
                check("swiped", input, changed, rust.swiped(word, textBefore))
                see("swiped kept the last", changed)
            }
            14 -> {
                reference.dropSwiped()
                rust.dropSwiped()
            }
            in 15..17 -> {
                val changed = reference.undo(start)
                check("undo", "$start", changed, rust.undo(start))
                see("undone", changed)
            }
            else -> if (random.nextInt(3) == 0) {
                reference.reset()
                rust.reset()
            }
        }
    }

    private fun describe(state: TypingState, maxCount: Int, threshold: Float, learning: Boolean): String {
        val previous = state.previousWords.words.joinToString(",") { hex(it) }
        return "${hex(state.typed)}|[$previous]/${state.previousWords.isBeginningOfSentence}|" +
            "mid ${state.isCursorMidWord}|taps ${state.taps.size}|max $maxCount|threshold $threshold|learning $learning"
    }

    /** Σ, which [sigmaExact] may leave out, made σ; upper-casing σ or ς makes one. */
    private fun String.sigmaFree() = if (sigmaExact) this else replace(CAPITAL_SIGMA, 'σ')

    private fun Random.text(maxPieces: Int) = List(nextInt(1, maxPieces + 1)) { pieces.random(this) }.joinToString("")

    /** A request's words: a few short stems, each continued, so they complete and spell each other. */
    private fun Random.vocabulary(): List<String> {
        val stems = List(3) { part(nextInt(1, 3)) }
        return List(10) { stems.random(this) + part(nextInt(4)) }
    }

    /** [count] letters, now and then a piece of something else. */
    private fun Random.part(count: Int) =
        List(count) { if (nextInt(8) == 0) pieces.random(this) else letters.random(this) }.joinToString("")

    /** One of the request's words, as it is or in another case, or now and then another. */
    private fun Random.word(vocabulary: List<String>): String {
        val word = vocabulary.random(this)
        val cased = when (nextInt(10)) {
            0 -> word.uppercase()
            1 -> word.lowercase()
            2 -> word.replaceFirstChar { it.titlecaseChar() }
            3 -> text(3)
            else -> word
        }
        return cased.sigmaFree()
    }

    /** What is being typed: nothing (to predict), the start of a word, or anything. */
    private fun Random.typed(vocabulary: List<String>): String = when (nextInt(12)) {
        0, 1 -> ""
        2 -> text(3)
        3 -> word(vocabulary) + DIGITS.random(this)
        else -> word(vocabulary).let { it.take(nextInt(1, it.length + 1)) }
    }

    private fun Random.state(vocabulary: List<String>): TypingState {
        val typed = typed(vocabulary)
        val previous = PreviousWords(List(nextInt(4)) { word(vocabulary) }, nextBoolean())
        val taps = if (nextInt(3) == 0) {
            emptyList()
        } else {
            List(typed.codePointCount(0, typed.length)) { if (nextInt(4) == 0) null else TapOffset(nextFloat() - 0.5f, nextFloat()) }
        }
        return TypingState(typed, previous, isCursorMidWord = nextInt(5) == 0, taps)
    }

    /** A decoder result for [input]: itself, its acronym, a completion, a correction, or nothing. */
    private fun Random.result(input: String, vocabulary: List<String>): String = when (nextInt(12)) {
        0 -> input
        1 -> input.uppercase().sigmaFree()
        2 -> input.replaceFirstChar { it.titlecaseChar() }.sigmaFree()
        3, 4 -> input + text(2)
        5 -> (input + letters.random(this)).uppercase().sigmaFree()
        6 -> input.dropLast(1) + letters.random(this)
        7 -> ""
        else -> word(vocabulary)
    }

    private fun Random.score() = if (nextInt(4) == 0) SCORES.random(this) else nextInt(0, 2_000_000)

    private fun Random.kindAndFlags(): Int {
        val appropriate = if (nextInt(3) == 0) 0 else RawSuggestion.FLAG_APPROPRIATE_FOR_AUTO_CORRECTION
        val exact = if (nextInt(5) == 0) RawSuggestion.FLAG_EXACT_MATCH else 0
        return KINDS.random(this) or appropriate or exact
    }

    private fun Random.maxCount() = if (nextInt(10) == 0) nextInt(-2, 1) else nextInt(1, 9)

    private fun Random.personal(vocabulary: List<String>): PersonalWords {
        if (nextInt(4) == 0) return PersonalWords.NONE
        return PersonalWords(
            List(nextInt(1, 6)) {
                val shortcut = if (nextInt(3) == 0) word(vocabulary) else null
                PersonalWords.Entry(word(vocabulary), FREQUENCIES.random(this), shortcut)
            },
        )
    }

    /** Learned words, mostly after the request's previous word, or null when not learning. */
    private fun Random.learned(vocabulary: List<String>, previous: PreviousWords): LearnedWords? {
        if (nextInt(4) == 0) return null
        val learned = LearnedWords(null, CAPACITIES.random(this), CAPACITIES.random(this))
        val here = LearnedWords.previousKey(previous)
        repeat(nextInt(40)) {
            val after = when (nextInt(5)) {
                0 -> null
                1 -> LearnedWords.SENTENCE_START
                2 -> word(vocabulary)
                else -> here
            }
            learned.add(word(vocabulary), after, keepSpelling = nextInt(4) == 0)
        }
        return learned
    }

    /** A finished word: mostly one of the run's, now and then one that can't be stored. */
    private fun Random.learnable(vocabulary: List<String>): String = when (nextInt(20)) {
        0 -> text(3)
        1 -> "a".repeat(nextInt(45, 52))
        else -> word(vocabulary)
    }

    private fun Random.textBefore(vocabulary: List<String>): String =
        List(nextInt(4)) { word(vocabulary) + SEPARATORS.random(this) }.joinToString("")

    private companion object {
        const val MAX_EXAMPLES = 20
        const val CAPITAL_SIGMA = 'Σ'
        const val FNV_OFFSET = -0x340d631b7bdddcdbL
        const val FNV_PRIME = 0x100000001b3L

        val LETTERS = listOf(
            "w", "o", "r", "k", "d", "a", "e", "W", "O", "R", "K", "D", "A", "E",
            "é", "É", "ß", "σ", "ς", "Σ", "ı", "İ", "ǅ", "K",
        )

        /** Pieces of words, among them what a word can't hold and what the JVM and Android judge apart. */
        val PIECES = listOf(
            "a", "B", "é", "ß", "ẞ", "ǅ", "Σ", "σ", "ς", "ı", "İ", "i", "I", "K", "ª", "ʰ", "ʼ", "ﬁ", "Ⓐ",
            "'", "’", "-", "1", "٣", "²", " ", " ", "\t", "\n", "　", "\u0085", "᠎", "😀", "𐐀", "\uD800",
            "\uDC00", "́", ".", "x",
        )

        val DIGITS = listOf("1", "٣", "²")

        val SEPARATORS = listOf(" ", " ", ". ", ", ", "! ", "\n", "", "'", " - ", "? ")

        /**
         * On the 0-255 log scale, many exactly [ReferenceLatin.LatinSuggester.RARER_BY] apart, and
         * some beyond it, so subtracting them overflows.
         */
        val PROBABILITIES = listOf(0, 1, 50, 51, 100, 101, 150, 200, 205, 255, -5, 1000, Int.MAX_VALUE, Int.MIN_VALUE)

        val SCORES = listOf(0, 1, -1, 100_000, 1_000_000, 2_000_000, Int.MAX_VALUE, Int.MIN_VALUE)

        val KINDS = with(RawSuggestion) {
            listOf(KIND_CORRECTION, KIND_CORRECTION, KIND_CORRECTION, KIND_WHITELIST, KIND_SHORTCUT, KIND_PREDICTION, 0, 2, 0xFF)
        }

        val THRESHOLDS = listOf(
            AutoCorrection.MODEST_THRESHOLD, AutoCorrection.MODEST_THRESHOLD, 0f, 0.067f, 0.185f, 1f, 2f, -1f,
            Float.NaN, Float.POSITIVE_INFINITY,
        )

        val FREQUENCIES = listOf(0, 1, 100, 200, 255, -1, Int.MAX_VALUE, Int.MIN_VALUE)

        /** Small, so the oldest go, and none, and the keyboard's own. */
        val CAPACITIES = listOf(0, 1, 2, 3, 5, 8, -1, 5000)

        /** Where finished words start: often the same place, so one replaces another. */
        val STARTS = listOf(-1, 0, 0, 3, 3, 7, 12)

        private fun hex(text: String) = text.map { "%04X".format(it.code) }.joinToString(" ")

        private fun show(value: Any?): String = when (value) {
            is String -> "\"${hex(value)}\""
            is List<*> -> value.joinToString(prefix = "[", postfix = "]") { show(it) }
            is ArrangedWord -> "${show(value.text)}${if (value.isAutoCorrection) "*" else ""}${if (value.isTypedWord) "^" else ""}"
            is SpellCheck -> "valid ${value.isValid} ${show(value.suggestions)} confident ${value.isHighConfidence}"
            else -> value.toString()
        }
    }
}
