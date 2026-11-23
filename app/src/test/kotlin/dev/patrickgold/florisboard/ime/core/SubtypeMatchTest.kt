package dev.patrickgold.florisboard.ime.core

import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Seeded traces of the keyboard's subtypes, main's (frozen in ReferenceKeyboardSubtypes.kt and,
 * for its key graph row, ReferenceSubtypeBuildRow.kt) against the branch's, where Rust
 * (lib/rust/crates/imesubtype) holds the list and the active subtype. See
 * docs/phase3-theme-subtype.md.
 */
class SubtypeMatchTest {
    @Test fun mainsSubtypesAndRustsKeepTheSameListActiveSubtypeSettingsAndKeyboards() {
        for (seed in SEEDS) {
            val trace = SubtypeTrace(seed)
            trace.run(STATES)
            println("Subtype trace seed $seed: $STATES states identical; ${trace.seen.toSortedMap()}")
            for (what in COVERED) assertTrue((trace.seen[what] ?: 0) > 0, "seed $seed never saw $what")
        }
    }

    private companion object {
        const val STATES = 50_000
        val SEEDS = listOf(20261122L, 11L, 31337L)
        val COVERED = listOf("add", "add a duplicate id", "add a copy", "remove", "modify", "reorder", "move", "rewrite",
            "next", "previous", "switch by id", "active id stored", "edit stored an active id", "currency sets loaded",
            "number row", "number hints", "resources changed", "process death", "process death mid-flight",
            "edits conflated", "compared mid-flight", "failed alike", "stored list", "stored active", "duplicate ids",
            "right-to-left subtype", "list of 0", "list of 5", "fallback currency set", "loaded currency set", "keyboard from the cache",
            "keyboard built", "keyboard computed")
    }
}
