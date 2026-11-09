package dev.patrickgold.florisboard.ime.text.composing

import java.io.File
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * The text composers in Rust ([HangulUnicode], [KanaUnicode] and [WithRules], which call
 * lib/rust/crates/composing) do exactly what the Kotlin they replaced did: Korean, kana and
 * Vietnamese typed keystroke by keystroke, odd keystrokes after every kind of character, every
 * UTF-16 unit, and random rule tables whose endings are empty, overlap or change length with
 * their case.
 */
class ComposingMatchTest {
    private fun comparison() = ComposingComparison(sigmaExact = false, File(COMPOSERS).readText())

    private fun assertMatch(comparison: ComposingComparison, outcomes: List<String>) {
        assertEquals(emptyList(), comparison.differences)
        for (outcome in outcomes) {
            val seen = comparison.seen[outcome] ?: 0
            assertTrue(seen >= MIN_SEEN, "$outcome only $seen times, of ${comparison.seen}")
        }
    }

    @Test
    fun hangulMatches() {
        val comparison = comparison()
        comparison.hangul(Random(SEED), runs = 2_000, steps = 40)
        assertMatch(comparison, HANGUL_OUTCOMES.map { "hangul: $it" })
    }

    @Test
    fun kanaMatches() {
        val comparison = comparison()
        comparison.kana(Random(SEED), runs = 2_000, steps = 40)
        assertMatch(comparison, KANA_OUTCOMES.map { "kana: $it" })
    }

    @Test
    fun telexMatches() {
        val comparison = comparison()
        comparison.telex(Random(SEED), runs = 2_000, steps = 40)
        assertMatch(comparison, TELEX_OUTCOMES.map { "telex: $it" })
    }

    @Test
    fun randomRulesMatch() {
        val comparison = comparison()
        comparison.randomRules(Random(SEED), tables = 4_000, steps = 20)
        assertMatch(comparison, RULES_OUTCOMES.map { "rules: $it" })
    }

    @Test
    fun everyUnitMatches() {
        val comparison = comparison()
        comparison.everyUnit()
        assertMatch(comparison, listOf("hangul: keep", "kana: other keep", "telex: keep"))
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        const val COMPOSERS = "src/main/assets/ime/keyboard/org.florisboard.composers/extension.json"
        val HANGUL_OUTCOMES = listOf(
            "keep", "underscore after a syllable", "initial and vowel", "final added", "finals combine in a syllable",
            "final starts a syllable", "composed final split", "vowels combine in a syllable", "vowels combine",
            "finals combine",
        )
        val KANA_OUTCOMES = listOf("other keep") + listOf("dakuten", "handakuten").flatMap { mark ->
            listOf("keep", "nothing", "mark alone", "made", "second mark", "marks cancel").map { "$mark $it" }
        } + listOf("small nothing", "small made")
        val TELEX_OUTCOMES = listOf("keep", "delete 0", "delete 0, upper", "delete 1", "delete 1, upper", "delete 2", "delete 2, upper")
        val RULES_OUTCOMES = TELEX_OUTCOMES + listOf("delete -1", "delete -1, upper", "no table")
    }
}
