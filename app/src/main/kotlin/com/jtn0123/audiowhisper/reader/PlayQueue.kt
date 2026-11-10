package com.jtn0123.audiowhisper.reader

/**
 * The blocks [NeuralVoice] has queued, placed on the audio output's timeline: a block's audio
 * starts at the frame where its first sample was written and ends where its last one was. As the
 * output plays on, [heard] says which blocks the listener has started hearing and which are over,
 * so the reader's follow-along and look-ahead go by what is heard, not by what was generated
 * (which runs seconds ahead).
 *
 * Blocks are written in the order they were queued. Safe to use from several threads.
 */
internal class PlayQueue {
    /** What changed since the last [heard]: blocks now being heard, and how many are over. */
    class Heard(val started: List<() -> Unit>, val finished: Int)

    private class Block(val id: Int, var onStart: (() -> Unit)?) {
        var start: Long? = null
        var end: Long? = null
    }

    private val blocks = ArrayDeque<Block>()

    /** Blocks queued and not over yet. */
    val size: Int @Synchronized get() = blocks.size

    @Synchronized
    fun add(id: Int, onStart: () -> Unit) {
        blocks.addLast(Block(id, onStart))
    }

    /** Block [id]'s first sample goes out at [frame]. */
    @Synchronized
    fun begins(id: Int, frame: Long) {
        find(id)?.let { if (it.start == null) it.start = frame }
    }

    /** Block [id]'s last sample went out before [frame]. One with no audio begins and ends there. */
    @Synchronized
    fun ends(id: Int, frame: Long) {
        val block = find(id) ?: return
        if (block.start == null) block.start = frame
        block.end = frame
    }

    /** The output has played everything before [frame]. */
    @Synchronized
    fun heard(frame: Long): Heard {
        val started = mutableListOf<() -> Unit>()
        for (block in blocks) {
            val start = block.start ?: break
            val over = block.end?.let { frame >= it } ?: false
            if (frame <= start && !over) break
            block.onStart?.let(started::add)
            block.onStart = null
        }
        var finished = 0
        while (blocks.firstOrNull()?.end?.let { frame >= it } == true) {
            blocks.removeFirst()
            finished++
        }
        return Heard(started, finished)
    }

    /** Drops every block; returns how many there were. */
    @Synchronized
    fun clear(): Int = blocks.size.also { blocks.clear() }

    private fun find(id: Int): Block? = blocks.firstOrNull { it.id == id }
}
