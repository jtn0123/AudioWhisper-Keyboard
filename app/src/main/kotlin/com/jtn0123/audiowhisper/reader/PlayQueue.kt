package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.PlaybackQueue

/** Rust owns frame order/start/finish/stop. Kotlin holds and executes managed callbacks. */
internal class PlayQueue {
    class Heard(val started: List<() -> Unit>, val finished: Int)
    private val native = PlaybackQueue()
    private val starts = mutableMapOf<Int, () -> Unit>()
    val size: Int @Synchronized get() = native.call(5)[0].toInt()
    @Synchronized fun add(id: Int, onStart: () -> Unit) { native.call(0, id); starts[id] = onStart }
    @Synchronized fun begins(id: Int, frame: Long) { native.call(1, id, frame) }
    @Synchronized fun ends(id: Int, frame: Long) { native.call(2, id, frame) }
    @Synchronized fun heard(frame: Long): Heard {
        val result = native.call(3, frame = frame)
        return Heard(result.drop(1).mapNotNull { starts.remove(it.toInt()) }, result[0].toInt())
    }
    @Synchronized fun clear(): Int {
        val count = native.call(4)[0].toInt()
        starts.clear()
        return count
    }
}
