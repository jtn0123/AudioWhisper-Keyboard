package dev.patrickgold.florisboard.ime.popup

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The long-press popup's rules in Rust (lib/rust/crates/popup) do exactly what the Kotlin they
 * replaced did ([ReferencePopupGrid], [ReferenceMutablePopupSet]): random extended popups over
 * odd bounds and keyboards narrower than them, fingers either side of each edge, and random runs
 * of filling, merging and ordering popup sets in every hint mode.
 */
class PopupMatchTest {
    @Test
    fun popupsMatch() {
        val comparison = PopupComparison()
        val random = Random(SEED)
        comparison.layouts(random, count = 100_000)
        comparison.pointers(random, count = 200_000)
        comparison.elements(random, count = 20_000)
        comparison.popupSets(random, runs = 6_000, steps = 40)
        assertEquals(emptyList<String>(), comparison.differences)
        val rare = OUTCOMES.filter { (comparison.seen[it] ?: 0) < MIN_SEEN }.map { "$it: ${comparison.seen[it]}" }
        assertEquals(emptyList<String>(), rare, "seen ${comparison.seen}")
        assertEquals(emptySet(), comparison.seen.keys - OUTCOMES.toSet(), "outcomes not listed")
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        val ORDERS = listOf("HINT_PRIORITY", "ACCENT_PRIORITY", "SMART_PRIORITY").let { modes ->
            modes.flatMap { s -> modes.map { n -> "order: both $s $n" } }.filter { !it.endsWith("ACCENT_PRIORITY ACCENT_PRIORITY") } +
                listOf("main", "relevant", "hint").map { "order: both ACCENT_PRIORITY ACCENT_PRIORITY $it" } +
                listOf("symbol", "number").flatMap { hint ->
                    listOf("$hint HINT_PRIORITY", "$hint SMART_PRIORITY") + listOf("main", "relevant", "hint").map { "$hint ACCENT_PRIORITY $it" }
                }.map { "order: $it" } +
                "order: no hints"
        }
        val POINTERS = listOf("left", "right").flatMap { side ->
            listOf("1", "0").flatMap { row -> listOf("end", "start", "before", "after").map { "pointer: $side $row $it" } }
        } + listOf("pointer: off: y", "pointer: off: left x", "pointer: off: right x")
        val OUTCOMES = listOf(
            "extend: none", "extend: one row", "extend: two rows odd", "extend: two rows even", "extend: left", "extend: right",
            "extend: offset cut", "extend: two: right", "extend: two: left", "extend: three: sides",
            "extend: three: right", "extend: three: left",
            "element: none", "element: last row", "element: first row",
            "merge: skips a key", "merge: adds main", "merge: sets main",
        ) + POINTERS + ORDERS
    }
}
