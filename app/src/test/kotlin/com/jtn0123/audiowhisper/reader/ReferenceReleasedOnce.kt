package com.jtn0123.audiowhisper.reader

/**
 * A native object that one thread may be using while another frees it: [use] and [close] take
 * turns, [close] frees it once, and a [use] after that does nothing instead of calling into
 * freed memory. Read-aloud turned off mid-sentence is the case: the service frees the voice
 * while its worker thread may be about to start the next sentence.
 */
internal class ReferenceReleasedOnce<T : Any>(value: T, private val release: (T) -> Unit) : AutoCloseable {
    private val lock = Any()
    private var value: T? = value

    /** Runs [block] with the object; false, without running it, once [close] has freed it. */
    fun use(block: (T) -> Unit): Boolean = synchronized(lock) {
        val live = value ?: return false
        block(live)
        true
    }

    /** Frees the object, after any [use] in progress; later calls do nothing. */
    override fun close() {
        synchronized(lock) {
            val live = value ?: return
            value = null
            release(live)
        }
    }
}
