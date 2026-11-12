package com.jtn0123.audiowhisper.dictation

import java.util.concurrent.ConcurrentHashMap

/**
 * The keyboard's two large models, dictation's (about 580 MB) and the read-aloud voice (a few
 * hundred MB), live in one process with their own idle timers. Loading one while the other sits
 * idle would hold both, making Android likelier to close the keyboard, so before either loads it
 * asks the other to let go of its model if it isn't in use.
 */
object LargeModels {
    enum class Kind { DICTATION, VOICE }

    private val freers = ConcurrentHashMap<Kind, () -> Unit>()

    /** [freeIfIdle] frees [kind]'s model unless it is in use. Called from any thread; it may finish later. */
    fun register(kind: Kind, freeIfIdle: () -> Unit) {
        freers[kind] = freeIfIdle
    }

    fun unregister(kind: Kind) {
        freers.remove(kind)
    }

    /** [kind] is about to load its model: the other frees its own if idle. */
    fun loading(kind: Kind) {
        for ((other, freeIfIdle) in freers) if (other != kind) freeIfIdle()
    }
}
