package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.dictation.Dictation.Status
import com.jtn0123.audiowhisper.dictation.MicKey.Action
import kotlin.test.Test
import kotlin.test.assertEquals

class MicKeyTest {
    private var now = 1_000L
    private val key = MicKey { now }

    private fun pressFor(ms: Long, status: Status): Pair<Action, Action> {
        val down = key.press(status)
        now += ms
        return down to key.release()
    }

    @Test
    fun aTapStartsAndTheNextTapStops() {
        assertEquals(Action.START to Action.NONE, pressFor(80, Status.IDLE))
        assertEquals(Action.NONE to Action.FINISH, pressFor(80, Status.LISTENING))
    }

    @Test
    fun aHoldTalksUntilLetGo() {
        assertEquals(Action.START to Action.END_HOLD, pressFor(MicKey.HOLD_MS, Status.IDLE))
        assertEquals(Action.START to Action.END_HOLD, pressFor(4_000, Status.IDLE))
        assertEquals(Action.START to Action.NONE, pressFor(MicKey.HOLD_MS - 1, Status.IDLE))
    }

    @Test
    fun holdingWhileListeningStopsOnRelease() {
        assertEquals(Action.NONE to Action.FINISH, pressFor(2_000, Status.LISTENING))
    }

    @Test
    fun aPressWhileFinishingDoesNothing() {
        assertEquals(Action.NONE to Action.NONE, pressFor(80, Status.FINISHING))
        assertEquals(Action.NONE to Action.NONE, pressFor(2_000, Status.FINISHING))
    }

    @Test
    fun aReleaseWithoutAPressTogglesAsBefore() {
        assertEquals(Action.TOGGLE, key.release())
        pressFor(80, Status.IDLE)
        assertEquals(Action.TOGGLE, key.release())
    }
}
