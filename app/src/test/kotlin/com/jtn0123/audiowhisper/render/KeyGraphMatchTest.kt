package com.jtn0123.audiowhisper.render

import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Seeded traces of keyboards, each computed by main's Kotlin key graph (frozen in
 * ReferenceKeyGraph.kt) and by the Rust graph (lib/rust/crates/keygraph) side by side, over the
 * bundled layouts and popup mappings: every listed key, field by field, its popup keys for two
 * hint settings, and the layout must match exactly. See docs/phase3-key-graph.md.
 */
class KeyGraphMatchTest {
    @Test fun mainsKeyGraphAndTheRustGraphComputeTheSameKeys() {
        for (seed in SEEDS) {
            val trace = KeyGraphTrace(seed)
            trace.run(STATES)
            println("Key graph trace seed $seed: $STATES states identical; ${trace.seen.toSortedMap()}")
            for (mode in MODES) assertTrue((trace.seen["mode $mode"] ?: 0) > 0, "seed $seed never computed $mode")
            for (what in COVERED) assertTrue((trace.seen[what] ?: 0) > 0, "seed $seed never saw $what")
        }
    }

    private companion object {
        const val STATES = 50_000
        val SEEDS = listOf(20261122L, 7L, 424242L)
        val MODES = listOf(KeyboardMode.CHARACTERS, KeyboardMode.SYMBOLS, KeyboardMode.SYMBOLS2, KeyboardMode.NUMERIC,
            KeyboardMode.NUMERIC_ADVANCED, KeyboardMode.PHONE, KeyboardMode.PHONE2)
        val COVERED = listOf("hidden key", "disabled key", "label", "hinted label", "multi-text key", "own popup",
            "popup keys", "popup hint", "layout", "layout of nothing", "recreated", "abandoned with questions",
            "abandoned after an answer", "right-to-left locale", "broken layout", "misnamed modifier",
            "missing currency slots", "synthetic layouts", "synthetic popup mapping", "synthetic modifier") +
            InputShiftState.entries.map { "shift $it" } + KeyVariation.entries.map { "variation $it" } +
            (0..7).map { "ime action $it" } + (1..4).map { "input class $it" }
    }
}
