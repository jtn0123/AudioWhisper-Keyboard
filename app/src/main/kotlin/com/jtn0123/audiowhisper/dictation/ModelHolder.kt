package com.jtn0123.audiowhisper.dictation

/** The speech model could not be loaded: its files are missing or damaged, or there wasn't the memory. */
class ModelLoadException(cause: Throwable) : Exception("The speech model could not be loaded", cause)

/**
 * The speech model, loaded on the first [get] and released by [close].
 *
 * A failed load is remembered until [retry], so the phrases after it fail at
 * once instead of each repeating a load that takes seconds and hundreds of
 * MB. Not thread-safe: use it from one thread.
 */
class ModelHolder<T : AutoCloseable>(private val load: () -> T) {
    private var model: T? = null
    private var failure: ModelLoadException? = null

    fun get(): T {
        model?.let { return it }
        failure?.let { throw it }
        return try {
            load().also { model = it }
        } catch (e: Throwable) {
            // Throwable, not Exception: a missing native library and running
            // out of memory, the likeliest ways an 800 MB model fails, are Errors.
            throw ModelLoadException(e).also { failure = it }
        }
    }

    /** [get] would hand back a model without loading one. */
    val isLoaded: Boolean get() = model != null

    /** Forgets a failed load, so the next [get] tries again. */
    fun retry() {
        failure = null
    }

    /** Releases the model. False if none was loaded. */
    fun close(): Boolean {
        val loaded = model ?: return false
        model = null
        loaded.close()
        return true
    }
}
