package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import java.security.MessageDigest
import kotlin.random.Random

/** Fixed public fixtures only. No editor, user dictionary or learned file is read or logged. */
object A1Replay {
    const val DECODER_COUNT = 4_000
    const val PIPELINE_COUNT = 2_000

    data class Question(val state: TypingState, val block: Boolean, val limit: Int)

    fun questions(words: List<String>, count: Int, seed: Int): List<Question> {
        val random = Random(seed)
        val fixed = listOf("hello", "helo", "teh", "im", "dont", "work", "world", "keyboard", "café", "can't")
        return List(count) { index ->
            var word = if (index % 3 == 0) words[random.nextInt(words.size)] else fixed.random(random)
            word = when (index % 13) {
                0 -> ""
                1 -> word.take(random.nextInt(1, word.length + 1))
                2 -> word.dropLast(1)
                3 -> word + "e"
                4 -> word.uppercase(java.util.Locale.US)
                5 -> word.replaceFirstChar { it.uppercaseChar() }
                6 -> if (word.length > 1) word[1] + word[0].toString() + word.drop(2) else word
                7 -> "a".repeat(47 + index % 4)
                8 -> listOf("😀", "é", "\uD800", "İ", "-", "a1")[index % 6] + word
                else -> word
            }
            val previous = when (index % 6) {
                0 -> PreviousWords.NONE
                1 -> PreviousWords(emptyList(), true)
                2 -> PreviousWords(listOf("the"), false)
                3 -> PreviousWords(listOf("to", "want", "I"), true)
                4 -> PreviousWords.from("I want to ")
                else -> PreviousWords(List(random.nextInt(4)) { fixed.random(random) }, random.nextBoolean())
            }
            val taps = if (index % 4 == 0) emptyList() else List(word.codePointCount(0, word.length)) {
                if (random.nextInt(4) == 0) null else TapOffset((random.nextInt(1001) - 500) / 1000f, (random.nextInt(1001) - 500) / 1000f)
            }
            Question(TypingState(word, previous, index % 9 == 0, taps), index % 2 == 0, index % 8 + 1)
        }
    }

    fun capture(dictionary: File, output: File): Pair<String, String> {
        output.mkdirs()
        val engine = requireNotNull(LatinDictionaryEngine.open(dictionary.path, 0, dictionary.length(), "en_US"))
        engine.use {
            val words = engine.words()
            val decoder: TypingDecoder = engine
            val decoderFile = File(output, "decoder.txt")
            decoderFile.bufferedWriter().use { out ->
                questions(words, DECODER_COUNT, 20261113).forEach { q ->
                    val s = q.state
                    out.appendLine(pack(decoder.decode(s.typed, s.previousWords, q.block, s.taps)))
                    out.appendLine(snapshot(engine).joinToString("."))
                    out.appendLine("${decoder.probability(s.typed)}|${decoder.ngramProbability(s.previousWords, s.typed)}")
                }
            }
            val personal = PersonalWords(listOf(PersonalWords.Entry("AudioWhisper", 255, "aw"), PersonalWords.Entry("helloworld", 120)))
            val learned = LearnedWords(null)
            repeat(5) { learned.add("hello", "the"); learned.add("keyboard", LearnedWords.SENTENCE_START) }
            val pipelineFile = File(output, "pipeline.txt")
            pipelineFile.bufferedWriter().use { out ->
                val suggester = LatinSuggester(engine, personalWords = { personal }, history = { learned })
                questions(words, PIPELINE_COUNT, 20261114).forEach { q ->
                    val strip = suggester.suggest(q.state, q.limit, q.block)
                    out.appendLine(strip.joinToString("|") { "${units(it.text)}:${it.isAutoCorrection}:${it.isTypedWord}" })
                    val spell = suggester.spell(q.state.typed, q.limit, q.block)
                    out.appendLine("${spell.isValid}:${spell.isHighConfidence}|" + spell.suggestions.joinToString("|") { units(it) })
                }
            }
            return digest(decoderFile) to digest(pipelineFile)
        }
    }

    /**
     * Records each line [capture] wrote to [output] on the golden tapes `a1-decoder-[name]` and
     * `a1-pipeline-[name]`, made by [tape], one answer a line, and finishes them
     * (docs/phase6-kotlin-references.md). The goldens hold the answers of the AOSP engine glue the
     * native session replaced, written on the host, where that glue ran; the shipped dictionary's
     * are checked there and on a device.
     */
    fun record(output: File, name: String, tape: (String) -> GoldenTape) {
        for (part in listOf("decoder", "pipeline")) {
            val answers = tape("a1-$part-$name")
            File(output, "$part.txt").useLines { lines -> lines.forEach { answers.record(it) } }
            answers.finish()
        }
    }

    /** The native session's output buffers and weight, as the AOSP session laid its fields out. */
    internal fun snapshot(engine: LatinDictionaryEngine): IntArray {
        val session = engine.javaClass.getDeclaredField("session").also { it.isAccessible = true }.get(engine)
        val handle = session.javaClass.getDeclaredField("handle").also { it.isAccessible = true }.getLong(session)
        return A1Probe.snapshot(handle)
    }

    fun pack(results: List<RawSuggestion>) = results.joinToString("|") { "${units(it.word)}:${it.score}:${it.kindAndFlags}" }
    private fun units(text: String) = text.map { it.code }.joinToString(".")
    private fun digest(file: File) = MessageDigest.getInstance("SHA-256").digest(file.readBytes()).joinToString("") { "%02x".format(it) }
}

object A1Probe { @JvmStatic external fun snapshot(handle: Long): IntArray }
