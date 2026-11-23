package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test

/**
 * Actual frame-head histories plus seeded out-of-order generation, stop, and late callbacks, each
 * step checked against the frozen Kotlin's in golden/play-queue-timelines.txt.
 */
class PlaybackQueueMatchTest {
    @Test fun frameTimelinesMatchTheFrozenKotlin() = GoldenTape.check("play-queue-timelines") { tape ->
        repeat(200) { seed ->
            val random = Random(seed)
            val native = PlayQueue()
            val starts = mutableListOf<Int>()
            var next = 0
            var frame = 0L
            var told = 0
            repeat(250) {
                val id = if (next == 0) 0 else random.nextInt(next + 2)
                val op = random.nextInt(6)
                var answer = ""
                when (op) {
                    0 -> { val fresh = next++; native.add(fresh) { starts += fresh } }
                    1 -> native.begins(id, frame)
                    2 -> native.ends(id, frame)
                    3 -> { frame += random.nextLong(0, 128); val heard = native.heard(frame); heard.started.forEach { it() }; answer = "${heard.finished}" }
                    4 -> answer = "${native.clear()}"
                    5 -> { val heard = native.heard(frame); heard.started.forEach { it() }; answer = "${heard.finished}" }
                }
                // Each step: what was done, what it gave, the queue's size, and the blocks it started.
                tape.record(listOf("$op", answer, "${native.size}", starts.subList(told, starts.size).joinToString(",")))
                told = starts.size
            }
        }
    }
}
