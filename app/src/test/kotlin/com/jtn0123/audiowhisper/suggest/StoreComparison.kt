package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import kotlin.random.Random

/**
 * Asks the word stores in Rust (lib/rust/crates/suggest, behind [LearnedWords], [KeptWords] and
 * [PersonalWords]) the questions the Kotlin they replaced was asked, and puts every answer on
 * [tape], whose golden files hold that Kotlin's answers (docs/phase6-kotlin-references.md): random
 * runs of learning, keeping, saving and loading, asking the same questions after every step. The
 * stores save to files in [dir].
 *
 * [sigmaExact] says whether words may hold Σ, which lower-cases to ς at the end of a word and to σ
 * elsewhere. Rust decides that by ICU's rule, as Android does; the JVM the goldens came from used
 * its own, so off a phone Σ is left out.
 */
class StoreComparison(private val sigmaExact: Boolean, private val dir: File, private val tape: GoldenTape) {
    private val pieces = if (sigmaExact) PIECES else PIECES - "$CAPITAL_SIGMA"

    private fun check(what: String, answer: Any?) = tape.answer(what, answer)

    /** What [body] gives, or the exception it throws, so a throw is an answer too. */
    private inline fun outcome(body: () -> Any?): Any? = try {
        body()
    } catch (e: IllegalArgumentException) {
        "threw ${e.javaClass.name}: ${e.message}"
    }

    /** A store of learned words, the file it saves to, and the words a run uses. */
    private class Learned(val rust: LearnedWords, val rustFile: File, val vocabulary: List<String>)

    /** [runs] runs of [steps] random steps each on a store of learned words. */
    fun learned(random: Random, runs: Int, steps: Int) = repeat(runs) {
        val (words, pairs) = CAPACITIES.random(random) to CAPACITIES.random(random)
        val rustFile = File(dir, "rust-learned.txt").apply { delete() }
        val stores = Learned(LearnedWords(rustFile, words, pairs), rustFile, random.vocabulary())
        repeat(steps) {
            learnedStep(random, stores)
            askLearned(random, stores)
        }
    }

    private fun learnedStep(random: Random, stores: Learned) {
        val word = random.word(stores.vocabulary)
        val previous = random.previous(stores.vocabulary)
        when (random.nextInt(20)) {
            in 0..9 -> stores.rust.add(word, previous, keepSpelling = random.nextBoolean())
            in 10..12 -> stores.rust.remove(word, previous)
            13 -> if (random.nextInt(8) == 0) stores.rust.clear()
            14 -> checkSaved("learned", stores.rust.save(), stores.rustFile)
            15 -> {
                load(random, stores.rustFile) { random.learnedText(stores.vocabulary) }
                stores.rust.load()
            }
        }
    }

    private fun askLearned(random: Random, stores: Learned) {
        val rust = stores.rust
        check("knownSpellings", rust.knownSpellings())
        val word = random.word(stores.vocabulary)
        check("spellingOf", rust.spellingOf(word))
        val typed = random.typed(stores.vocabulary)
        val maxCount = random.nextInt(-1, 5)
        val accept = ACCEPTS.random(random)
        check("completions $maxCount", rust.completions(typed, maxCount, accept))
        val previous = random.previousWords(stores.vocabulary)
        val (minCount, mostCount) = random.nextInt(-1, 4) to random.nextInt(-2, 5)
        check("followers", outcome { rust.followers(previous, minCount, mostCount) })
        check("pairCount", rust.pairCount(previous, word))
    }

    /** A store of kept words, the file it saves to, and the words a run uses. */
    private class Kept(val rust: KeptWords, val rustFile: File, val vocabulary: List<String>)

    /** [runs] runs of [steps] random steps each on a store of kept words. */
    fun kept(random: Random, runs: Int, steps: Int) = repeat(runs) {
        val capacity = CAPACITIES.random(random)
        val rustFile = File(dir, "rust-kept.txt").apply { delete() }
        val stores = Kept(KeptWords(rustFile, capacity), rustFile, random.vocabulary())
        repeat(steps) {
            keptStep(random, stores)
            for (word in stores.vocabulary + random.word(stores.vocabulary)) check("contains", word in stores.rust)
        }
    }

    private fun keptStep(random: Random, stores: Kept) {
        val word = random.word(stores.vocabulary)
        when (random.nextInt(10)) {
            in 0..6 -> {
                val remember = random.nextInt(4) != 0
                check("add $remember", stores.rust.store.add(word, remember))
            }
            7 -> checkSaved("kept", stores.rust.save(), stores.rustFile)
            8 -> {
                load(random, stores.rustFile) { random.keptText(stores.vocabulary) }
                stores.rust.load()
            }
        }
    }

    /** Whether it saved, and what it wrote. */
    private fun checkSaved(what: String, saved: Boolean, file: File) {
        check("$what save", saved)
        check("$what saved file", if (file.isFile) file.readText() else null)
    }

    /** Writes the file to load: [text], damaged now and then, or none at all. */
    private fun load(random: Random, file: File, text: () -> String) {
        if (random.nextInt(10) == 0) {
            file.delete()
            return
        }
        val bytes = text().toByteArray(Charsets.UTF_8)
        if (random.nextInt(4) == 0 && bytes.isNotEmpty()) {
            repeat(random.nextInt(1, 4)) { bytes[random.nextInt(bytes.size)] = BAD_BYTES.random(random) }
        }
        file.writeBytes(bytes)
    }

    /** [count] random personal dictionaries, asked the same questions. */
    fun personal(random: Random, count: Int) = repeat(count) {
        val vocabulary = random.vocabulary()
        val entries = List(random.nextInt(9)) {
            val word = if (random.nextInt(4) == 0) random.text(4) else random.word(vocabulary)
            val shortcut = if (random.nextInt(3) == 0) random.word(vocabulary) else null
            PersonalWords.Entry(word, FREQUENCIES.random(random), shortcut)
        }
        val rust = PersonalWords(entries)
        check("entries", rust.entries.map { listOf(it.word, it.frequency, it.shortcut) })
        check("isEmpty", rust.isEmpty)
        repeat(4) {
            check("spellingOf", rust.spellingOf(random.word(vocabulary)))
            val typed = random.typed(vocabulary)
            val maxCount = random.nextInt(-1, 6)
            check("completions $maxCount", rust.completions(typed, maxCount))
        }
        val (tag, language) = TAGS.random(random) to LANGUAGES.random(random)
        check("appliesTo", PersonalWords.appliesTo(tag, language))
    }

    /** Every tag against every language. */
    fun languages() {
        for (tag in TAGS) {
            for (language in LANGUAGES) check("appliesTo", PersonalWords.appliesTo(tag, language))
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
        1 -> LearnedWords.SENTENCE_START
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
    }
}
