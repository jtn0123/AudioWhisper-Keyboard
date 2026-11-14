package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.PlaybackQueue
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Frozen Speaker.kt live/start/last-sentence decisions, without constructing Android TTS. */
class SystemProgressMatchTest {
    private class ReferenceSystemQueue {
        private val live = mutableSetOf<Int>()
        private val starts = mutableSetOf<Int>()
        fun call(op: Int, id: Int, text: String): LongArray {
            var started = -1
            var ended = 0
            when (op) {
                0 -> { live += id; starts += id }
                1 -> text.substringBefore(':').toIntOrNull()?.let { if (starts.remove(it)) { started = it; ended = 1 } }
                2 -> if (text.substringAfter(':') == "last") {
                    text.substringBefore(':').toIntOrNull()?.let { starts.remove(it); if (live.remove(it)) { started = it; ended = 1 } }
                }
                3 -> { ended = live.size; live.clear(); starts.clear() }
            }
            return longArrayOf(started.toLong(), ended.toLong(), live.size.toLong())
        }
    }
    @Test fun progressLateCallbacksAndStopMatch() {
        repeat(200) { seed ->
            val rng = Random(seed)
            val reference = ReferenceSystemQueue()
            PlaybackQueue().use { native ->
                repeat(250) {
                    val op = rng.nextInt(5)
                    val id = listOf(rng.nextInt(20), -1, Int.MIN_VALUE, Int.MAX_VALUE)[rng.nextInt(4)]
                    val text = when (rng.nextInt(5)) {
                        0 -> "$id:last"
                        1 -> "$id:0"
                        2 -> "$id"
                        3 -> "bad:last"
                        else -> "$id:2:last"
                    }
                    assertContentEquals(reference.call(op, id, text), native.system(op, id, text))
                }
            }
        }
    }
}
