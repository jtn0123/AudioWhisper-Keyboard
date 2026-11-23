package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random

/**
 * Puts the mic key in Rust ([MicKey]) up against the Kotlin it replaced, [ReferenceMicKey],
 * wherever this runs: the JVM's tests and, copied in for a check, the phone. It records each
 * difference and how often each outcome was seen, so a caller can tell the runs reached them all.
 */
class MicKeyComparison {
    /** The reference's answers, in order (docs/phase6-kotlin-references.md). */
    val tape = GoldenTape.ofReference("mic-key-runs")
    val differences = mutableListOf<String>()
    val seen = mutableMapOf<String, Int>()

    private fun check(what: String, input: String, kotlin: Any?, rust: Any?) {
        if (kotlin != rust && differences.size < MAX_DIFFERENCES) differences += "$what($input): kotlin $kotlin, rust $rust"
        tape.record("$what: $kotlin")
    }

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    fun constants() = check("HOLD_MS", "", ReferenceMicKey.HOLD_MS, MicKey.HOLD_MS)

    /**
     * Random presses and releases of a pair of mic keys sharing a clock, with gaps either side of
     * a hold, any gap at all, and clocks near where a Long wraps.
     */
    fun keys(random: Random, runs: Int, steps: Int) = repeat(runs) {
        var now = TIMES.random(random)
        val kotlin = ReferenceMicKey { now }
        val rust = MicKey { now }
        repeat(steps) { step ->
            now += GAPS.random(random).takeIf { it != 0L } ?: random.nextLong()
            if (random.nextInt(2) == 0) {
                val status = Dictation.Status.entries.random(random)
                val action = rust.press(status)
                check("press($status)", "step $step at $now", kotlin.press(status), action)
                see("press $status: $action")
            } else {
                val action = rust.release()
                check("release", "step $step at $now", kotlin.release(), action)
                see("release: $action")
            }
        }
    }

    private companion object {
        const val MAX_DIFFERENCES = 20
        const val HOLD = ReferenceMicKey.HOLD_MS
        val TIMES = listOf(0L, 1_000L, Long.MAX_VALUE - HOLD, Long.MAX_VALUE, Long.MIN_VALUE, -HOLD)
        val GAPS = listOf(0L, 1L, 80L, HOLD - 1, HOLD, HOLD + 1, 5_000L, -1L, -HOLD)
    }
}
