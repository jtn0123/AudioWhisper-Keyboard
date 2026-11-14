package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.DictationSessionPlan

class ModelLoadException(cause: Throwable) : Exception("The speech model could not be loaded", cause)

/** Managed resource/exception storage. Rust decides load, reuse, cached failure and release. */
class ModelHolder<T : AutoCloseable>(private val policy: DictationSessionPlan, private val load: () -> T) {
    constructor(load: () -> T) : this(DictationSessionPlan(), load)
    private var model: T? = null
    private var failure: ModelLoadException? = null
    fun get(): T = when (policy.model("get")) {
        1 -> checkNotNull(model)
        2 -> throw checkNotNull(failure)
        0 -> try {
            load().also { model = it; policy.model("loaded") }
        } catch (error: Throwable) {
            throw ModelLoadException(error).also { failure = it; policy.model("failed") }
        }
        else -> error("Unknown native model disposition")
    }
    val isLoaded: Boolean get() = policy.model("isLoaded") == 1
    fun retry() { policy.model("retry"); failure = null }
    fun close(): Boolean {
        if (policy.model("close") == 0) return false
        val loaded = checkNotNull(model); model = null; loaded.close(); return true
    }
}
