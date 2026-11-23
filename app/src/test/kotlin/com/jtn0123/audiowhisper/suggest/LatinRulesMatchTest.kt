package com.jtn0123.audiowhisper.suggest

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The suggester and the word learner in Rust give exactly what the Kotlin they replaced gave
 * ([ReferenceLatin]), through [LatinComparison]: random strips and spell checks, and random runs
 * of finished, swiped and undone words. The same comparison runs on the emulator, whose Java
 * judges some characters differently.
 */
class LatinRulesMatchTest {
    private fun assertNone(name: String, reached: List<String>, body: LatinComparison.() -> Unit) {
        val tape = GoldenTape.ofReference("latin-rules-$name")
        val comparison = LatinComparison(sigmaExact = false, tape).apply(body)
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} differ")
        tape.finish()
        for (outcome in reached) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times")
        }
    }

    @Test
    fun stripsAndSpellChecksMatch() = assertNone("strips", STRIP_OUTCOMES) { suggester(Random(SEED), 40_000) }

    @Test
    fun learnersStoreTheSame() = assertNone("learners", LEARNER_OUTCOMES) { learner(Random(SEED), runs = 1_500, steps = 60) }

    private companion object {
        const val SEED = 20261108L

        /** Each outcome must come up this often, or the run wasn't testing it. */
        const val MIN_SEEN = 100

        val STRIP_OUTCOMES = listOf(
            "auto-correction", "no auto-correction", "more than the typed word", "predictions",
            "valid", "misspelled", "confident", "spell threw",
        )

        val LEARNER_OUTCOMES = listOf(
            "finished", "swiped kept the last", "undone", "an address piece", "an address piece took one back",
        )
    }
}
