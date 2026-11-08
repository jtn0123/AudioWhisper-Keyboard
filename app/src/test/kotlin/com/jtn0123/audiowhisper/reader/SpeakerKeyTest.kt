package com.jtn0123.audiowhisper.reader

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class SpeakerKeyTest {
    private var now = 1_000L
    private val key = SpeakerKey { now }

    @Test
    fun aTapReadsFromTheScreen() {
        assertNull(key.down(reading = false))
        now += 120
        assertEquals(SpeakerKey.Action.FROM_SCREEN, key.up())
    }

    @Test
    fun aHoldReadsFromTheTopOfThePage() {
        key.down(reading = false)
        now += SpeakerKey.HOLD_MS
        assertEquals(SpeakerKey.Action.FROM_TOP, key.up())
    }

    @Test
    fun aPressWhileReadingStopsAndItsLiftStartsNothing() {
        assertEquals(SpeakerKey.Action.STOP, key.down(reading = true))
        now += SpeakerKey.HOLD_MS
        assertNull(key.up())
    }

    @Test
    fun aFingerSlidOffReadsNothing() {
        key.down(reading = false)
        key.cancel()
        assertNull(key.up())
    }
}
