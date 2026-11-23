package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ReadingRules
import com.jtn0123.audiowhisper.rust.VoiceHold
import com.jtn0123.audiowhisper.rust.VoiceHoldStep
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** What [LoadedVoice] does to Android: the voice's files and load, the timer, the models' registry. */
internal interface VoiceSource<E : AutoCloseable> {
    /** Whether the voice's files aren't there, by names and sizes: cheap. */
    fun missing(): Boolean

    /** Tells the large models a voice is loading, so they free what they can first. */
    fun loading()

    /** Loads the voice off the main thread; null if it won't load. */
    suspend fun load(): E?

    /** Asks to be told to [evict] the voice when memory is wanted. */
    fun register(evict: () -> Unit)

    fun unregister()

    suspend fun wait(ms: Long)
}

/**
 * Keeps the neural voice loaded between readings, so only the first waits for it, and frees it (a
 * few hundred MB) once [ReadingRules.voiceIdleMs] pass without one. When to load, use or free it
 * is Rust's ([VoiceHold], reader::lifetime); this keeps the voice and the timer.
 */
internal class LoadedVoice<E : AutoCloseable>(private val source: VoiceSource<E>, private val scope: CoroutineScope) {
    private val mutex = Mutex()
    private val hold = VoiceHold()
    private var engine: E? = null
    private var release: Job? = null
    private var closed = false

    init {
        source.register { scope.launch { freeIfIdle() } }
    }

    /** The voice, loading it if need be; null if it isn't installed or won't load. */
    suspend fun take(): E? = mutex.withLock {
        when (hold.take(loaded = engine != null)) {
            VoiceHoldStep.USE -> engine
            VoiceHoldStep.CHECK -> if (hold.checked(source.missing()) == VoiceHoldStep.LOAD) load() else null
            else -> null
        }
    }

    private suspend fun load(): E? {
        source.loading()
        return source.load().also {
            engine = it
            hold.loaded(ok = it != null)
        }
    }

    /** Whether the last [take] found the voice won't load; true once per failure, to say so once. */
    suspend fun failedJustNow(): Boolean = mutex.withLock { hold.failedJustNow() }

    /** A reading is over: free the voice unless another starts soon. */
    suspend fun idle() {
        mutex.withLock { hold.idle() }
        release?.cancel()
        release = scope.launch {
            source.wait(ReadingRules.voiceIdleMs)
            freeIfIdle()
        }
    }

    suspend fun close() {
        source.unregister()
        mutex.withLock {
            free()
            closed = true
            hold.close()
        }
    }

    private suspend fun freeIfIdle() = mutex.withLock { if (!closed && hold.mayFree()) free() }

    private fun free() {
        engine?.close()
        engine = null
    }
}
