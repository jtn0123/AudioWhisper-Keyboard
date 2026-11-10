package com.jtn0123.audiowhisper.suggest

import com.android.inputmethod.keyboard.ProximityInfo
import com.android.inputmethod.latin.BinaryDictionary
import com.android.inputmethod.latin.DicTraverseSession
import java.io.Closeable

/**
 * The AOSP dictionary engine (ported to Rust, in libawnative.so) behind [TypingDecoder]. One decoder session is
 * shared, so every call is serialized on this object; a call takes a few milliseconds.
 */
class LatinDictionaryEngine private constructor(
    private val dictionary: BinaryDictionary,
    private val session: DicTraverseSession,
    private val proximityInfo: ProximityInfo,
) : TypingDecoder, Closeable {
    @Synchronized
    override fun decode(
        input: String,
        previous: PreviousWords,
        blockOffensiveWords: Boolean,
        taps: List<TapOffset?>,
    ): List<RawSuggestion> {
        val codePoints = input.codePoints().toArray()
        if (codePoints.size > BinaryDictionary.MAX_WORD_LENGTH) return emptyList()
        val knownTaps = taps.takeIf { it.size == codePoints.size }
        session.resetForTyping(blockOffensiveWords)
        codePoints.forEachIndexed { index, codePoint ->
            session.mInputCodePoints[index] = codePoint
            QwertyGeometry.touchPoint(codePoint, knownTaps?.get(index))?.let { (x, y) ->
                session.mXCoordinates[index] = x
                session.mYCoordinates[index] = y
            }
        }
        val slots = previousWordSlots(previous)
        for (slot in 0 until slots.count) {
            session.mPrevWordCodePointArrays[slot] = slots.codePoints[slot]
            session.mIsBeginningOfSentenceArray[slot] = slots.isBeginningOfSentence[slot]
        }
        dictionary.getSuggestions(session, proximityInfo.nativeProximityInfo, codePoints.size, slots.count)
        return readResults()
    }

    @Synchronized
    override fun probability(word: String): Int {
        if (word.isEmpty()) return NOT_A_PROBABILITY
        return dictionary.getProbability(word.codePoints().toArray())
    }

    @Synchronized
    override fun ngramProbability(previous: PreviousWords, word: String): Int {
        val slots = previousWordSlots(previous)
        if (slots.count == 0 || word.isEmpty()) return NOT_A_PROBABILITY
        return dictionary.getNgramProbability(slots.codePoints, slots.isBeginningOfSentence, word.codePoints().toArray())
    }

    /**
     * Every word the dictionary has a probability for (about 160,000), in dictionary order;
     * entries that are only shortcuts, and the sentence-start marker, are left out. One native
     * walk of the dictionary: under 100 ms on the emulator, splitting included, where stepping
     * through it word by word took 1.2 s. Typing waits for the native part.
     */
    fun words(): List<String> = splitWords(synchronized(this) { dictionary.words })

    @Synchronized
    override fun close() {
        session.close()
        proximityInfo.close()
        dictionary.close()
    }

    /** The decoder's output buffers as results, highest score first (native leaves them unsorted). */
    private fun readResults(): List<RawSuggestion> {
        val count = session.mOutputSuggestionCount[0].coerceIn(0, DicTraverseSession.MAX_RESULTS)
        val results = ArrayList<RawSuggestion>(count)
        for (index in 0 until count) {
            val start = index * BinaryDictionary.MAX_WORD_LENGTH
            var length = 0
            while (length < BinaryDictionary.MAX_WORD_LENGTH && session.mOutputCodePoints[start + length] != 0) length++
            val word = String(session.mOutputCodePoints, start, length)
            results += RawSuggestion(word, session.mOutputScores[index], session.mOutputTypes[index])
        }
        results.sortByDescending { it.score }
        return results
    }

    /** Native n-gram arrays: words most recent first, then an empty word flagged as sentence start. */
    private class Slots(val codePoints: Array<IntArray>, val isBeginningOfSentence: BooleanArray) {
        val count: Int get() = codePoints.size
    }

    private fun previousWordSlots(previous: PreviousWords): Slots {
        val count = previous.slotCount
        val codePoints = Array(count) { slot ->
            previous.words.getOrNull(slot)?.codePoints()?.toArray() ?: IntArray(0)
        }
        val isBeginningOfSentence = BooleanArray(count) { slot -> slot >= previous.words.size }
        return Slots(codePoints, isBeginningOfSentence)
    }

    companion object {
        /** The bundled dictionary's word count, rounded up, so the list is sized once. */
        internal const val EXPECTED_WORD_COUNT = 170_000

        /**
         * Opens the dictionary stored at [offset]..[offset] + [length] in [path] (a plain file or
         * an uncompressed APK entry). Returns null if the file is not a usable dictionary.
         */
        fun open(path: String, offset: Long, length: Long, locale: String): LatinDictionaryEngine? {
            val dictionary = BinaryDictionary(path, offset, length)
            if (!dictionary.isValid) {
                dictionary.close()
                return null
            }
            val session = DicTraverseSession(locale, dictionary)
            return LatinDictionaryEngine(dictionary, session, qwertyProximityInfo())
        }

        private fun qwertyProximityInfo(): ProximityInfo {
            val keys = QwertyGeometry.keys
            return ProximityInfo(
                QwertyGeometry.KEYBOARD_WIDTH, QwertyGeometry.KEYBOARD_HEIGHT,
                QwertyGeometry.GRID_WIDTH, QwertyGeometry.GRID_HEIGHT,
                QwertyGeometry.KEY_WIDTH, QwertyGeometry.KEY_HEIGHT,
                QwertyGeometry.proximityGrid(), keys.size,
                IntArray(keys.size) { keys[it].x }, IntArray(keys.size) { keys[it].y },
                IntArray(keys.size) { QwertyGeometry.KEY_WIDTH }, IntArray(keys.size) { QwertyGeometry.KEY_HEIGHT },
                IntArray(keys.size) { keys[it].codePoint },
            )
        }
    }
}

/** [BinaryDictionary.getWords]'s text as words: each one is followed by a newline. */
internal fun splitWords(text: String): List<String> {
    val words = ArrayList<String>(LatinDictionaryEngine.EXPECTED_WORD_COUNT)
    var start = 0
    while (start < text.length) {
        val end = text.indexOf('\n', start).let { if (it < 0) text.length else it }
        if (end > start) words += text.substring(start, end)
        start = end + 1
    }
    return words
}
