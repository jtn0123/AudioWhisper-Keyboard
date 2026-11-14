package com.jtn0123.audiowhisper.rust

/** Only platform facts. Rust requests children in reading order and decides which text to read. */
interface ReaderTree {
    fun facts(id: Int): IntArray
    fun className(id: Int): String
    fun text(id: Int): String
    fun child(id: Int, index: Int): Int
}
object ReaderWalk {
    init { RustLibrary.load() }
    @JvmStatic external fun walk(tree: ReaderTree, visible: IntArray): IntArray
    @JvmStatic external fun followConfig(): LongArray
}
/** Reader policy constants; Android only schedules the requested waits and queue capacity. */
object ReaderTiming {
    private val values = ReaderWalk.followConfig()
    val maxTurns: Int = values[0].toInt()
    val turnMs: Long = values[1]
    val lookahead: Int = values[2].toInt()
    val followMs: Long = values[3]
}
class PlaybackQueue : AutoCloseable {
    private val allocation = Allocation(PlaybackNative.create())
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    @Synchronized fun call(op: Int, id: Int = 0, frame: Long = 0): LongArray = allocation.call(op, id, frame)
    @Synchronized fun system(op: Int, id: Int = 0, text: String = ""): LongArray = allocation.system(op, id, text)
    @Synchronized override fun close() = cleanable.clean()
    private class Allocation(private var handle: Long) : Runnable {
        @Synchronized fun call(op: Int, id: Int, frame: Long): LongArray {
            check(handle != 0L) { "Playback queue closed" }
            return PlaybackNative.call(handle, op, id, frame)
        }
        @Synchronized fun system(op: Int, id: Int, text: String): LongArray {
            check(handle != 0L) { "Playback queue closed" }
            return PlaybackNative.system(handle, op, id, text)
        }
        @Synchronized override fun run() { val owned=handle; handle=0; PlaybackNative.free(owned) }
    }
}
internal object PlaybackNative {
    init { RustLibrary.load() }
    @JvmStatic external fun create(): Long
    @JvmStatic external fun call(handle: Long, op: Int, id: Int, frame: Long): LongArray
    @JvmStatic external fun system(handle: Long, op: Int, id: Int, text: String): LongArray
    @JvmStatic external fun free(handle: Long)
}
