package dev.patrickgold.florisboard.ime.nlp

import com.jtn0123.audiowhisper.rust.KeptWordStore
import com.jtn0123.audiowhisper.rust.LearnedWordStore
import com.jtn0123.audiowhisper.rust.Learner
import com.jtn0123.audiowhisper.rust.NlpEvent
import com.jtn0123.audiowhisper.rust.NlpEventKind
import com.jtn0123.audiowhisper.rust.NlpEventPlan
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.ime.editor.EditorContent
import com.jtn0123.audiowhisper.rust.TapPositions
import org.florisboard.lib.kotlin.safeSubstring

/** AudioWhisper: the native stores a typing event changes besides the session (see [NlpState.event]). */
internal class TypingStores(val learner: Learner, val store: LearnedWordStore, val taps: TapPositions, val kept: KeptWordStore)

/**
 * AudioWhisper: each typing event goes to the NLP session as one call, which answers with one
 * plan (see lib/rust/crates/nlp/src/typing.rs). The session counts, learns, teaches the tap trail
 * and arms or takes the auto-correction undo itself; this carries the event in and does what the
 * plan asks of Android: start the save timers and write the kept words. Never logs a word.
 *
 * [send] takes an [NlpEvent] row, the [NlpContext] row after it (empty for kinds without one),
 * the candidate tapped, and whether to lend the dictionary; it returns the [NlpEventPlan] row.
 */
internal class TypingEvents(
    private val send: (event: Array<String>, context: Array<String>, candidate: SuggestionCandidate?, lends: Boolean) -> Array<String>,
    private val context: (EditorContent) -> Array<String>,
    private val now: () -> Long,
    private val saveAfter: (delayMs: Long) -> Unit,
    private val saveKept: () -> Unit,
) {
    /** Replace the [length] characters left of the cursor with [text], leaving a phantom space if [phantomSpace]. */
    data class Revert(val length: Int, val text: String, val phantomSpace: Boolean)

    /** A key that types went down: a space or Enter. */
    fun key() { event(NlpEventKind.KEY) }

    /** A tapped character, [text], went in. */
    fun typed(text: String) { event(NlpEventKind.TYPED, fields = arrayOf(NlpEvent.WORD to text)) }

    /** The tap trail's last character was deleted. */
    fun tapDeleted() { event(NlpEventKind.TAP_DELETED) }

    /** A backspace, with the selection at [start]..[end] before it deleted anything. */
    fun backspace(start: Int, end: Int) {
        event(NlpEventKind.BACKSPACE, fields = arrayOf(NlpEvent.SELECTION_START to "$start", NlpEvent.SELECTION_END to "$end"))
    }

    /** The undo key. */
    fun undo() { event(NlpEventKind.UNDO) }

    /** The word being typed in [content] was finished by [trigger] (null for a space or Enter) and stays as typed. */
    fun finished(content: EditorContent, trigger: Int?, aim: Boolean) {
        event(NlpEventKind.FINISHED, content, fields = word(content, content.composingText, trigger) +
            (NlpEvent.AIM to WireFlag.encode(aim)))
    }

    /**
     * [candidate] was tapped in [content]: it goes in if [commit], as the auto-correction of the word
     * being typed if [corrects] too. Returns whether it may go in.
     */
    fun picked(candidate: SuggestionCandidate, content: EditorContent, commit: Boolean, corrects: Boolean): Boolean {
        val fields = word(content, candidate.text.toString(), null) + arrayOf(NlpEvent.TYPED to content.composingText,
            NlpEvent.WORD_CANDIDATE to WireFlag.encode(candidate is WordSuggestionCandidate),
            NlpEvent.COMMIT to WireFlag.encode(commit), NlpEvent.CORRECTS to WireFlag.encode(corrects))
        return WireFlag.decode(event(NlpEventKind.PICKED, content, candidate, fields)[NlpEventPlan.ACCEPTED])
    }

    /** [word] was swiped in after what [content] holds. */
    fun swiped(word: String, content: EditorContent) { event(NlpEventKind.SWIPED, content, fields = word(content, word, null)) }

    /** What was just done can no longer undo an auto-correction. */
    fun disarm() { event(NlpEventKind.DISARM) }

    /** A backspace in [content] straight after an auto-correction: how to undo it, or null for nothing to undo. */
    fun undoCorrection(content: EditorContent): Revert? {
        val plan = event(NlpEventKind.UNDO_CORRECTION, content, fields = arrayOf(
            NlpEvent.CURSOR_MODE to WireFlag.encode(content.selection.isCursorMode),
            NlpEvent.SELECTION_START to "${content.selection.start}", NlpEvent.TEXT_BEFORE_SELECTION to content.textBeforeSelection))
        val length = plan[NlpEventPlan.REVERT].toInt()
        return if (length < 0) null else Revert(length, plan[NlpEventPlan.REVERT_TEXT], WireFlag.decode(plan[NlpEventPlan.PHANTOM]))
    }

    /** The undo [undoCorrection] asked for put back what was typed. */
    fun correctionUndone() { event(NlpEventKind.CORRECTION_UNDONE) }

    /** Input moved to another field. */
    fun field() { event(NlpEventKind.FIELD) }

    /** The fields of [word], finished by [trigger] in [content]. */
    private fun word(content: EditorContent, word: String, trigger: Int?): Array<Pair<Int, String>> = arrayOf(
        NlpEvent.TRIGGER to "${trigger ?: -1}", NlpEvent.WORD to word,
        NlpEvent.BEFORE_COMPOSING to content.text.safeSubstring(0, content.localComposing.start),
        NlpEvent.COMPOSING_START to "${content.composing.start}",
        NlpEvent.COMPOSING_VALID to WireFlag.encode(content.localComposing.isValid),
        NlpEvent.TEXT_BEFORE_SELECTION to content.textBeforeSelection)

    /** Sends event [kind], with [content]'s context if given, and carries out its plan. */
    private fun event(kind: Int, content: EditorContent? = null, candidate: SuggestionCandidate? = null,
                      fields: Array<Pair<Int, String>> = emptyArray()): Array<String> {
        val row = Array(NlpEvent.SIZE) { "" }
        row[NlpEvent.KIND] = "$kind"
        row[NlpEvent.NOW] = "${now()}"
        for ((field, value) in fields) row[field] = value
        val plan = send(row, content?.let(context) ?: emptyArray(), candidate, content != null)
        if (WireFlag.decode(plan[NlpEventPlan.KEPT])) saveKept()
        saveAfter(plan[NlpEventPlan.SAVE].toLong())
        saveAfter(plan[NlpEventPlan.SAVE_AGAIN].toLong())
        return plan
    }
}
