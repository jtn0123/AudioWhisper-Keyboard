package com.jtn0123.audiowhisper.reader

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/** Actual frame-head histories plus seeded out-of-order generation, stop, and late callbacks. */
class PlaybackQueueMatchTest {
    @Test fun frameTimelinesMatchTheFrozenKotlin() {
        repeat(200) { seed ->
            val random = Random(seed)
            val reference = ReferencePlayQueue()
            val native = PlayQueue()
            val oldStarts = mutableListOf<Int>()
            val newStarts = mutableListOf<Int>()
            var next = 0
            var frame = 0L
            repeat(250) {
                val id = if (next == 0) 0 else random.nextInt(next + 2)
                when (random.nextInt(6)) {
                    0 -> { val fresh = next++; reference.add(fresh) { oldStarts += fresh }; native.add(fresh) { newStarts += fresh } }
                    1 -> { reference.begins(id, frame); native.begins(id, frame) }
                    2 -> { reference.ends(id, frame); native.ends(id, frame) }
                    3 -> { frame += random.nextLong(0, 128); val a = reference.heard(frame); val b = native.heard(frame)
                        a.started.forEach { it() }; b.started.forEach { it() }; assertEquals(a.finished, b.finished) }
                    4 -> { assertEquals(reference.clear(), native.clear()) }
                    5 -> { val a = reference.heard(frame); val b = native.heard(frame)
                        a.started.forEach { it() }; b.started.forEach { it() }; assertEquals(a.finished, b.finished) }
                }
                assertEquals(reference.size, native.size)
                assertEquals(oldStarts, newStarts)
            }
        }
    }
}
