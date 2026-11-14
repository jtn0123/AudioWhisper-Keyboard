package com.jtn0123.audiowhisper.rust

/** Ordered session effects; owns no Android resources and never calls back while locked. */
class DictationSessionPlan : AutoCloseable {
    private val allocation = Allocation(DictationSessionNative.create())
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    @Synchronized fun call(operation: String, now: Long, generation: Int = 0, vararg fields: String): Array<String> =
        allocation.call(operation, now, generation, fields)
    @Synchronized fun speech(generation: Int, detected: Boolean) = allocation.speech(generation, detected)
    @Synchronized fun model(operation: String): Int = allocation.model(operation)
    @Synchronized fun claimPreview(): Boolean = allocation.preview(true)
    @Synchronized fun completePreview() { allocation.preview(false) }
    @Synchronized override fun close() = cleanable.clean()
    private class Allocation(private var handle: Long) : Runnable {
        @Synchronized fun call(operation: String, now: Long, generation: Int, fields: Array<out String>): Array<String> {
            check(handle != 0L) { "the dictation session is closed" }
            return DictationSessionNative.call(handle, operation, now, generation, fields)
        }
        @Synchronized fun speech(generation: Int, detected: Boolean) {
            check(handle != 0L) { "the dictation session is closed" }
            DictationSessionNative.speech(handle, generation, detected)
        }
        @Synchronized fun model(operation: String): Int {
            check(handle != 0L) { "the dictation session is closed" }
            return DictationSessionNative.model(handle, operation)
        }
        @Synchronized fun preview(claim: Boolean): Boolean {
            check(handle != 0L) { "the dictation session is closed" }
            return DictationSessionNative.preview(handle, claim)
        }
        @Synchronized override fun run() { val owned = handle; handle = 0; DictationSessionNative.free(owned) }
    }
    companion object {
        fun releaseAt(idle: Long, now: Long, shown: Boolean): Long = DictationSessionNative.releaseAt(idle, now, shown)
        fun ending(typed: Boolean, failures: Int): String = DictationSessionNative.ending(typed, failures)
    }
}
internal object DictationSessionNative {
    init { RustLibrary.load() }
    @JvmStatic external fun speech(handle: Long, generation: Int, detected: Boolean)
    @JvmStatic external fun model(handle: Long, operation: String): Int
    @JvmStatic external fun preview(handle: Long, claim: Boolean): Boolean
    @JvmStatic external fun create(): Long
    @JvmStatic external fun free(handle: Long)
    @JvmStatic external fun call(handle: Long, operation: String, now: Long, generation: Int, fields: Array<out String>): Array<String>
    @JvmStatic external fun releaseAt(idle: Long, now: Long, shown: Boolean): Long
    @JvmStatic external fun ending(typed: Boolean, failures: Int): String
}
