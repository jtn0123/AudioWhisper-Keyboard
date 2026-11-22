package com.jtn0123.audiowhisper.rust

import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** Copied JNI packets within one complete editor round trip. */
interface EditorCalls {
    fun call(operation: EditorOp, vararg fields: String): Array<String>
}

/** Complete editor messages use UTF-16. No platform callbacks run while Rust holds its mutex. */
class EditorSession : EditorCalls, AutoCloseable {
    private val allocation = Allocation(EditorNative.create())
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    private val gate = Mutex()
    private val direct = object : EditorCalls {
        override fun call(operation: EditorOp, vararg fields: String) = allocation.call(operation.code, fields)
    }
    override fun call(operation: EditorOp, vararg fields: String): Array<String> = runBlocking {
        gate.withLock { direct.call(operation, *fields) }
    }
    /** Own the pending native query across suspending platform analysis and all plan effects. */
    suspend fun <T> transaction(block: suspend (EditorCalls) -> T): T = gate.withLock {
        try { block(direct) } finally { java.lang.ref.Reference.reachabilityFence(this) }
    }
    /**
     * Whether a cursor update (selection, composing region) reports one of the keyboard's own edits that it has
     * since followed with more: news of the past, typed faster than the app reports.
     */
    fun isLate(selectionStart: Int, selectionEnd: Int, composingStart: Int, composingEnd: Int): Boolean =
        WireFlag.decode(call(EditorOp.LATE, *cursor(selectionStart, selectionEnd, composingStart, composingEnd))[0])
    override fun close() = runBlocking { gate.withLock { cleanable.clean() } }
    private class Allocation(private var handle: Long) : Runnable {
        @Synchronized fun call(operation: String, fields: Array<out String>): Array<String> {
            check(handle != 0L) { "the editor is closed" }
            return EditorNative.call(handle, operation, fields)
        }
        @Synchronized override fun run() { val owned = handle; handle = 0; EditorNative.free(owned) }
    }
    companion object {
        /** An [EditorCursor] row. */
        fun cursor(selectionStart: Int, selectionEnd: Int, composingStart: Int, composingEnd: Int) = with(EditorCursor) { row(
            SELECTION_START to "$selectionStart", SELECTION_END to "$selectionEnd", COMPOSING_START to "$composingStart", COMPOSING_END to "$composingEnd") }
    }
}
object EditorAttributes {
    fun capsAt(text: String, cursor: Int, input: Int): Int = query(EditorAttributeOp.CAPS_AT, "$cursor", text, "$input")[0]
    fun enterAction(input: Int, options: Int, shift: Boolean): Int =
        query(EditorAttributeOp.ENTER_ACTION, "$input", "$options", WireFlag.encode(shift))[0]
    /** An [EditorKeyboard] row. */
    fun keyboard(input: Int, options: Int, suggestions: Boolean, incognito: Int, force: Boolean): IntArray =
        query(EditorAttributeOp.KEYBOARD, *with(EditorKeyboardRequest) { row(INPUT_TYPE to "$input", IME_OPTIONS to "$options",
            SUGGESTIONS to WireFlag.encode(suggestions), INCOGNITO_MODE to "$incognito", FORCE_INCOGNITO to WireFlag.encode(force)) })
    /** An [EditorInputType] row. */
    fun input(raw: Int): IntArray = query(EditorAttributeOp.INPUT, "$raw")
    /** An [EditorImeOptions] row. */
    fun options(raw: Int): IntArray = query(EditorAttributeOp.OPTIONS, "$raw")
    fun caps(raw: Int): Int = query(EditorAttributeOp.CAPS, "$raw")[0]
    fun capsFlags(mode: Int): Int = query(EditorAttributeOp.CAPS_FLAGS, "$mode")[0]
    fun rich(raw: Int, start: Int, end: Int): Boolean = query(EditorAttributeOp.RICH, "$raw", "$start", "$end")[0] != 0
    private fun query(op: EditorAttributeOp, vararg values: String): IntArray =
        EditorNative.attributes(arrayOf("${op.code}", *values)).map(String::toInt).toIntArray()
}
