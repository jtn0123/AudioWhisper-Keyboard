package com.jtn0123.audiowhisper.reader

import android.os.SystemClock

/**
 * The suggestion bar's speaker button: a tap reads from the top of the screen down, a hold reads
 * from the top of the page, and pressing it while it reads stops it. The reading starts when the
 * finger lifts, since only then is a tap told from a hold.
 */
class SpeakerKey(private val clock: () -> Long = SystemClock::uptimeMillis) {
    enum class Action { STOP, FROM_SCREEN, FROM_TOP }

    private var downAt: Long? = null

    /** The press; [Action.STOP] if a reading is on, and the lift that follows then does nothing. */
    fun down(reading: Boolean): Action? {
        if (reading) {
            downAt = null
            return Action.STOP
        }
        downAt = clock()
        return null
    }

    fun up(): Action? {
        val at = downAt ?: return null
        downAt = null
        return if (clock() - at >= HOLD_MS) Action.FROM_TOP else Action.FROM_SCREEN
    }

    /** The finger slid off: nothing is read. */
    fun cancel() {
        downAt = null
    }

    companion object {
        const val HOLD_MS = 400L
    }
}
