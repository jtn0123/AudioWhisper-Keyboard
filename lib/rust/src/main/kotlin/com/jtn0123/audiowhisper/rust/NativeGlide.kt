package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/** One native context/rank exchange; the session monitor serializes it with close. */
object NativeGlide {
    fun rank(session: LatinSession?, history: LearnedWordStore?, words: List<String>, costs: FloatArray,
        previous: List<String>, sentenceStart: Boolean): IntArray {
        val packet = Utf16Batch(words + previous)
        fun call(handle: Long) = NativeGlideNative.rank(handle, history?.handle ?: 0L,
            packet.text, packet.lengths, costs, words.size, sentenceStart)
        return (session?.withHandle(::call) ?: call(-1L)).also { Reference.reachabilityFence(history) }
    }
}
internal object NativeGlideNative {
    init { RustLibrary.load() }
    @JvmStatic external fun rank(session: Long, history: Long, text: String, lengths: IntArray,
        costs: FloatArray, wordCount: Int, sentenceStart: Boolean): IntArray
}
