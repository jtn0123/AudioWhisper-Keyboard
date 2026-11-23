package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import kotlin.test.Test
import kotlin.test.assertContentEquals
import com.jtn0123.audiowhisper.rust.ReaderWalk

/**
 * The device sentences, cut by the native chunking as the frozen Kotlin cut them
 * (golden/tts-chunk-sentences.txt).
 */
class TtsChunkMatchTest {
    @Test fun committedSentencesAndBoundarySizesMatch() {
        // Frozen FollowAlong/ReadAloudService/NeuralVoice timing and look-ahead values.
        assertContentEquals(longArrayOf(8, 200, 2, 40), ReaderWalk.followConfig())
        val file = listOf(File("src/androidTest/assets/tts-sentences.txt"), File("app/src/androidTest/assets/tts-sentences.txt")).first { it.isFile }
        val sentences = file.readLines()
        GoldenTape.check("tts-chunk-sentences") { tape ->
            for (text in sentences + sentences.joinToString(" ").repeat(20)) {
                for (limit in listOf(1, 32, 128, 3900)) tape.record(PageText.utterances(text, limit))
            }
        }
    }
}
