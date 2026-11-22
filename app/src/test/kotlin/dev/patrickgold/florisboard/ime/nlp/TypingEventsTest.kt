package dev.patrickgold.florisboard.ime.nlp

import com.jtn0123.audiowhisper.rust.KeptWordStore
import com.jtn0123.audiowhisper.rust.Learner
import com.jtn0123.audiowhisper.rust.NlpEvent
import com.jtn0123.audiowhisper.rust.NlpEventKind
import com.jtn0123.audiowhisper.rust.NlpEventPlan
import com.jtn0123.audiowhisper.rust.SuggestRules
import com.jtn0123.audiowhisper.rust.TapPositions
import com.jtn0123.audiowhisper.rust.WireFlag
import com.jtn0123.audiowhisper.suggest.LearnedWords
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.editor.EditorRange
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/** [TypingEvents] carries each event in as one row and does only what its plan asks of Android. */
class TypingEventsTest {
    private class Sent(val event: Array<String>, val context: Array<String>, val candidate: SuggestionCandidate?, val lends: Boolean)

    private val sent = mutableListOf<Sent>()
    private val asked = mutableListOf<String>()
    private var answer = plan()
    private val typing = TypingEvents(
        send = { event, context, candidate, lends -> sent += Sent(event, context, candidate, lends); answer },
        context = { arrayOf("context of ${it.text}") },
        now = { 4_242L },
        saveAfter = { asked += "save $it" },
        saveKept = { asked += "kept" },
    )

    private fun plan(save: Long = -1, again: Long = -1, kept: Boolean = false, accepted: Boolean = false,
                     revert: Int = -1, text: String = "", phantom: Boolean = false): Array<String> = Array(NlpEventPlan.SIZE) { "" }.also {
        it[NlpEventPlan.SAVE] = "$save"; it[NlpEventPlan.SAVE_AGAIN] = "$again"; it[NlpEventPlan.KEPT] = WireFlag.encode(kept)
        it[NlpEventPlan.ACCEPTED] = WireFlag.encode(accepted); it[NlpEventPlan.REVERT] = "$revert"
        it[NlpEventPlan.REVERT_TEXT] = text; it[NlpEventPlan.PHANTOM] = WireFlag.encode(phantom); it[NlpEventPlan.PUBLISH] = WireFlag.encode(false)
    }

    /** "teh" being typed after "say ", the cursor after it. */
    private val typingTeh = EditorContent("say teh", 0, EditorRange(7, 7), EditorRange(4, 7), EditorRange(4, 7))

    @Test fun anEventWithoutAContextSendsOnlyItsRowAndLendsNothing() {
        typing.typed("e")
        val one = sent.single()
        assertEquals("${NlpEventKind.TYPED}", one.event[NlpEvent.KIND])
        assertEquals("4242", one.event[NlpEvent.NOW])
        assertEquals("e", one.event[NlpEvent.WORD])
        assertEquals(NlpEvent.SIZE, one.event.size)
        assertEquals(0, one.context.size)
        assertFalse(one.lends)
        assertEquals(listOf("save -1", "save -1"), asked)
    }

    @Test fun aFinishedWordSendsItsWordsAndTheEditorAsItStands() {
        typing.finished(typingTeh, trigger = '.'.code, aim = true)
        val one = sent.single()
        assertEquals("${NlpEventKind.FINISHED}", one.event[NlpEvent.KIND])
        assertEquals("teh", one.event[NlpEvent.WORD])
        assertEquals("say ", one.event[NlpEvent.BEFORE_COMPOSING])
        assertEquals("4", one.event[NlpEvent.COMPOSING_START])
        assertEquals("${'.'.code}", one.event[NlpEvent.TRIGGER])
        assertEquals(WireFlag.encode(true), one.event[NlpEvent.AIM])
        assertEquals("say teh", one.event[NlpEvent.TEXT_BEFORE_SELECTION])
        assertContentEquals(arrayOf("context of say teh"), one.context)
        assertTrue(one.lends)
    }

    @Test fun thePlanSavesAndWritesTheKeptWordsInOrder() {
        answer = plan(save = 0, again = 5_000, kept = true)
        typing.correctionUndone()
        assertEquals(listOf("kept", "save 0", "save 5000"), asked)
    }

    @Test fun anAutoCorrectionSendsWhatWasTypedAndSaysWhetherItWentIn() {
        val candidate = WordSuggestionCandidate("the")
        answer = plan(accepted = true)
        assertTrue(typing.picked(candidate, typingTeh, commit = true, corrects = true))
        val one = sent.single()
        assertEquals("the", one.event[NlpEvent.WORD])
        assertEquals("teh", one.event[NlpEvent.TYPED])
        assertEquals(WireFlag.encode(true), one.event[NlpEvent.WORD_CANDIDATE])
        assertEquals(WireFlag.encode(true), one.event[NlpEvent.CORRECTS])
        assertEquals(WireFlag.encode(true), one.event[NlpEvent.COMMIT])
        assertEquals(candidate, one.candidate)
        answer = plan(accepted = false)
        assertFalse(typing.picked(candidate, typingTeh, commit = false, corrects = false))
        assertEquals(WireFlag.encode(false), sent.last().event[NlpEvent.COMMIT])
    }

    @Test fun anUndoIsTheRevertThePlanNamesOrNothing() {
        val after = EditorContent("say the ", 0, EditorRange(8, 8), EditorRange.Unspecified, EditorRange.Unspecified)
        answer = plan(revert = 4, text = "teh", phantom = true)
        assertEquals(TypingEvents.Revert(4, "teh", true), typing.undoCorrection(after))
        val one = sent.single()
        assertEquals(WireFlag.encode(true), one.event[NlpEvent.CURSOR_MODE])
        assertEquals("8", one.event[NlpEvent.SELECTION_START])
        assertEquals("say the ", one.event[NlpEvent.TEXT_BEFORE_SELECTION])
        answer = plan()
        assertNull(typing.undoCorrection(after))
    }

    @Test fun theSessionLetsInOnlyACandidateItShowedAndPublishesThePlansRow() {
        val state = NlpSessionState()
        val learned = LearnedWords(null)
        val stores = TypingStores(Learner(learned.store), learned.store, TapPositions(SuggestRules.tapTrailCapacity), KeptWordStore())
        val environment = TraceEnvironment()
        val events = TypingEvents({ event, context, candidate, _ -> state.event(stores, null, event, context, candidate) },
            { environment.fields(it) }, { 1_000L }, {}, {})
        val shown = WordSuggestionCandidate("the")
        state.configure(environment.fields(typingTeh))
        state.directly(listOf(shown))
        assertEquals(listOf<SuggestionCandidate>(shown), state.candidates.value)
        assertFalse(events.picked(WordSuggestionCandidate("the"), typingTeh, commit = false, corrects = false))
        assertTrue(events.picked(shown, typingTeh, commit = false, corrects = false))
        // The word finished in an editor that moved on: the row the plan carries has nothing to show.
        events.finished(EditorContent("say teh ", 0, EditorRange(8, 8), EditorRange.Unspecified, EditorRange.Unspecified), null, aim = false)
        assertEquals(emptyList(), state.candidates.value)
    }
}
