package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import kotlin.test.Test
import kotlin.test.assertFailsWith

/**
 * Checks the native session against the answers of the AOSP engine glue it replaced, on golden
 * tapes named a1-* (docs/phase6-kotlin-references.md).
 */
class A1SessionMatchTest {
    @Test fun rejectedTapResetsTheSameWeightBeforeThrowing() {
        val file = File("src/main/assets/dicts/main_en-US.dict")
        requireNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US")).use { native ->
            val taps = listOf(TapOffset(Float.NaN, 0f))
            val thrown = assertFailsWith<IllegalArgumentException> { native.decode("a", PreviousWords.NONE, true, taps) }
            val tape = GoldenTape("a1-rejected-tap")
            tape.answer("message", thrown.message)
            tape.answer("snapshot", A1Replay.snapshot(native).toList())
            tape.finish()
        }
    }

    @Test fun bundledDecoderAndFullPipelineMatch() {
        for (dictionary in TestDictionaries.both) {
            val output = File("build/stage2-a1", dictionary.nameWithoutExtension)
            A1Replay.capture(dictionary, output)
            A1Replay.record(output, dictionary.nameWithoutExtension) { GoldenTape(it) }
        }
    }
}
