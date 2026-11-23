package dev.patrickgold.florisboard.ime.nlp

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.suggest.LatinDictionaryEngine
import com.jtn0123.audiowhisper.suggest.TestDictionaries
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Seeded 50,000-event traces of synthetic typing through the keyboard's one call per event
 * ([TypingEvents]), over native stores and the shipped dictionary. After every event, what Android
 * was asked to do (kept-word writes and save timers, in order), what went into the editor, the
 * Smartbar's row, every store, and what was written to disk go on a golden tape per seed, across
 * save timers that fire, saves lost when the process dies, and service recreation. The goldens
 * held the records main's fan-out left on the same traces until the stale strip tap was fixed;
 * they now hold the session's records with that fix, rewritten after review
 * (docs/phase3-typing-fan-out.md, docs/phase6-kotlin-references.md).
 */
class TypingTraceMatchTest {
    private fun engine(): LatinDictionaryEngine = TestDictionaries.shipped.let {
        requireNotNull(LatinDictionaryEngine.open(it.path, 0, it.length(), "en_US"))
    }

    @Test fun theEventPlanLeavesTheReviewedTrace() {
        for (seed in SEEDS) {
            engine().use { engine ->
                val trace = GoldenTape.check("typing-trace-$seed") { TypingTrace(seed, engine, it).apply { run(EVENTS) } }
                println("Typing trace seed $seed: $EVENTS events as reviewed")
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
            }
        }
    }

    private companion object {
        const val EVENTS = 50_000
        val SEEDS = listOf(20261122L, 7L, 424242L)
    }
}
