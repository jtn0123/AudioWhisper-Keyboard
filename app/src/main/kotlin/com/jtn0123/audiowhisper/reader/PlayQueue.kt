package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.*

/** Rust owns frame order/start/finish/stop. Kotlin holds and executes managed callbacks. */
internal class PlayQueue {
    class Heard(val started: List<() -> Unit>, val finished: Int)
    private val native = PlaybackQueue()
    private val starts = mutableMapOf<Int, () -> Unit>()
    val size: Int @Synchronized get() = native.call(PlaybackOp.COUNT)[0].toInt()
    @Synchronized fun add(id: Int, onStart: () -> Unit) { native.call(PlaybackOp.ADD, id); starts[id] = onStart }
    @Synchronized fun begins(id: Int, frame: Long) { native.call(PlaybackOp.BEGINS, id, frame) }
    @Synchronized fun ends(id: Int, frame: Long) { native.call(PlaybackOp.ENDS, id, frame) }
    @Synchronized fun heard(frame: Long): Heard {
        val result = native.call(PlaybackOp.HEARD, frame = frame)
        return Heard(result.drop(PlaybackHeard.SIZE).mapNotNull { starts.remove(it.toInt()) }, result[PlaybackHeard.FINISHED].toInt())
    }
    @Synchronized fun clear(): Int {
        val count = native.call(PlaybackOp.CLEAR)[0].toInt()
        starts.clear()
        return count
    }
}
