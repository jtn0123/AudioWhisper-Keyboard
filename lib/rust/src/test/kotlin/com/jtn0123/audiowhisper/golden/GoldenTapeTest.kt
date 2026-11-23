package com.jtn0123.audiowhisper.golden

import org.opentest4j.AssertionFailedError
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/** The tape every retired reference's test now checks against: what passes and what fails. */
class GoldenTapeTest {
    private fun GoldenTape.recordSelf(last: List<String> = listOf("ab", "c")) {
        for (i in 0 until 5) record("a$i")
        record(last)
    }

    @Test
    fun theSameAnswersPassAgainstADigestWorkedOutApart() {
        GoldenTape.check("golden-tape-self") { it.recordSelf() }
        GoldenTape.ofReference("golden-tape-self").apply { recordSelf() }.finish()
    }

    /** As JUnit's own failure, which is what the mutation scripts count as caught. */
    @Test
    fun aDifferentAnswerFailsAtTheNextPowerOfTwoAndSaysWhere() {
        val tape = GoldenTape("golden-tape-self")
        tape.record("a0")
        tape.record("a1")
        tape.record("a2")
        val failure = assertFailsWith<AssertionFailedError> { tape.record("a3 changed") }
        assertTrue(failure.message!!.contains("answers 3 to 4 differ"), failure.message)
    }

    @Test
    fun partsDoNotHashAlikeByRunningTogether() {
        val tape = GoldenTape("golden-tape-self")
        tape.recordSelf(last = listOf("a", "bc"))
        val failure = assertFailsWith<AssertionError> { tape.finish() }
        assertTrue(failure.message!!.contains("after the last power of two"), failure.message)
    }

    @Test
    fun tooFewOrTooManyAnswersFailAtTheFinish() {
        val short = GoldenTape("golden-tape-self")
        for (i in 0 until 5) short.record("a$i")
        val failure = assertFailsWith<AssertionError> { short.finish() }
        assertEquals("golden-tape-self: 6 answers in the golden file, 5 recorded", failure.message)

        val long = GoldenTape("golden-tape-self")
        long.recordSelf()
        long.record("extra")
        assertFailsWith<AssertionError> { long.finish() }
    }

    @Test
    fun aMissingOrBrokenGoldenFileFails() {
        val missing = GoldenTape("golden-tape-nowhere")
        missing.record("a0")
        val failure = assertFailsWith<AssertionError> { missing.finish() }
        assertTrue(failure.message!!.contains("no golden file"), failure.message)
        assertFailsWith<AssertionError> { GoldenTape("golden-tape-no-count") }
    }

    @Test
    fun anAnswerOfNullIsNotTheTextNull() {
        GoldenTape.check("golden-tape-answers") {
            it.answer("x", null)
            it.answer("x", "null")
            it.answer("n", 1)
        }
        val swapped = GoldenTape("golden-tape-answers")
        assertFailsWith<AssertionFailedError> { swapped.answer("x", "null") }
    }

    @Test
    fun aFinishedTapeTakesNoMoreAnswers() {
        val tape = GoldenTape("golden-tape-self")
        tape.recordSelf()
        tape.finish()
        assertFailsWith<IllegalStateException> { tape.record("late") }
        assertFailsWith<IllegalStateException> { tape.finish() }
    }
}
