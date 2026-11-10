package com.jtn0123.audiowhisper.reader

import kotlin.test.Test
import kotlin.test.assertEquals

/** Blocks placed on the output's timeline, in frames. */
class PlayQueueTest {
    private val heard = mutableListOf<Int>()
    private val queue = PlayQueue()

    private fun add(vararg ids: Int) = ids.forEach { id -> queue.add(id) { heard += id } }

    private fun at(frame: Long): Int = queue.heard(frame).let { result ->
        result.started.forEach { it() }
        result.finished
    }

    @Test
    fun aBlockStartsWhenItsFirstFrameIsPlayedNotWhenItIsWritten() {
        add(1)
        queue.begins(1, 0)
        assertEquals(0, at(0))
        assertEquals(emptyList(), heard)
        assertEquals(0, at(10))
        assertEquals(listOf(1), heard)
    }

    @Test
    fun aBlockIsOverOnlyOnceItsLastFrameIsPlayed() {
        add(1, 2)
        queue.begins(1, 0)
        queue.ends(1, 100)
        queue.begins(2, 100)
        assertEquals(0, at(99))
        assertEquals(2, queue.size)
        assertEquals(1, at(100))
        assertEquals(1, queue.size)
        assertEquals(listOf(1), heard)
    }

    @Test
    fun theNextBlockStartsOnceTheOutputPassesIntoIt() {
        add(1, 2)
        queue.begins(1, 0)
        queue.ends(1, 100)
        queue.begins(2, 100)
        at(150)
        assertEquals(listOf(1, 2), heard)
    }

    @Test
    fun eachBlockStartsOnce() {
        add(1)
        queue.begins(1, 0)
        at(10)
        at(20)
        assertEquals(listOf(1), heard)
    }

    @Test
    fun aBlockNotYetGeneratedHoldsBackTheOnesAfterIt() {
        add(1, 2)
        assertEquals(0, at(1_000))
        assertEquals(emptyList(), heard)
        assertEquals(2, queue.size)
    }

    @Test
    fun aBlockWithNoAudioStartsAndEndsWhereItWouldHaveBegun() {
        add(1, 2)
        queue.begins(1, 0)
        queue.ends(1, 50)
        queue.ends(2, 50)
        assertEquals(2, at(50))
        assertEquals(listOf(1, 2), heard)
        assertEquals(0, queue.size)
    }

    @Test
    fun aLaterBeginningDoesNotMoveABlocksStart() {
        add(1)
        queue.begins(1, 0)
        queue.begins(1, 80)
        at(40)
        assertEquals(listOf(1), heard)
    }

    @Test
    fun clearingDropsEveryBlockAndSaysHowMany() {
        add(1, 2, 3)
        assertEquals(3, queue.clear())
        assertEquals(0, queue.size)
        assertEquals(0, at(1_000))
    }

    @Test
    fun framesForABlockNoLongerQueuedAreIgnored() {
        add(1)
        queue.clear()
        queue.begins(1, 0)
        queue.ends(1, 10)
        assertEquals(0, at(100))
        assertEquals(emptyList(), heard)
    }
}
