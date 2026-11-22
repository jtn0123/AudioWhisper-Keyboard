package dev.patrickgold.florisboard.ime.nlp

import com.jtn0123.audiowhisper.suggest.LatinDictionaryEngine
import com.jtn0123.audiowhisper.suggest.TestDictionaries
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Seeded 50,000-event traces of synthetic typing, run through main's fan-out (frozen in
 * [ReferenceTypingFanOut]) and the branch's one call per event ([TypingEvents]) side by side,
 * each over its own native stores and dictionary. After every event the two must agree on what
 * Android was asked to do (kept-word writes and save timers, in order), what went into the editor,
 * the Smartbar's row, every store, and what was written to disk, across save timers that fire,
 * saves lost when the process dies, and service recreation. See docs/phase3-typing-fan-out.md.
 */
class TypingTraceMatchTest {
    private fun engine(): LatinDictionaryEngine = TestDictionaries.shipped.let {
        requireNotNull(LatinDictionaryEngine.open(it.path, 0, it.length(), "en_US"))
    }

    @Test fun mainsFanOutAndTheEventPlanLeaveTheSameTrace() {
        for (seed in SEEDS) {
            engine().use { first -> engine().use { second ->
                val trace = TypingTrace(seed, first to second)
                val digest = trace.run(EVENTS)
                println("Typing trace seed $seed: $EVENTS events identical, digest $digest")
                println("  events ${trace.kinds}")
                println("  effects ${trace.effects}")
                println("  changed ${trace.changes}")
                for (kind in listOf("letter", "space", "punctuation", "backspace", "pick", "swipe", "undo", "field", "recreate", "death")) {
                    assertTrue((trace.kinds[kind] ?: 0) > 0, "seed $seed never reached $kind")
                }
                for (effect in listOf("kept", "save", "fired", "undone", "restarted")) {
                    assertTrue((trace.effects[effect] ?: 0) > 0, "seed $seed never asked for $effect")
                }
                for (part in listOf("candidates", "smartbar", "learned", "kept", "aim", "counters", "disk")) {
                    assertTrue((trace.changes[part] ?: 0) > 0, "seed $seed never changed $part")
                }
            } }
        }
    }

    private companion object {
        const val EVENTS = 50_000
        val SEEDS = listOf(20261122L, 7L, 424242L)
    }
}
