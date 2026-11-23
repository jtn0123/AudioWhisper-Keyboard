package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.MicKeyState
import kotlin.random.Random

/**
 * Runs the mic key in Rust ([MicKeyState]) through the questions the Kotlin it replaced answered,
 * and records each answer on [tape], checked against that Kotlin's golden digest
 * (docs/phase6-kotlin-references.md). It counts how often each outcome was seen, so a caller can
 * tell the runs reached them all.
 */
class MicKeyComparison {
    val tape = GoldenTape("mic-key-runs")
    val seen = mutableMapOf<String, Int>()

    private fun see(what: String) {
        seen[what] = (seen[what] ?: 0) + 1
    }

    fun constants() = tape.record("HOLD_MS: ${MicKeyState.holdMs}")

    /**
     * Random presses and releases of a mic key, with gaps either side of a hold, any gap at all,
     * and clocks near where a Long wraps.
     */
    fun keys(random: Random, runs: Int, steps: Int) = repeat(runs) {
        var now = TIMES.random(random)
        val rust = MicKeyState()
        repeat(steps) {
            now += GAPS.random(random).takeIf { it != 0L } ?: random.nextLong()
            if (random.nextInt(2) == 0) {
                val status = MicKeyState.Status.entries.random(random)
                val action = rust.press(status, now)
                tape.record("press($status): $action")
                see("press $status: $action")
            } else {
                val action = rust.release(now)
                tape.record("release: $action")
                see("release: $action")
            }
        }
    }

    private companion object {
        const val HOLD = 300L
        val TIMES = listOf(0L, 1_000L, Long.MAX_VALUE - HOLD, Long.MAX_VALUE, Long.MIN_VALUE, -HOLD)
        val GAPS = listOf(0L, 1L, 80L, HOLD - 1, HOLD, HOLD + 1, 5_000L, -1L, -HOLD)
    }
}
