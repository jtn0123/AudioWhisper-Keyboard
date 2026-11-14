package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.MicKeyState

/**
 * What a press of the mic key does, in Rust (lib/rust/crates/dictation/src/mic_key.rs). A tap
 * toggles dictation; a hold of [HOLD_MS] or more dictates until it is let go, the way Gboard's and
 * Samsung's mics work. Either way dictation starts on the press, so the first words are heard and
 * the model loads while they are spoken.
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

    private val state = MicKeyState()

    /** The key went down while dictation was [status]: [Action.START] if it was idle. */
    fun press(status: Dictation.Status): Action = action(state.press(status(status), clock()))

    /**
     * The key came up, or the finger slid off it. A press while listening finishes; a hold that
     * started dictation ends it; a release without a press seen toggles, as a tap did before.
     */
    fun release(): Action = action(state.release(clock()))

    private fun status(status: Dictation.Status) = when (status) {
        Dictation.Status.IDLE -> MicKeyState.Status.IDLE
        Dictation.Status.LISTENING -> MicKeyState.Status.LISTENING
        Dictation.Status.FINISHING -> MicKeyState.Status.FINISHING
    }

    private fun action(action: MicKeyState.Action) = when (action) {
        MicKeyState.Action.NONE -> Action.NONE
        MicKeyState.Action.START -> Action.START
        MicKeyState.Action.FINISH -> Action.FINISH
        MicKeyState.Action.END_HOLD -> Action.END_HOLD
        MicKeyState.Action.TOGGLE -> Action.TOGGLE
    }

    companion object {
        /** A press this long, in milliseconds, is a hold. */
        val HOLD_MS: Long = MicKeyState.holdMs
    }
}
