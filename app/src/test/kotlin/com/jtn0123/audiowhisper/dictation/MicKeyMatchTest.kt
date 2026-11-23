package com.jtn0123.audiowhisper.dictation

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * The mic key in Rust (lib/rust/crates/dictation/src/mic_key.rs) does exactly what the Kotlin it
 * replaced did, as that Kotlin's golden digest records (docs/phase6-kotlin-references.md, generated
 * at e00807e9d): random runs of presses and releases, in every status, at gaps around a hold and
 * across a wrapping clock.
 */
class MicKeyMatchTest {
    @Test
    fun keysMatch() {
        val comparison = MicKeyComparison()
        comparison.constants()
        comparison.keys(Random(SEED), runs = 2_000, steps = 40)
        comparison.tape.finish()
        for (outcome in OUTCOMES) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        val OUTCOMES = listOf(
            "press IDLE: START",
            "press LISTENING: NONE",
            "press FINISHING: NONE",
            "release: NONE",
            "release: FINISH",
            "release: END_HOLD",
            "release: TOGGLE",
        )
    }
}
