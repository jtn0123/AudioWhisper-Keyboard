package com.jtn0123.audiowhisper.rust

import java.lang.ref.Reference

/**
 * Dictation's text rules, in Rust (lib/rust/crates/dictation): finding spoken commands in a
 * transcribed phrase, joining phrases that were transcribed separately, and whether a backspace can
 * still take back the phrase typed last. The keyboard's `Spoken`, `Phrases` and `DictationUndo`
 * call these; their documentation explains the rules.
 */
object DictationRules {
    /** What a part of a phrase asks for. */
    enum class Kind { WORDS, MARK, BREAK, SCRATCH_THAT }

    /** A part of a phrase: its [kind], and its words, its mark or its line breaks as [text]. */
    class Part(val kind: Kind, val text: String)

    /** Delete [replacing] characters before the cursor, then type [text]. */
    class Typing(val replacing: Int, val text: String)

    /**
     * [phrase] tidied by the Mac app's rules, shared through AudioWhisper-Core: filler words ("uh",
     * "um,") removed if [removeFillers], then [vocabulary] applied. [vocabulary] is one entry per
     * line, as on the Mac: a word to spell exactly ("Kubernetes"), or `heard -> written`.
     */
    fun format(phrase: String, vocabulary: String, removeFillers: Boolean): String =
        if (vocabulary.isBlank() && !removeFillers) phrase else DictationNative.format(phrase, vocabulary, removeFillers)

    /** [phrase]'s words and commands, in order. */
    fun parse(phrase: String): List<Part> = DictationNative.parse(phrase).map { part ->
        val kind = when (part[0]) {
            'w' -> Kind.WORDS
            'm' -> Kind.MARK
            'b' -> Kind.BREAK
            's' -> Kind.SCRATCH_THAT
            else -> error("unknown part")
        }
        Part(kind, part.substring(1))
    }

    /** How to type the spoken punctuation [mark] after [before]. */
    fun punctuate(before: String, lastPhrase: String?, mark: Char): Typing =
        typing(DictationNative.punctuate(before, lastPhrase, mark))

    /**
     * Whether a backspace with the cursor at [cursor], after [textBeforeCursor], can still take
     * back [typed], the phrase dictation typed at [start].
     */
    fun revertsAt(typed: String, start: Int, cursor: Int, textBeforeCursor: String): Boolean =
        DictationNative.revertsAt(typed, start, cursor, textBeforeCursor)

    private fun typing(packed: String) = Typing(packed[0].code, packed.substring(1))
}

/**
 * The mic key between a press and its release, in Rust: a tap toggles dictation, and a hold of
 * [holdMs] or more dictates until it is let go. Each press and release says when it came, in
 * milliseconds. Safe to use from any thread. Its native memory is freed once nothing can reach it.
 */
class MicKeyState {
    /** What dictation is doing when the key goes down. */
    enum class Status { IDLE, LISTENING, FINISHING }

    /** What a press or a release asks dictation to do. */
    enum class Action { NONE, START, FINISH, END_HOLD, TOGGLE }

    private val handle = DictationNative.newMicKey()

    init {
        RustLibrary.cleaner.register(this, Free(handle))
    }

    /** The press, at [now], while dictation is [status]: [Action.START] if it was idle. */
    fun press(status: Status, now: Long): Action =
        Action.entries[DictationNative.micKeyPress(handle, status.ordinal, now)].also { Reference.reachabilityFence(this) }

    /** The release, at [now]; [Action.TOGGLE] if no press was seen. */
    fun release(now: Long): Action =
        Action.entries[DictationNative.micKeyRelease(handle, now)].also { Reference.reachabilityFence(this) }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = DictationNative.freeMicKey(handle)
    }

    companion object {
        /** A press this long, in milliseconds, is a hold. */
        val holdMs: Long = DictationNative.holdMs()
    }
}
