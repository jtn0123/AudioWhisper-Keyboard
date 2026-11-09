package com.jtn0123.audiowhisper.reader

import android.os.SystemClock
import com.jtn0123.audiowhisper.rust.SpeakerKeyState

/**
 * The suggestion bar's speaker button: a tap reads from the top of the screen down, a hold reads
 * from the top of the page, and pressing it while it reads stops it. The reading starts when the
 * finger lifts, since only then is a tap told from a hold. The rules are in Rust ([SpeakerKeyState]).
 */
class SpeakerKey(private val clock: () -> Long = SystemClock::uptimeMillis) {
    enum class Action { STOP, FROM_SCREEN, FROM_TOP }

    private val state = SpeakerKeyState()

    /** The press; [Action.STOP] if a reading is on, and the lift that follows then does nothing. */
    fun down(reading: Boolean): Action? = action(state.down(reading, clock()))

    fun up(): Action? = action(state.up(clock()))

    /** The finger slid off: nothing is read. */
    fun cancel() = state.cancel()

    private fun action(action: SpeakerKeyState.Action?): Action? = action?.let { Action.entries[it.ordinal] }

    companion object {
        val HOLD_MS: Long = SpeakerKeyState.holdMs
    }
}
