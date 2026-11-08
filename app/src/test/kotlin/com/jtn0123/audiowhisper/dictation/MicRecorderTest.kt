package com.jtn0123.audiowhisper.dictation

import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import kotlin.system.measureTimeMillis
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class MicRecorderTest {
    /** Audio whose reads block until [stop], as the mic's do between its 100 ms chunks; or fail. */
    private class FakeAudio(private val error: Int? = null) : MicAudio {
        val reading = CountDownLatch(1)
        val closed = CountDownLatch(1)
        private val stopped = CountDownLatch(1)
        @Volatile var stoppedAfterClose = false

        override fun read(buffer: FloatArray): Int {
            reading.countDown()
            error?.let { return it }
            return if (stopped.await(BLOCK_SECONDS, TimeUnit.SECONDS)) 0 else buffer.size
        }

        override fun stop() {
            if (closed.count == 0L) stoppedAfterClose = true
            stopped.countDown()
        }

        override fun close() = closed.countDown()
    }

    @Test
    fun stoppingCutsABlockedReadShort() {
        val audio = FakeAudio()
        val recorder = MicRecorder(onLevel = {}, onError = {}, onAudio = {})
        recorder.start { audio }
        assertTrue(audio.reading.await(5, TimeUnit.SECONDS))

        val took = measureTimeMillis { recorder.stop() }

        assertTrue(took < BLOCK_SECONDS * 1000 / 2, "stop waited $took ms for the read")
        assertEquals(0L, audio.closed.count, "the audio is closed by the time stop returns")
    }

    @Test
    fun stoppingAfterAMicErrorLeavesTheClosedAudioAlone() {
        val audio = FakeAudio(error = -3)
        val errors = mutableListOf<Int>()
        val recorder = MicRecorder(onLevel = {}, onError = { synchronized(errors) { errors += it } }, onAudio = {})
        recorder.start { audio }
        assertTrue(audio.closed.await(5, TimeUnit.SECONDS))

        recorder.stop()

        assertEquals(listOf(-3), synchronized(errors) { errors.toList() })
        assertFalse(audio.stoppedAfterClose)
    }

    private companion object {
        const val BLOCK_SECONDS = 10L
    }
}
