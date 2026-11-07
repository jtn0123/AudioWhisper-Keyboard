package com.jtn0123.audiowhisper.dictation

/**
 * What a press of the mic key does. A tap toggles dictation; a hold of [HOLD_MS] or more
 * dictates until it is let go, the way Gboard's and Samsung's mics work. Either way dictation
 * starts on the press, so the first words are heard and the model loads while they are spoken.
 */
internal class MicKey(private val clock: () -> Long) {
    enum class Action {
        NONE,
        START,
        FINISH,

        /** Finish: the key was let go after holding it to talk. Unlike a tap, nothing else marked the moment. */
        END_HOLD,
        TOGGLE,
    }

    private var pressedAt: Long? = null
    private var startedByPress = false
    private var stopsOnRelease = false

    /** The key went down while dictation was [status]. */
    fun press(status: Dictation.Status): Action {
        pressedAt = clock()
        startedByPress = status == Dictation.Status.IDLE
        stopsOnRelease = status == Dictation.Status.LISTENING
        return if (startedByPress) Action.START else Action.NONE
    }

    /** The key came up, or the finger slid off it. A release without a press seen acts as a tap did before. */
    fun release(): Action {
        val pressed = pressedAt ?: return Action.TOGGLE
        pressedAt = null
        return when {
            stopsOnRelease -> Action.FINISH
            startedByPress && clock() - pressed >= HOLD_MS -> Action.END_HOLD
            else -> Action.NONE
        }
    }

    companion object {
        /** Longer than a tap (about 100 ms), short enough that holding feels immediate. */
        const val HOLD_MS = 300L
    }
}
