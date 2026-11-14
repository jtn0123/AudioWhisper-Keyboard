package com.jtn0123.audiowhisper.reader

import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import kotlin.concurrent.thread
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ReleasedOnceTest {
    private class Native {
        var released = 0
        var usedAfterRelease = false
        fun call() {
            if (released > 0) usedAfterRelease = true
        }
    }

    @Test
    fun `a use after close does nothing`() {
        val native = Native()
        val held = ReferenceReleasedOnce(native) { it.released++ }
        assertTrue(held.use { it.call() })
        held.close()
        assertFalse(held.use { it.call() })
        assertFalse(native.usedAfterRelease)
    }

    @Test
    fun `close frees once however often it is called`() {
        val native = Native()
        val held = ReferenceReleasedOnce(native) { it.released++ }
        repeat(3) { held.close() }
        assertEquals(1, native.released)
    }

    @Test
    fun `close waits for a use in progress`() {
        val native = Native()
        val held = ReferenceReleasedOnce(native) { it.released++ }
        val using = CountDownLatch(1)
        val finish = CountDownLatch(1)
        var releasedDuringUse = -1
        val worker = thread {
            held.use {
                using.countDown()
                finish.await(5, TimeUnit.SECONDS)
                releasedDuringUse = it.released
            }
        }
        assertTrue(using.await(5, TimeUnit.SECONDS))
        val closer = thread { held.close() }
        Thread.sleep(50)
        finish.countDown()
        worker.join(5_000)
        closer.join(5_000)
        assertEquals(0, releasedDuringUse)
        assertEquals(1, native.released)
    }
}
