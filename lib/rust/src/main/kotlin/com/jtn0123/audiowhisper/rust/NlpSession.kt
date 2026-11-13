package com.jtn0123.audiowhisper.rust

/** One orchestration owner shared by both renderers. Calls copy UTF-16; no native callback. */
class NlpSession : AutoCloseable {
    private val allocation = Allocation(NlpNative.create())
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    @Synchronized fun call(operation: String, vararg fields: String): Array<String> = allocation.call(operation, fields)
    @Synchronized override fun close() = cleanable.clean()
    private class Allocation(private var handle: Long) : Runnable {
        @Synchronized fun call(operation: String, fields: Array<out String>): Array<String> {
            check(handle != 0L) { "the NLP session is closed" }
            return NlpNative.call(handle, operation, fields)
        }
        @Synchronized override fun run() { val owned = handle; handle = 0; NlpNative.free(owned) }
    }
}
internal object NlpNative {
    init { RustLibrary.load() }
    @JvmStatic external fun isName(dictionary: Long, personal: Long, word: String): Boolean
    @JvmStatic external fun unknownWords(dictionary: Long, words: Array<String>): Array<String>
    @JvmStatic external fun create(): Long
    @JvmStatic external fun free(handle: Long)
    @JvmStatic external fun call(handle: Long, operation: String, fields: Array<out String>): Array<String>
}

/** The same native policy when the dictionary has not loaded yet. */
object NlpNames {
    fun withoutDictionary(personal: PersonalWordStore, word: String): Boolean =
        NlpNative.isName(0, personal.handle, word).also { java.lang.ref.Reference.reachabilityFence(personal) }
}
