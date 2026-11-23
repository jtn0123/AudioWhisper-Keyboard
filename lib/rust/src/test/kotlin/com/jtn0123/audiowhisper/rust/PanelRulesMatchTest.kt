package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test

/** EmojiHistoryHelper's and QuickActionsEditorPanel's rules: golden digests of their frozen copies'
 * answers now (docs/phase6-kotlin-references.md). */
class PanelRulesMatchTest {
    @Test fun historyStrategiesLimitsPinningAndMovement() {
        val rng = Random(6307)
        val tape = GoldenTape("panel-history")
        for (ps in 0..3) for (rs in 0..3) for (pl in listOf(0, 1, 4, 100)) for (rl in listOf(0, 1, 5, 100)) {
            var state = arrayOf(intArrayOf(0, 1, 2), intArrayOf(3, 4, 5))
            repeat(300) { step ->
                val c = intArrayOf(if (step % 13 == 0) 0 else 1, ps, rs, pl, rl,
                    rng.nextInt(7), rng.nextInt(12), listOf(Int.MIN_VALUE, -100, -1, 0, 1, 100, Int.MAX_VALUE).random(rng))
                val actual = PanelRules.history(state[0], state[1], c)
                tape.answer("history", actual.map { it.toList() })
                state = actual
            }
        }
        tape.finish()
    }
    @Test fun distinctAndDragAcrossStickyDynamicHiddenAndEmptyGroups() {
        val rng = Random(6308)
        val tape = GoldenTape("panel-arrange")
        repeat(2000) {
            val dynamic = IntArray(rng.nextInt(9)) { rng.nextInt(10) }
            val hidden = IntArray(rng.nextInt(9)) { rng.nextInt(10) }
            val initial = intArrayOf(rng.nextInt(-1, 10), 0, 0, 0)
            val distinct = PanelRules.arrange(initial, dynamic, hidden)
            tape.answer("distinct", distinct.map { it.toList() })
            val ops = mutableListOf(distinct[0][0])
            repeat(30) {
                val group = rng.nextInt(3)
                // Index zero remains a valid drop target even in placeholder-only groups.
                ops.addAll(listOf(1, group, 0, 2, rng.nextInt(3), 0, 3, 0, 0))
            }
            val config = ops.toIntArray()
            tape.answer("dragged", PanelRules.arrange(config, distinct[1], distinct[2]).map { it.toList() })
        }
        // Reordering within the dynamic section, including movement past the final tile.
        for (from in 0..7) for (to in 0..7) {
            val c = intArrayOf(8, 1, 1, from, 2, 1, to)
            val d = IntArray(8) { it }
            tape.answer("moved $from $to", PanelRules.arrange(c, d, intArrayOf()).map { it.toList() })
        }
        tape.finish()
    }
}
