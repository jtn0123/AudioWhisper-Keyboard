package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/** Actual frame-head histories plus seeded out-of-order generation, stop, and late callbacks. */
class PlaybackQueueMatchTest {
    @Test fun frameTimelinesMatchTheFrozenKotlin() {
        val tape = GoldenTape.ofReference("play-queue-timelines")
        repeat(200) { seed ->
            val random = Random(seed)
            val reference = ReferencePlayQueue()
            val native = PlayQueue()
            val oldStarts = mutableListOf<Int>()
            val newStarts = mutableListOf<Int>()
            var next = 0
            var frame = 0L
            var told = 0
            repeat(250) {
                val id = if (next == 0) 0 else random.nextInt(next + 2)
                val op = random.nextInt(6)
                var answer = ""
                when (op) {
                    0 -> { val fresh = next++; reference.add(fresh) { oldStarts += fresh }; native.add(fresh) { newStarts += fresh } }
                    1 -> { reference.begins(id, frame); native.begins(id, frame) }
                    2 -> { reference.ends(id, frame); native.ends(id, frame) }
                    3 -> { frame += random.nextLong(0, 128); val a = reference.heard(frame); val b = native.heard(frame)
                        a.started.forEach { it() }; b.started.forEach { it() }; assertEquals(a.finished, b.finished); answer = "${a.finished}" }
                    4 -> { val cleared = reference.clear(); assertEquals(cleared, native.clear()); answer = "$cleared" }
                    5 -> { val a = reference.heard(frame); val b = native.heard(frame)
                        a.started.forEach { it() }; b.started.forEach { it() }; assertEquals(a.finished, b.finished); answer = "${a.finished}" }
                }
                assertEquals(reference.size, native.size)
                assertEquals(oldStarts, newStarts)
                // Each step: what was done, what it gave, the queue's size, and the blocks it started.
                tape.record(listOf("$op", answer, "${reference.size}", oldStarts.subList(told, oldStarts.size).joinToString(",")))
                told = oldStarts.size
            }
        }
        tape.finish()
    }
}
