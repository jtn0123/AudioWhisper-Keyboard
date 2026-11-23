package com.jtn0123.audiowhisper.reader

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Main's LoadedVoice (b1cf45f18, reader/VoiceEngine.kt), verbatim apart from the calls to Android,
 * each routed through the same [VoiceSource] the Rust-held [LoadedVoice] uses:
 *
 * - `LargeModels.register(VOICE) { … }`, `.unregister(VOICE)`, `.loading(VOICE)`:
 *   `source.register { … }`, `source.unregister()`, `source.loading()`;
 * - `VoiceEngine.quickState(context) is ModelState.Missing`: `source.missing()`;
 * - `withContext(Dispatchers.IO) { VoiceEngine.load(context) }`: `source.load()`;
 * - `delay(IDLE_MS)`: `source.wait(IDLE_MS)`; and the engine is any [AutoCloseable].
 */
internal class ReferenceLoadedVoice<E : AutoCloseable>(private val source: VoiceSource<E>, private val scope: CoroutineScope) {
    private val mutex = Mutex()
    private var engine: E? = null
    private var release: Job? = null
    private var inUse = false
    private var failed = false
    private var justFailed = false

    init {
        source.register { scope.launch { mutex.withLock { if (!inUse) free() } } }
    }

    suspend fun take(): E? = mutex.withLock {
        inUse = true
        engine?.let { return@withLock it }
        if (failed || source.missing()) return@withLock null
        source.loading()
        source.load()
            .also { engine = it; failed = it == null; justFailed = failed }
    }

    suspend fun failedJustNow(): Boolean = mutex.withLock { justFailed.also { justFailed = false } }

    suspend fun idle() {
        mutex.withLock { inUse = false }
        release?.cancel()
        release = scope.launch {
            source.wait(IDLE_MS)
            mutex.withLock { if (!inUse) free() }
        }
    }

    suspend fun close() {
        source.unregister()
        mutex.withLock { free() }
    }

    private fun free() {
        engine?.close()
        engine = null
    }

    private companion object {
        const val IDLE_MS = 3 * 60_000L
    }
}

/** Main's ReadAloud routing (b1cf45f18): read's three ways, and attach's wait for the service. */
internal object ReferenceReadAloud {
    const val READ = "read"
    const val ASK = "ask"
    const val WAIT = "wait"
    private const val WAIT_MS = 5_000L

    /** `read`: `service?.let { return it.read(from) }`, then `if (!isTurnedOn) return askToTurnOn`, else wait. */
    fun read(serviceUp: Boolean, turnedOn: () -> Boolean): String {
        if (serviceUp) return READ
        if (!turnedOn()) return ASK
        return WAIT
    }

    /** `attach`: `waiting?.takeIf { SystemClock.uptimeMillis() - it.at < WAIT_MS }`. */
    fun stillAsked(at: Long, now: Long) = now - at < WAIT_MS
}
