package dev.patrickgold.florisboard.ime.text.composing

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Telex and generic rule composers match the original Kotlin implementation through JNI. */
class ComposingMatchTest {
    private fun comparison(tape: GoldenTape) = ComposingComparison(sigmaExact = false, File(COMPOSERS).readText(), tape)

    private fun assertMatch(comparison: ComposingComparison, outcomes: List<String>) {
        assertEquals(emptyList(), comparison.differences)
        for (outcome in outcomes) {
            val seen = comparison.seen[outcome] ?: 0
            assertTrue(seen >= MIN_SEEN, "$outcome only $seen times, of ${comparison.seen}")
        }
    }

    @Test
    fun telexMatches() {
        val tape = GoldenTape.ofReference("composing-telex")
        val comparison = comparison(tape)
        comparison.telex(Random(SEED), runs = 2_000, steps = 40)
        tape.finish()
        assertMatch(comparison, TELEX_OUTCOMES.map { "telex: $it" })
    }

    @Test
    fun randomRulesMatch() {
        val tape = GoldenTape.ofReference("composing-rules")
        val comparison = comparison(tape)
        comparison.randomRules(Random(SEED), tables = 4_000, steps = 20)
        tape.finish()
        assertMatch(comparison, RULES_OUTCOMES.map { "rules: $it" })
    }

    @Test
    fun everyUnitMatches() {
        val tape = GoldenTape.ofReference("composing-every-unit")
        val comparison = comparison(tape)
        comparison.everyUnit()
        tape.finish()
        assertMatch(comparison, listOf("telex: keep"))
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        const val COMPOSERS = "src/main/assets/ime/keyboard/org.florisboard.composers/extension.json"
        val TELEX_OUTCOMES = listOf("keep", "delete 0", "delete 0, upper", "delete 1", "delete 1, upper", "delete 2", "delete 2, upper")
        val RULES_OUTCOMES = TELEX_OUTCOMES + listOf("delete -1", "delete -1, upper", "no table")
    }
}
