package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.rust.LatinSession
import java.io.Closeable
import java.io.RandomAccessFile

/** File I/O and UI model conversion around the native session. Decoder policy lives in Rust. */
class LatinDictionaryEngine private constructor(internal val session: LatinSession) : TypingDecoder, Closeable {
    override fun decode(input: String, previous: PreviousWords, blockOffensiveWords: Boolean, taps: List<TapOffset?>): List<RawSuggestion> =
        session.decode(input, previous.words, previous.isBeginningOfSentence, blockOffensiveWords, taps.packed())
            .map { RawSuggestion(it.word, it.score, it.kindAndFlags) }

    override fun probability(word: String): Int = session.probability(word)
    override fun ngramProbability(previous: PreviousWords, word: String): Int = session.ngram(word, previous.words, previous.isBeginningOfSentence)
    fun words(): List<String> = session.words()

    /** Adds the n-gram model at [offset] of [path]: next words, and context for typing and swiping. */
    fun attachModel(path: String, offset: Long, length: Long): Boolean =
        offset >= 0 && length in 0..Int.MAX_VALUE && session.attachModel(path, offset, length)
    override fun close() = session.close()

    companion object {
        /** Read a plain dictionary or the uncompressed dictionary entry inside the APK. */
        fun open(path: String, offset: Long, length: Long, locale: String): LatinDictionaryEngine? {
            if (offset < 0 || length < 0 || length > Int.MAX_VALUE) return null
            val bytes = try {
                RandomAccessFile(path, "r").use { file ->
                    file.seek(offset)
                    ByteArray(length.toInt()).also(file::readFully)
                }
            } catch (_: java.io.IOException) { return null }
            return LatinSession.open(bytes)?.let(::LatinDictionaryEngine)
        }
    }
}

/** JNI representation only: presence, dx, dy for each nullable tap. */
internal fun List<TapOffset?>.packed(): FloatArray = FloatArray(size * 3).also { packed ->
    forEachIndexed { index, tap ->
        packed[index * 3] = if (tap != null) 1f else 0f
        packed[index * 3 + 1] = tap?.dx ?: 0f
        packed[index * 3 + 2] = tap?.dy ?: 0f
    }
}
