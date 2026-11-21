package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.*

class ModelLoadException(cause: Throwable) : Exception("The speech model could not be loaded", cause)

/** Managed resource/exception storage. Rust decides load, reuse, cached failure and release. */
class ModelHolder<T : AutoCloseable>(private val policy: DictationSessionPlan, private val load: () -> T) {
    constructor(load: () -> T) : this(DictationSessionPlan(), load)
    private var model: T? = null
    private var failure: ModelLoadException? = null
    fun get(): T = when (policy.model(DictationModelOp.GET)) {
        DictationModelState.LOADED -> checkNotNull(model)
        DictationModelState.FAILED -> throw checkNotNull(failure)
        DictationModelState.NONE -> try {
            load().also { model = it; policy.model(DictationModelOp.LOADED) }
        } catch (error: Throwable) {
            throw ModelLoadException(error).also { failure = it; policy.model(DictationModelOp.FAILED) }
        }
        else -> error("Unknown native model disposition")
    }
    val isLoaded: Boolean get() = policy.model(DictationModelOp.IS_LOADED) == 1
    fun retry() { policy.model(DictationModelOp.RETRY); failure = null }
    fun close(): Boolean {
        if (policy.model(DictationModelOp.CLOSE) == 0) return false
        val loaded = checkNotNull(model); model = null; loaded.close(); return true
    }
}
