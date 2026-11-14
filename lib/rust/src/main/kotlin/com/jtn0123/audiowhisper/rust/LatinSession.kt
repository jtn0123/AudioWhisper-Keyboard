package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/** Owns one native dictionary, search cache and QWERTY model. Only transfers data across JNI. */
class LatinSession private constructor(private var handle: Long) : AutoCloseable {
    private val cleanable = RustLibrary.cleaner.register(this, Free(handle))
    class Candidate(val word: String, val score: Int, val kindAndFlags: Int)

    /** Keep close serialized with an entire native engine exchange. A closed lookup fails lazily,
     * just as probability() did: a no-op learner event need not ask the dictionary at all. */
    @Synchronized internal fun <T> withHandle(action: (Long) -> T): T = action(handle)

    @Synchronized fun isName(personal: PersonalWordStore, word: String): Boolean {
        check(handle != 0L) { "the session is closed" }
        return NlpNative.isName(handle, personal.handle, word).also { Reference.reachabilityFence(personal) }
    }
    @Synchronized fun unknownWords(words: List<String>): List<String> = NlpNative.unknownWords(handle, words.toTypedArray()).asList()
    @Synchronized fun probability(word: String): Int = LatinSessionNative.probability(handle, word)
    @Synchronized fun ngram(word: String, previous: List<String>, sentenceStart: Boolean): Int =
        LatinSessionNative.ngram(handle, word, previous.toTypedArray(), sentenceStart)

    @Synchronized fun words(): List<String> {
        val lengths = LatinSessionNative.wordLengths(handle)
        val text = LatinSessionNative.words(handle)
        var at = 0
        return lengths.map { length -> text.substring(at, at + length).also { at += length } }
    }

    @Synchronized fun decode(input: String, previous: List<String>, sentenceStart: Boolean, block: Boolean, taps: FloatArray): List<Candidate> {
        val packed = LatinSessionNative.decode(handle, input, previous.toTypedArray(), sentenceStart, block, taps)
        val results = ArrayList<Candidate>()
        var at = 0
        while (at < packed.size) {
            val score = packed[at++]
            val kind = packed[at++]
            val length = packed[at++]
            val units = CharArray(length) { packed[at++].toChar() }
            results += Candidate(units.concatToString(), score, kind)
        }
        return results
    }

    @Synchronized fun suggest(
        personal: PersonalWordStore, learned: LearnedWordStore?, typed: String, previous: List<String>,
        sentenceStart: Boolean, midWord: Boolean, block: Boolean, taps: FloatArray, maxCount: Int, threshold: Float,
    ): Suggester.Strip {
        val marks = IntArray(2)
        val words = LatinSessionNative.suggest(handle, personal.handle, learned?.handle ?: 0L, typed,
            previous.toTypedArray(), sentenceStart, midWord, block, taps, maxCount, threshold, marks)
        Reference.reachabilityFence(personal)
        Reference.reachabilityFence(learned)
        return Suggester.Strip(words.asList(), marks[0], marks[1])
    }

    @Synchronized fun spell(personal: PersonalWordStore, learned: LearnedWordStore?, word: String, block: Boolean, maxCount: Int, threshold: Float): Suggester.Verdict {
        val verdict = IntArray(2)
        val words = LatinSessionNative.spell(handle, personal.handle, learned?.handle ?: 0L, word, block, maxCount, threshold, verdict)
        Reference.reachabilityFence(personal)
        Reference.reachabilityFence(learned)
        return Suggester.Verdict(verdict[0] != 0, words.asList(), verdict[1] != 0)
    }

    @Synchronized override fun close() {
        cleanable.clean()
        handle = 0
    }
    private class Free(private val handle: Long) : Runnable {
        override fun run() = LatinSessionNative.free(handle)
    }
    companion object {
        fun open(bytes: ByteArray): LatinSession? = LatinSessionNative.open(bytes).let { if (it == 0L) null else LatinSession(it) }
    }
}

internal object LatinSessionNative {
    init { RustLibrary.load() }
    @JvmStatic external fun open(bytes: ByteArray): Long
    @JvmStatic external fun free(handle: Long)
    @JvmStatic external fun probability(handle: Long, word: String): Int
    @JvmStatic external fun ngram(handle: Long, word: String, previous: Array<String>, sentenceStart: Boolean): Int
    @JvmStatic external fun words(handle: Long): String
    @JvmStatic external fun wordLengths(handle: Long): IntArray
    @JvmStatic external fun decode(handle: Long, input: String, previous: Array<String>, sentenceStart: Boolean, block: Boolean, taps: FloatArray): IntArray
    @JvmStatic external fun suggest(handle: Long, personal: Long, learned: Long, typed: String, previous: Array<String>, sentenceStart: Boolean,
        midWord: Boolean, block: Boolean, taps: FloatArray, maxCount: Int, threshold: Float, marks: IntArray): Array<String>
    @JvmStatic external fun spell(handle: Long, personal: Long, learned: Long, word: String, block: Boolean, maxCount: Int, threshold: Float, verdict: IntArray): Array<String>
}
