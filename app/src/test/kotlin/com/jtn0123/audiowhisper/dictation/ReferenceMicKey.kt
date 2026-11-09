package com.jtn0123.audiowhisper.dictation

/** The Kotlin `MicKey` replaced by lib/rust/crates/dictation/src/mic_key.rs, kept to test against. */
internal class ReferenceMicKey(private val clock: () -> Long) {
    private var pressedAt: Long? = null
    private var startedByPress = false
    private var stopsOnRelease = false

    fun press(status: Dictation.Status): MicKey.Action {
        pressedAt = clock()
        startedByPress = status == Dictation.Status.IDLE
        stopsOnRelease = status == Dictation.Status.LISTENING
        return if (startedByPress) MicKey.Action.START else MicKey.Action.NONE
    }

    fun release(): MicKey.Action {
        val pressed = pressedAt ?: return MicKey.Action.TOGGLE
        pressedAt = null
        return when {
            stopsOnRelease -> MicKey.Action.FINISH
            startedByPress && clock() - pressed >= HOLD_MS -> MicKey.Action.END_HOLD
            else -> MicKey.Action.NONE
        }
    }

    companion object {
        const val HOLD_MS = 300L
    }
}
