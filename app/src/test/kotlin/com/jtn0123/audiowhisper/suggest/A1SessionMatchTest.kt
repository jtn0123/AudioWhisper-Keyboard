package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith

/** The reference's answers are recorded on golden tapes named a1-* (docs/phase6-kotlin-references.md). */
class A1SessionMatchTest {
    @Test fun rejectedTapResetsTheSameWeightBeforeThrowing() {
        val file = File("src/main/assets/dicts/main_en-US.dict")
        requireNotNull(ReferenceLatinDictionaryEngine.open(file.path, 0, file.length(), "en_US")).use { old ->
            requireNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US")).use { native ->
                val taps = listOf(TapOffset(Float.NaN, 0f))
                val expected = assertFailsWith<IllegalArgumentException> { old.decode("a", PreviousWords.NONE, true, taps) }
                val actual = assertFailsWith<IllegalArgumentException> { native.decode("a", PreviousWords.NONE, true, taps) }
                assertEquals(expected.message, actual.message)
                val snapshot = A1Replay.snapshot(old)
                assertContentEquals(snapshot, A1Replay.snapshot(native))
                val tape = GoldenTape.ofReference("a1-rejected-tap")
                tape.answer("message", expected.message)
                tape.answer("snapshot", snapshot.toList())
                tape.finish()
            }
        }
    }

    @Test fun bundledDecoderAndFullPipelineMatch() {
        for (dictionary in TestDictionaries.both) {
            val output = File("build/stage2-a1", dictionary.nameWithoutExtension)
            val baseline = A1Replay.capture(dictionary, File(output, "reference"), reference = true)
            A1Replay.record(File(output, "reference"), dictionary.nameWithoutExtension) { GoldenTape.ofReference(it) }
            val actual = A1Replay.capture(dictionary, File(output, "current"))
            assertEquals(baseline, actual, dictionary.name)
        }
    }
}
