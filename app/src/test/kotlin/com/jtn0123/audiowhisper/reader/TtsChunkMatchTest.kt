package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertContentEquals
import com.jtn0123.audiowhisper.rust.ReaderWalk

/** The file-only device sentences also exercise the frozen/native chunking boundary. */
class TtsChunkMatchTest {
    @Test fun committedSentencesAndBoundarySizesMatch() {
        // Frozen FollowAlong/ReadAloudService/NeuralVoice timing and look-ahead values.
        assertContentEquals(longArrayOf(8, 200, 2, 40), ReaderWalk.followConfig())
        val file = listOf(File("src/androidTest/assets/tts-sentences.txt"), File("app/src/androidTest/assets/tts-sentences.txt")).first { it.isFile }
        val sentences = file.readLines()
        val tape = GoldenTape.ofReference("tts-chunk-sentences")
        for (text in sentences + sentences.joinToString(" ").repeat(20)) {
            for (limit in listOf(1, 32, 128, 3900)) {
                val reference = ReferenceReader.PageText.utterances(text, limit)
                tape.record(reference)
                assertEquals(reference, PageText.utterances(text, limit))
            }
        }
        tape.finish()
    }
}
