package com.jtn0123.audiowhisper.rust

/** A shared key lifecycle. Native calls finish before any Android work or receiver callback. */
class InputDispatch : AutoCloseable {
    private val allocation = Allocation(InputNative.create())
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    @Synchronized fun call(operation: Int, vararg fields: Long): LongArray = allocation.call(operation, fields)
    @Synchronized override fun close() = cleanable.clean()
    private class Allocation(private var handle: Long) : Runnable {
        @Synchronized fun call(operation: Int, fields: LongArray): LongArray {
            check(handle != 0L) { "the input dispatcher is closed" }
            return InputNative.call(handle, operation, fields)
        }
        @Synchronized override fun run() {val owned=handle;handle=0;InputNative.free(owned)}
    }
}
object InputRules {
    init { RustLibrary.load() }
    @JvmStatic external fun control(fields: LongArray, text: String = ""): LongArray
    @JvmStatic external fun route(fields: LongArray): LongArray
    @JvmStatic external fun feedback(fields: LongArray): LongArray
    @JvmStatic external fun visual(fields: LongArray): Int
    @JvmStatic external fun hold(code: Int, popup: Boolean, uninterrupted: Boolean, spaceAction: Boolean): Int
    @JvmStatic external fun pointer(fields: FloatArray): Boolean
}
internal object InputNative {
    init { RustLibrary.load() }
    @JvmStatic external fun create(): Long
    @JvmStatic external fun free(handle: Long)
    @JvmStatic external fun call(handle: Long, operation: Int, fields: LongArray): LongArray
}
