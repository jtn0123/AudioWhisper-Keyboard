package com.jtn0123.audiowhisper.rust

/** Complete editor messages use UTF-16. No platform callbacks run while Rust holds its mutex. */
class EditorSession : AutoCloseable {
    private val allocation = Allocation(EditorNative.create())
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    @Synchronized fun call(operation: String, vararg fields: String): Array<String> =
        allocation.call(operation, fields)
    @Synchronized override fun close() = cleanable.clean()
    private class Allocation(private var handle: Long) : Runnable {
        @Synchronized fun call(operation: String, fields: Array<out String>): Array<String> {
            check(handle != 0L) { "the editor is closed" }
            return EditorNative.call(handle, operation, fields)
        }
        @Synchronized override fun run() { val owned = handle; handle = 0; EditorNative.free(owned) }
    }
}
object EditorAttributes {
    fun capsAt(text: String, cursor: Int, input: Int): Int = EditorNative.attributes(arrayOf("6",cursor.toString(),text,input.toString()))[0].toInt()
    fun enterAction(input: Int, options: Int, shift: Boolean): Int = query(7,input,options,if (shift) 1 else 0)[0]
    fun keyboard(input: Int, options: Int, suggestions: Boolean, incognito: Int, force: Boolean): IntArray =
        query(5,input,options,if (suggestions) 1 else 0,incognito,if (force) 1 else 0)
    fun input(raw: Int): IntArray = query(0,raw)
    fun options(raw: Int): IntArray = query(1,raw)
    fun caps(raw: Int): Int = query(2,raw)[0]
    fun capsFlags(mode: Int): Int = query(3,mode)[0]
    fun rich(raw: Int, start: Int, end: Int): Boolean = query(4,raw,start,end)[0] != 0
    private fun query(vararg values: Int): IntArray = EditorNative.attributes(values.map(Int::toString).toTypedArray()).map(String::toInt).toIntArray()
}
internal object EditorNative {
    init { RustLibrary.load() }
    @JvmStatic external fun create(): Long
    @JvmStatic external fun free(handle: Long)
    @JvmStatic external fun call(handle: Long, operation: String, fields: Array<out String>): Array<String>
    @JvmStatic external fun attributes(fields: Array<String>): Array<String>
}
