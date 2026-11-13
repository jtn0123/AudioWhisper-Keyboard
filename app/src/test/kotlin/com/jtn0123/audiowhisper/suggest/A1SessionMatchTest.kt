package com.jtn0123.audiowhisper.suggest

import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith

class A1SessionMatchTest {
    @Test fun rejectedTapResetsTheSameWeightBeforeThrowing() {
        val file = File("src/main/assets/dicts/main_en-US.dict")
        requireNotNull(ReferenceLatinDictionaryEngine.open(file.path, 0, file.length(), "en_US")).use { old ->
            requireNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US")).use { native ->
                val taps = listOf(TapOffset(Float.NaN, 0f))
                val expected = assertFailsWith<IllegalArgumentException> { old.decode("a", PreviousWords.NONE, true, taps) }
                val actual = assertFailsWith<IllegalArgumentException> { native.decode("a", PreviousWords.NONE, true, taps) }
                assertEquals(expected.message, actual.message)
                assertContentEquals(A1Replay.snapshot(old), A1Replay.snapshot(native))
            }
        }
    }

    @Test fun bundledDecoderAndFullPipelineMatch() {
        val dictionary = File("src/main/assets/dicts/main_en-US.dict")
        val baseline = A1Replay.capture(dictionary, File("build/stage2-a1/reference"), reference = true)
        val actual = A1Replay.capture(dictionary, File("build/stage2-a1/current"))
        assertEquals(baseline, actual)
    }
}
