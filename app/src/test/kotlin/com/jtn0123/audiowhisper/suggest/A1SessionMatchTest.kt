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
        for (dictionary in TestDictionaries.both) {
            val output = File("build/stage2-a1", dictionary.nameWithoutExtension)
            val baseline = A1Replay.capture(dictionary, File(output, "reference"), reference = true)
            val actual = A1Replay.capture(dictionary, File(output, "current"))
            assertEquals(baseline, actual, dictionary.name)
        }
    }
}
