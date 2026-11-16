package com.jtn0123.audiowhisper.suggest

import com.android.inputmethod.keyboard.ProximityInfo
import com.android.inputmethod.latin.BinaryDictionary
import com.android.inputmethod.latin.DicTraverseSession
import java.io.File
import java.lang.reflect.InvocationTargetException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/** In :app, which already depends on LatinIME and loads the host libawnative for JVM tests. */
class LatinImeJniTest {
    // The answers pinned below are the AOSP dictionary's.
    private val dictionaryFile = TestDictionaries.aosp

    private class Fixture(file: File) : AutoCloseable {
        val dictionary = BinaryDictionary(file.absolutePath, 0, file.length())
        val session = DicTraverseSession("en_US", dictionary)
        private val keys = QwertyGeometry.keys
        val proximity = ProximityInfo(
            QwertyGeometry.KEYBOARD_WIDTH, QwertyGeometry.KEYBOARD_HEIGHT,
            QwertyGeometry.GRID_WIDTH, QwertyGeometry.GRID_HEIGHT,
            QwertyGeometry.KEY_WIDTH, QwertyGeometry.KEY_HEIGHT,
            QwertyGeometry.proximityGrid(), keys.size,
            IntArray(keys.size) { keys[it].x }, IntArray(keys.size) { keys[it].y },
            IntArray(keys.size) { QwertyGeometry.KEY_WIDTH }, IntArray(keys.size) { QwertyGeometry.KEY_HEIGHT },
            IntArray(keys.size) { keys[it].codePoint },
        )

        init {
            session.resetForTyping(true)
            "helo".codePoints().toArray().forEachIndexed { index, code ->
                session.mInputCodePoints[index] = code
                val point = requireNotNull(QwertyGeometry.keyCenter(code))
                session.mXCoordinates[index] = point.first
                session.mYCoordinates[index] = point.second
            }
        }

        fun arguments(): Array<Any> = arrayOf(
            dictionary.nativeHandle, proximity.nativeProximityInfo, session.session,
            session.mXCoordinates, session.mYCoordinates, session.mTimes, session.mPointerIds,
            session.mInputCodePoints, 4, intArrayOf(0, 0, 1, 0, 1000),
            session.mPrevWordCodePointArrays, session.mIsBeginningOfSentenceArray, 0,
            session.mOutputSuggestionCount, session.mOutputCodePoints, session.mOutputScores,
            session.mSpaceIndices, session.mOutputTypes, session.mOutputAutoCommitFirstWordConfidence,
            session.mInputOutputWeightOfLangModelVsSpatialModel,
        )

        override fun close() {
            session.close()
            proximity.close()
            dictionary.close()
        }
    }

    // Reflection bypasses Java's closed-dictionary guard and permits deliberately malformed arrays
    // without changing the production Java API. Unwrap only the reflection wrapper, not JNI errors.
    private fun native(name: String, vararg arguments: Any): Any? {
        val method = BinaryDictionary::class.java.declaredMethods.single { it.name == name }
        method.isAccessible = true
        return try {
            method.invoke(null, *arguments)
        } catch (error: InvocationTargetException) {
            throw error.targetException
        }
    }

    @Test
    fun bundledDictionarySuggestionsCrossJni() {
        Fixture(dictionaryFile).use { fixture ->
            assertTrue(fixture.dictionary.isValid)
            assertTrue(fixture.proximity.nativeProximityInfo != 0L)
            fixture.dictionary.getSuggestions(fixture.session, fixture.proximity.nativeProximityInfo, 4, 0)
            val session = fixture.session
            val words = (0 until session.mOutputSuggestionCount[0])
                .sortedByDescending { session.mOutputScores[it] }
                .map { index ->
                    val start = index * BinaryDictionary.MAX_WORD_LENGTH
                    val length = (0 until BinaryDictionary.MAX_WORD_LENGTH)
                        .firstOrNull { session.mOutputCodePoints[start + it] == 0 } ?: BinaryDictionary.MAX_WORD_LENGTH
                    String(session.mOutputCodePoints, start, length)
                }
            // The decoder ranks corrections; the strip inserts the literal input separately.
            assertEquals(listOf("help", "hell", "he'll"), words.take(3))
        }
    }

    @Test
    fun realDecoderProducesTheDocumentedCandidateStrip() {
        val decoder = requireNotNull(
            LatinDictionaryEngine.open(dictionaryFile.absolutePath, 0, dictionaryFile.length(), "en_US"),
        )
        decoder.use {
            val strip = LatinSuggester(it).suggest(TypingState.of("", "helo", ""), 3, true)
            assertEquals(listOf("help", "helo", "hell"), strip.map { word -> word.text })
        }
    }

    @Test
    fun theStrengthSettingReachesTheNativeSuggester() {
        val decoder = requireNotNull(
            LatinDictionaryEngine.open(dictionaryFile.absolutePath, 0, dictionaryFile.length(), "en_US"),
        )
        decoder.use {
            fun strip(strength: AutoCorrectStrength) =
                LatinSuggester(it, strength = { strength }).suggest(TypingState.of("", "helo", ""), 3, true)
            val corrected = AutoCorrectStrength.entries.associateWith { strength ->
                strip(strength).firstOrNull { word -> word.isAutoCorrection }?.text
            }
            val expected = mapOf(
                AutoCorrectStrength.OFF to null, AutoCorrectStrength.MODEST to "help",
                AutoCorrectStrength.NORMAL to "help", AutoCorrectStrength.AGGRESSIVE to "help",
            )
            assertEquals(expected, corrected)
            // Off still offers the same words, the typed one first as when no correction is sure enough.
            assertEquals(listOf("helo", "help", "hell"), strip(AutoCorrectStrength.OFF).map { word -> word.text })
        }
    }

    @Test
    fun closedDictionaryHandleThrowsFromNative() {
        Fixture(dictionaryFile).use { fixture ->
            fixture.dictionary.close()
            val error = assertFailsWith<IllegalArgumentException> {
                native("getSuggestionsNative", *fixture.arguments())
            }
            assertEquals("the dictionary is closed", error.message)
            val handle = fixture.dictionary.nativeHandle
            assertEquals(0L, handle)
            for ((name, arguments) in listOf(
                "getProbabilityNative" to arrayOf(handle, intArrayOf('a'.code)),
                "getNgramProbabilityNative" to arrayOf(handle, emptyArray<IntArray>(), BooleanArray(0), intArrayOf('a'.code)),
                "getWordsNative" to arrayOf(handle),
            )) {
                val closed = assertFailsWith<IllegalArgumentException> { native(name, *arguments) }
                assertEquals("the dictionary is closed", closed.message)
            }
        }
    }

    @Test
    fun closedSessionHandleThrowsFromNative() {
        Fixture(dictionaryFile).use { fixture ->
            fixture.session.close()
            val error = assertFailsWith<IllegalArgumentException> {
                native("getSuggestionsNative", *fixture.arguments())
            }
            assertEquals("the session is closed", error.message)
        }
    }

    @Test
    fun shortSuggestionArraysThrowFromNative() {
        Fixture(dictionaryFile).use { fixture ->
            // Each input coordinate/code-point buffer and each output buffer is checked separately.
            for (slot in listOf(3, 4, 7, 13, 14, 15, 16, 17, 18, 19)) {
                val arguments = fixture.arguments()
                arguments[slot] = if (slot == 19) FloatArray(0) else IntArray(0)
                assertFailsWith<IllegalArgumentException>("array argument $slot") {
                    native("getSuggestionsNative", *arguments)
                }
            }
        }
    }

    @Test
    fun shortPreviousWordArraysThrowFromNative() {
        Fixture(dictionaryFile).use { fixture ->
            val arguments = fixture.arguments()
            arguments[12] = 1
            arguments[10] = emptyArray<IntArray>()
            assertFailsWith<IllegalArgumentException> { native("getSuggestionsNative", *arguments) }
            arguments[10] = arrayOf(intArrayOf('a'.code))
            arguments[11] = BooleanArray(0)
            assertFailsWith<IllegalArgumentException> { native("getSuggestionsNative", *arguments) }
            assertFailsWith<IllegalArgumentException> {
                fixture.dictionary.getNgramProbability(arrayOf(intArrayOf('a'.code)), BooleanArray(0), intArrayOf('b'.code))
            }
        }
    }
}
