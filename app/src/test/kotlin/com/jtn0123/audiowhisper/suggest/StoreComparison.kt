package com.jtn0123.audiowhisper.suggest

import java.io.File
import kotlin.random.Random

/**
 * Holds the word stores in Rust (lib/rust/crates/suggest, behind [LearnedWords], [KeptWords] and
 * [PersonalWords]) up against the Kotlin they replaced, [ReferenceStores], wherever this runs:
 * random runs of learning, keeping, saving and loading on a pair of stores, the same steps on
 * each, asking both the same questions after every step. Self-contained, so a device check can
 * copy it into the app and run it on a phone, whose Java judges some characters differently from
 * a computer's. The stores save to files in [dir].
 *
 * [sigmaExact] says whether words may hold Σ, which lower-cases to ς at the end of a word and to σ
 * elsewhere. Rust decides that by ICU's rule, as Android does; the JVM uses its own, so off a
 * phone Σ is left out.
 */
class StoreComparison(private val sigmaExact: Boolean, private val dir: File) {
    /** How many comparisons failed, and the first few, described. */
    var failures = 0
        private set
    val examples = mutableListOf<String>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    private val pieces = if (sigmaExact) PIECES else PIECES - "$CAPITAL_SIGMA"

    private fun check(what: String, input: String, kotlin: Any?, rust: Any?) {
        checks++
        if (kotlin == rust) return
        failures++
        if (examples.size < MAX_EXAMPLES) examples += "$what(${hex(input)}): Kotlin ${show(kotlin)}, Rust ${show(rust)}"
    }

    /** What [body] gives, or the exception it throws, so both sides must throw alike. */
    private inline fun outcome(body: () -> Any?): Any? = try {
        body()
    } catch (e: IllegalArgumentException) {
        "threw ${e.javaClass.name}: ${e.message}"
    }

    /** Two stores of learned words given the same steps, each saving to its own file. */
    private class Learned(
        val reference: ReferenceStores.LearnedWords,
        val rust: LearnedWords,
        val referenceFile: File,
        val rustFile: File,
        val vocabulary: List<String>,
    )

    /** [runs] runs of [steps] random steps each on two stores of learned words. */
    fun learned(random: Random, runs: Int, steps: Int) = repeat(runs) {
        val (words, pairs) = CAPACITIES.random(random) to CAPACITIES.random(random)
        val referenceFile = File(dir, "reference-learned.txt").apply { delete() }
        val rustFile = File(dir, "rust-learned.txt").apply { delete() }
        val stores = Learned(
            ReferenceStores.LearnedWords(referenceFile, words, pairs),
            LearnedWords(rustFile, words, pairs),
            referenceFile,
            rustFile,
            random.vocabulary(),
        )
        repeat(steps) {
            learnedStep(random, stores)
            askLearned(random, stores)
        }
    }

    private fun learnedStep(random: Random, stores: Learned) {
        val word = random.word(stores.vocabulary)
        val previous = random.previous(stores.vocabulary)
        when (random.nextInt(20)) {
            in 0..9 -> {
                val keepSpelling = random.nextBoolean()
                stores.reference.add(word, previous, keepSpelling)
                stores.rust.add(word, previous, keepSpelling)
            }
            in 10..12 -> {
                stores.reference.remove(word, previous)
                stores.rust.remove(word, previous)
            }
            13 -> if (random.nextInt(8) == 0) {
                stores.reference.clear()
                stores.rust.clear()
            }
            14 -> compareSaved("learned", stores.reference.save(), stores.rust.save(), stores.referenceFile, stores.rustFile)
            15 -> {
                loadBoth(random, stores.referenceFile, stores.rustFile) { random.learnedText(stores.vocabulary) }
                stores.reference.load()
                stores.rust.load()
            }
        }
    }

    private fun askLearned(random: Random, stores: Learned) {
        val (reference, rust) = stores.reference to stores.rust
        check("knownSpellings", "", reference.knownSpellings(), rust.knownSpellings())
        val word = random.word(stores.vocabulary)
        check("spellingOf", word, reference.spellingOf(word), rust.spellingOf(word))
        val typed = random.typed(stores.vocabulary)
        val maxCount = random.nextInt(-1, 5)
        val accept = ACCEPTS.random(random)
        check(
            "completions $maxCount",
            typed,
            reference.completions(typed, maxCount, accept),
            rust.completions(typed, maxCount, accept),
        )
        val previous = random.previousWords(stores.vocabulary)
        val (minCount, mostCount) = random.nextInt(-1, 4) to random.nextInt(-2, 5)
        val input = "${previous.words.firstOrNull()}|${previous.isBeginningOfSentence}|$minCount|$mostCount"
        check(
            "followers",
            input,
            outcome { reference.followers(previous, minCount, mostCount) },
            outcome { rust.followers(previous, minCount, mostCount) },
        )
        check("pairCount", "$input|$word", reference.pairCount(previous, word), rust.pairCount(previous, word))
    }

    /** Two stores of kept words given the same steps, each saving to its own file. */
    private class Kept(
        val reference: ReferenceStores.KeptWords,
        val rust: KeptWords,
        val referenceFile: File,
        val rustFile: File,
        val vocabulary: List<String>,
    )

    /** [runs] runs of [steps] random steps each on two stores of kept words. */
    fun kept(random: Random, runs: Int, steps: Int) = repeat(runs) {
        val capacity = CAPACITIES.random(random)
        val referenceFile = File(dir, "reference-kept.txt").apply { delete() }
        val rustFile = File(dir, "rust-kept.txt").apply { delete() }
        val stores = Kept(
            ReferenceStores.KeptWords(referenceFile, capacity),
            KeptWords(rustFile, capacity),
            referenceFile,
            rustFile,
            random.vocabulary(),
        )
        repeat(steps) {
            keptStep(random, stores)
            for (word in stores.vocabulary + random.word(stores.vocabulary)) {
                check("contains", word, word in stores.reference, word in stores.rust)
            }
        }
    }

    private fun keptStep(random: Random, stores: Kept) {
        val word = random.word(stores.vocabulary)
        when (random.nextInt(10)) {
            in 0..6 -> {
                val remember = random.nextInt(4) != 0
                check("add $remember", word, stores.reference.add(word, remember), stores.rust.store.add(word, remember))
            }
            7 -> compareSaved("kept", stores.reference.save(), stores.rust.save(), stores.referenceFile, stores.rustFile)
            8 -> {
                loadBoth(random, stores.referenceFile, stores.rustFile) { random.keptText(stores.vocabulary) }
                stores.reference.load()
                stores.rust.load()
            }
        }
    }

    /** Both saved alike, and wrote the same bytes. */
    private fun compareSaved(what: String, reference: Boolean, rust: Boolean, referenceFile: File, rustFile: File) {
        check("$what save", "", reference, rust)
        val written = listOf(referenceFile, rustFile).map { if (it.isFile) it.readText() else null }
        check("$what saved file", "", written[0], written[1])
    }

    /** Writes the same file for both to load: [text], damaged now and then, or none at all. */
    private fun loadBoth(random: Random, referenceFile: File, rustFile: File, text: () -> String) {
        if (random.nextInt(10) == 0) {
            referenceFile.delete()
            rustFile.delete()
            return
        }
        val bytes = text().toByteArray(Charsets.UTF_8)
        if (random.nextInt(4) == 0 && bytes.isNotEmpty()) {
            repeat(random.nextInt(1, 4)) { bytes[random.nextInt(bytes.size)] = BAD_BYTES.random(random) }
        }
        referenceFile.writeBytes(bytes)
        rustFile.writeBytes(bytes)
    }

    /** [count] random personal dictionaries, asked the same questions. */
    fun personal(random: Random, count: Int) = repeat(count) {
        val vocabulary = random.vocabulary()
        val entries = List(random.nextInt(9)) {
            val word = if (random.nextInt(4) == 0) random.text(4) else random.word(vocabulary)
            val shortcut = if (random.nextInt(3) == 0) random.word(vocabulary) else null
            PersonalWords.Entry(word, FREQUENCIES.random(random), shortcut)
        }
        val reference = ReferenceStores.PersonalWords(entries.map { ReferenceStores.PersonalWords.Entry(it.word, it.frequency, it.shortcut) })
        val rust = PersonalWords(entries)
        val input = entries.joinToString("|") { "${it.word}/${it.frequency}/${it.shortcut}" }
        check("entries", input, reference.entries.map { listOf(it.word, it.frequency, it.shortcut) }, rust.entries.map { listOf(it.word, it.frequency, it.shortcut) })
        check("isEmpty", input, reference.isEmpty, rust.isEmpty)
        repeat(4) {
            val word = random.word(vocabulary)
            check("spellingOf", "$input|$word", reference.spellingOf(word), rust.spellingOf(word))
            val typed = random.typed(vocabulary)
            val maxCount = random.nextInt(-1, 6)
            check("completions $maxCount", "$input|$typed", reference.completions(typed, maxCount), rust.completions(typed, maxCount))
        }
        val (tag, language) = TAGS.random(random) to LANGUAGES.random(random)
        check("appliesTo", "$tag|$language", ReferenceStores.PersonalWords.appliesTo(tag, language), PersonalWords.appliesTo(tag, language))
    }

    /** Every tag against every language. */
    fun languages() {
        for (tag in TAGS) {
            for (language in LANGUAGES) {
                check("appliesTo", "$tag|$language", ReferenceStores.PersonalWords.appliesTo(tag, language), PersonalWords.appliesTo(tag, language))
            }
        }
    }

    private fun Random.text(maxPieces: Int) = List(nextInt(1, maxPieces + 1)) { pieces.random(this) }.joinToString("")

    /** A run's words: mostly letters, so most can be learned, some at the length limit. */
    private fun Random.vocabulary() = List(12) {
        when (nextInt(12)) {
            0 -> "a".repeat(nextInt(45, 49)) + text(1)
            1 -> text(3) + " " + text(2)
            else -> List(nextInt(1, 4)) { if (nextInt(3) == 0) pieces.random(this) else LETTERS.random(this) }.joinToString("")
        }
    }

    /** One of the run's words, as it is or in another case, or now and then another. */
    private fun Random.word(vocabulary: List<String>): String {
        val word = vocabulary.random(this)
        val cased = when (nextInt(10)) {
            0 -> word.uppercase()
            1 -> word.lowercase()
            2 -> word.replaceFirstChar { it.titlecaseChar() }
            3 -> text(3)
            else -> word
        }
        // Upper-casing σ or ς makes a Σ.
        return if (sigmaExact) cased else cased.replace(CAPITAL_SIGMA, 'σ')
    }

    /** What a word was typed after: unknown, a sentence start, or a word. */
    private fun Random.previous(vocabulary: List<String>): String? = when (nextInt(4)) {
        0 -> null
        1 -> ReferenceStores.LearnedWords.SENTENCE_START
        else -> word(vocabulary)
    }

    private fun Random.previousWords(vocabulary: List<String>): PreviousWords {
        val words = List(nextInt(3)) { if (nextInt(10) == 0) "" else word(vocabulary) }
        return PreviousWords(words, nextBoolean())
    }

    /** The start of one of the run's words, in any case, or anything. */
    private fun Random.typed(vocabulary: List<String>): String {
        val word = word(vocabulary)
        return if (nextInt(6) == 0) text(2) else word.take(nextInt(word.length + 1))
    }

    private fun Random.learnedText(vocabulary: List<String>): String {
        val lines = List(nextInt(12)) {
            when (nextInt(7)) {
                0, 1 -> "w\t${COUNTS.random(this)}\t${word(vocabulary)}"
                2, 3 -> "p\t${COUNTS.random(this)}\t${previous(vocabulary) ?: ""}\t${word(vocabulary)}"
                4 -> "# AudioWhisper learned words, v1"
                5 -> List(nextInt(5)) { if (nextBoolean()) word(vocabulary) else COUNTS.random(this) }.joinToString("\t")
                else -> text(2)
            }
        }
        return lines.joinToString("") { it + LINE_BREAKS.random(this) }
    }

    private fun Random.keptText(vocabulary: List<String>): String {
        val lines = List(nextInt(12)) {
            when (nextInt(4)) {
                0 -> " ${word(vocabulary)}\t"
                1 -> text(2)
                else -> word(vocabulary)
            }
        }
        return lines.joinToString("") { it + LINE_BREAKS.random(this) }
    }

    private companion object {
        const val MAX_EXAMPLES = 20
        const val CAPITAL_SIGMA = 'Σ'

        /** Small, so the oldest go, and none, and the keyboard's own. */
        val CAPACITIES = listOf(0, 1, 2, 3, 5, 8, -1, 5000)

        val LETTERS = listOf("a", "b", "e", "r", "A", "B", "E", "R", "é", "É", "ß", "ǅ", "ǆ", "Ǆ", "ı", "İ", "K", "k", "σ", "ς")

        /**
         * Pieces of words: letters whose case is hard, connectors, letters a word can't hold
         * (digits, whitespace the JVM and Android disagree on, symbols, marks, surrogates), and the
         * file's own separators.
         */
        val PIECES = listOf(
            "a", "b", "A", "B", "é", "É", "ß", "ẞ", "ǅ", "Σ", "σ", "ς", "ı", "İ", "i", "I", "K", "k", "ª", "ʰ", "ʼ",
            "ﬁ", "Ⓐ", "'", "’", "-", "1", "٣", "²", " ", " ", "\t", "\n", "\r", "　", "\u0085", "᠎", "\u001C", "😀",
            "𐐀", "\uD800", "\uDC00", "́", "x",
        )

        val COUNTS = listOf(
            "1", "2", "3", "5", "0", "-1", "+2", "-", "٣", "٠٢", "07", "x", "", "2147483646", "2147483647",
            "2147483648", "-2147483648", " 2",
        )

        val LINE_BREAKS = listOf("\n", "\n", "\r\n", "\r", "\n\n", "")

        /** Bytes that make a UTF-8 file malformed. */
        val BAD_BYTES = listOf(0x80, 0xBF, 0xC0, 0xC3, 0xE2, 0xED, 0xF0, 0xF4, 0xF8, 0xFF, 0x00).map { it.toByte() }

        val ACCEPTS: List<(String) -> Boolean> = listOf({ true }, { false }, { it.length % 2 == 0 }, { !it.startsWith("a") })

        val FREQUENCIES = listOf(0, 1, 100, 100, 200, 255, 255, -1, Int.MAX_VALUE, Int.MIN_VALUE)

        val TAGS = listOf(null, "", "en", "EN", "En", "en_US", "en-GB", "en_", "de_DE", "_en", "-en", "e", "eN-us_x", "K", "ı", "İn")

        val LANGUAGES = listOf("en", "EN", "", "de", "k", "in", "i̇n")

        private fun hex(text: String) = text.map { "%04X".format(it.code) }.joinToString(" ")

        private fun show(value: Any?) = if (value is String) "\"${hex(value)}\"" else value.toString()
    }
}
