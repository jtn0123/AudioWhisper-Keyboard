package dev.patrickgold.florisboard.ime.nlp

import com.jtn0123.audiowhisper.suggest.LatinDictionaryEngine
import com.jtn0123.audiowhisper.suggest.TestDictionaries
import dev.patrickgold.florisboard.ime.editor.EditorContent
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlin.test.assertNotSame
import kotlin.test.assertSame
import kotlin.test.assertTrue

/**
 * A tap on the Smartbar's row as the keyboard shows it. The row is a StateFlow, which keeps the
 * objects it holds when a list equal to them is published, so the keyboard can show, and the user
 * tap, objects the session did not get from the latest list. Such a tap must go in as a tap on
 * the same place in a fresh row does (docs/phase6-kotlin-references.md, the stale tap).
 */
class StripTapTest {
    private fun engine(): LatinDictionaryEngine = TestDictionaries.shipped.let {
        requireNotNull(LatinDictionaryEngine.open(it.path, 0, it.length(), "en_US"))
    }

    private fun row(vararg words: String): List<SuggestionCandidate> = words.map { WordSuggestionCandidate(it) }

    private val empty = TraceEditor().content()

    /** A word being typed, so a pick counts the place it was made at. */
    private val typing = TraceEditor().apply { letter("t"); letter("h") }.content()

    /** A fresh side, configured for a text field holding [content], with [rows] shown in turn. */
    private fun side(engine: LatinDictionaryEngine, content: EditorContent, vararg rows: List<SuggestionCandidate>): TypingSide {
        val environment = TraceEnvironment()
        return TypingSide(TraceDisk(), environment, engine) { 1_000_000L }.apply {
            configure(environment.fields(content))
            rows.forEach(::show)
        }
    }

    private fun TypingSide.counters() = observe().toMap().getValue("counters")

    @Test fun aTapOnARowKeptFromAnEqualListGoesIn() = engine().use { engine ->
        val side = side(engine, empty, row("the", "and", "you"))
        val kept = side.candidates
        side.show(row("the", "and", "you"))
        assertSame(kept, side.candidates, "an equal list leaves the row's objects as they were")
        assertTrue(side.commit(side.candidates[1], empty))
    }

    @Test fun aTapOnAKeptRowCountsThePlaceTapped() = engine().use { engine ->
        // The same word at the second place of a fresh row and of a kept one, and at the first place
        // of another fresh row. Every side is shown two rows.
        val fresh = side(engine, typing, row("you", "that", "this"), row("the", "and", "you"))
        val kept = side(engine, typing, row("the", "and", "you"), row("the", "and", "you"))
        val first = side(engine, typing, row("you", "that", "this"), row("and", "the", "you"))
        assertTrue(fresh.commit(fresh.candidates[1], typing))
        assertTrue(kept.commit(kept.candidates[1], typing))
        assertTrue(first.commit(first.candidates[0], typing))
        assertEquals(fresh.counters(), kept.counters())
        assertNotEquals(first.counters(), kept.counters(), "the counters tell the first place from the second")
    }

    @Test fun aRemovalFromAKeptRowIsAsked() = engine().use { engine ->
        val side = side(engine, empty, row("the", "and"), row("the", "and"))
        assertTrue(side.remove(side.candidates[0], empty))
    }

    @Test fun aTapOnAWordNoLongerShownIsRefused() = engine().use { engine ->
        val side = side(engine, empty, row("the", "and"))
        val gone = side.candidates[0]
        side.show(row("you", "that"))
        assertNotSame(gone, side.candidates[0])
        assertFalse(side.commit(gone, empty))
    }

    @Test fun aTapOnAReplacedRowIsRefusedThoughItsWordIsShown() = engine().use { engine ->
        // A different list is new objects; the tapped one is from a row the keyboard no longer
        // shows, so it is refused, as on main, though the new row has the same word elsewhere.
        val side = side(engine, empty, row("the", "and"))
        val replaced = side.candidates[0]
        side.show(row("and", "the"))
        assertEquals(replaced, side.candidates[1])
        assertNotSame(replaced, side.candidates[1])
        assertFalse(side.commit(replaced, empty))
    }
}
