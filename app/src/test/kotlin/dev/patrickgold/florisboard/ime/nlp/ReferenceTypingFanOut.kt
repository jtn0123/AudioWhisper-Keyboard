package dev.patrickgold.florisboard.ime.nlp

import com.jtn0123.audiowhisper.rust.KeptWordStore
import com.jtn0123.audiowhisper.rust.NlpAim
import com.jtn0123.audiowhisper.rust.NlpCandidate
import com.jtn0123.audiowhisper.rust.NlpCandidateKind
import com.jtn0123.audiowhisper.rust.NlpLearn
import com.jtn0123.audiowhisper.rust.NlpLearnKind
import com.jtn0123.audiowhisper.rust.NlpLearnPlan
import com.jtn0123.audiowhisper.rust.NlpOp
import com.jtn0123.audiowhisper.rust.NlpPolicy
import com.jtn0123.audiowhisper.rust.NlpSession
import com.jtn0123.audiowhisper.rust.NlpSnapshot
import com.jtn0123.audiowhisper.rust.NlpTally
import com.jtn0123.audiowhisper.rust.NlpTallyEvent
import com.jtn0123.audiowhisper.rust.TapPositions
import com.jtn0123.audiowhisper.rust.WireFlag
import com.jtn0123.audiowhisper.suggest.AutoCorrectionUndo
import com.jtn0123.audiowhisper.suggest.LatinDictionaryEngine
import com.jtn0123.audiowhisper.suggest.LearnedWords
import com.jtn0123.audiowhisper.suggest.WordLearner
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.editor.EditorRange
import org.florisboard.lib.kotlin.safeSubstring

/**
 * Main's NlpState at f1fdf3f34, frozen for [TypingTraceMatchTest], less the inline and clipboard
 * parts the trace never reaches: it fanned each typing event out one call at a time.
 */
internal class ReferenceNlpState {
    private val native = NlpSession()
    private var nextId = 0L
    private var automatic = emptyMap<String, SuggestionCandidate>()
    private var words = emptyMap<String, SuggestionCandidate>()
    var candidates = emptyList<SuggestionCandidate>(); private set
    var wordsShown = false; private set
    var expanded = -1; private set
    fun call(op: NlpOp, vararg fields: String): Array<String> = native.call(op, *fields)
    fun configure(fields: Array<String>) { publish(native.call(NlpOp.CONTEXT, *fields)) }
    fun clear() { native.call(NlpOp.CLEAR); words = emptyMap(); publish() }
    private fun pack(values: List<SuggestionCandidate>): Pair<Map<String, SuggestionCandidate>, Array<String>> {
        val objects = values.associateBy { (++nextId).toString() }
        val fields = objects.flatMap { (id, v) ->
            with(NlpCandidate) { row(ID to id, TEXT to v.text.toString(), KIND to "${NlpCandidateKind.WORD}", AUTO to WireFlag.encode(v.isEligibleForAutoCommit)).asList() }
        }.toTypedArray()
        return objects to fields
    }
    fun autoCandidate(candidate: SuggestionCandidate, requested: Array<String>, current: Array<String>): Boolean {
        val (objects, fields) = pack(listOf(candidate))
        val accepted = WireFlag.decode(native.call(NlpOp.AUTO_CANDIDATE, *requested, *current, *fields)[0])
        if (accepted) automatic = objects
        publish()
        return accepted
    }
    fun directly(values: List<SuggestionCandidate>) {
        val (objects, fields) = pack(values)
        val result = native.call(NlpOp.DIRECT, *fields); words = objects; publish(result)
    }
    fun pick(candidate: SuggestionCandidate, current: Array<String>): Boolean {
        native.call(NlpOp.CONTEXT, *current)
        val id = (words + automatic).entries.firstOrNull { it.value === candidate }?.key
        if (id == null) { publish(); return false }
        val result = WireFlag.decode(native.call(NlpOp.PICK, id)[0])
        publish()
        return result
    }
    fun field() { native.call(NlpOp.FIELD); publish() }
    private fun publish(snapshot: Array<String> = native.call(NlpOp.SNAPSHOT)) {
        val count = snapshot[NlpSnapshot.COUNT].toInt()
        candidates = List(count) { words.getValue(snapshot[NlpSnapshot.SIZE + it]) }
        expanded = snapshot[NlpSnapshot.EXPANDED].toInt()
        wordsShown = WireFlag.decode(native.call(NlpOp.WORDS)[0])
        if (count == 0) words = emptyMap()
    }
}

/**
 * Main's typing fan-out at f1fdf3f34, frozen: NlpManager's per-step calls, KeyboardManager's
 * auto-correction undo and commit, TapTrail's events and TypingCounters' tallies, with Android's
 * editor and settings read through [environment]. Records what it asks of Android in [effects].
 * Never edited to match the branch: it is what the branch is compared against.
 */
internal class ReferenceTypingFanOut(
    private val state: ReferenceNlpState,
    learnedWords: LearnedWords,
    private val positions: TapPositions,
    private val kept: KeptWordStore,
    private val environment: TraceEnvironment,
    engine: LatinDictionaryEngine,
    private val now: () -> Long,
    private val effects: TraceEffects,
) {
    private val learner = WordLearner(learnedWords) { engine.takeIf { environment.loaded } }
    private var autoCorrectionUndo: AutoCorrectionUndo? = null

    private fun configure(content: EditorContent) = state.configure(environment.fields(content))
    private fun policy(content: EditorContent, ready: Boolean = true) = state.call(NlpOp.POLICY, *environment.fields(content, ready))
    private fun tally(event: NlpTallyEvent, start: Int = 0, end: Int = 0) {
        state.call(NlpOp.TALLY, *with(NlpTally) { row(EVENT to event.code, START to "$start", END to "$end") })
    }

    /** KeyboardKeyEvents' CLEAR_UNDO. */
    fun clearUndo() { autoCorrectionUndo = null }

    /** TAP_TYPED. */
    fun tapTyped(text: String) { tally(NlpTallyEvent.KEY); positions.typed(text) }

    fun noteKey() = tally(NlpTallyEvent.KEY)

    /** KeyboardManager.commitCandidate's decision and learning; true if it went in. */
    fun commitCandidate(candidate: SuggestionCandidate, content: EditorContent): Boolean {
        if (!canCommit(candidate, content)) return false
        if (candidate is WordSuggestionCandidate) learnCandidate(candidate.text.toString(), content)
        return true
    }

    /** KeyboardEditing.autoCorrect, given what getAutoCommitCandidate answered. */
    fun autoCorrect(content: EditorContent, trigger: Int?, candidate: SuggestionCandidate?): Boolean {
        val typed = content.composingText
        val start = content.composing.start
        if (candidate == null) { learnTypedWord(content, trigger); return false }
        val committed = commitCandidate(candidate, content)
        val corrected = candidate.text.toString()
        if (typed.isNotEmpty() && start >= 0 && corrected != typed) autoCorrectionUndo = AutoCorrectionUndo(typed, corrected, start)
        return committed
    }

    /** KeyboardEditing.undoAutoCorrection, before the editor reverts. */
    fun undoAutoCorrection(content: EditorContent): AutoCorrectionUndo.Revert? {
        val undo = autoCorrectionUndo ?: return null
        autoCorrectionUndo = null
        if (!content.selection.isCursorMode) return null
        val revert = undo.revertAt(content.selection.start, content.textBeforeSelection) ?: return null
        keepAsTyped(undo.typed, content)
        learn(content) { it.undo(undo.start) }
        return revert
    }

    fun canCommit(candidate: SuggestionCandidate, content: EditorContent): Boolean =
        state.pick(candidate, environment.fields(content)).also {
            if (it && candidate is WordSuggestionCandidate) {
                positions.event(if (content.composingText.isNotEmpty()) TapPositions.PICKED_FOR_WORD else TapPositions.PICKED)
            }
            noteTypingChanged()
        }

    private fun keepAsTyped(word: String, content: EditorContent) {
        configure(content)
        val remember = WireFlag.decode(state.call(NlpOp.KEEP, word)[0])
        if (kept.add(word, remember)) effects.kept()
    }

    fun learnTypedWord(content: EditorContent, triggerCodePoint: Int? = null) {
        learningPlan(NlpLearnKind.TYPED, content.composingText, content, triggerCodePoint)
        val aim = state.call(NlpOp.AIM, *with(NlpAim) { row(SETTING to WireFlag.encode(environment.aim),
            TRIGGER to "${triggerCodePoint ?: -1}", WORD to content.composingText, COMPOSING_VALID to WireFlag.encode(content.localComposing.isValid)) })
        if (WireFlag.decode(aim[0])) positions.kept(content.composingText)
        noteTypingChanged()
    }

    private fun learnCandidate(text: String, content: EditorContent) {
        learner.dropSwiped()
        learningPlan(NlpLearnKind.PICKED, text, content)
    }

    fun learnSwipedWord(word: String, content: EditorContent) {
        learningPlan(NlpLearnKind.SWIPED, word, content)
        positions.event(TapPositions.SWIPED)
        noteTypingChanged()
    }

    private fun learningPlan(kind: Int, word: String, content: EditorContent, trigger: Int? = null) {
        configure(content)
        val plan = state.call(NlpOp.LEARN, *with(NlpLearn) { row(KIND to "$kind", TRIGGER to "${trigger ?: -1}", WORD to word,
            BEFORE_COMPOSING to content.text.safeSubstring(0, content.localComposing.start), COMPOSING_START to "${content.composing.start}",
            COMPOSING_VALID to WireFlag.encode(content.localComposing.isValid), TEXT_BEFORE_SELECTION to content.textBeforeSelection) })
        if (plan.isEmpty()) return
        learn(content) { learner -> with(NlpLearnPlan) { when (plan[KIND].toInt()) {
            NlpLearnKind.SWIPED -> learner.swiped(plan[WORD], plan[BEFORE])
            else -> learner.finished(plan[WORD], plan[BEFORE], plan[START].toInt())
        } } }
    }

    /** DELETE_DROP, with the selection as it stood. */
    fun dropSwipedWord(selection: EditorRange) {
        learner.dropSwiped()
        positions.event(TapPositions.BACKSPACE)
        tally(NlpTallyEvent.DELETED, selection.start, selection.end)
        noteTypingChanged()
    }

    /** DELETE_TAP. */
    fun tapDeleted() = positions.deleted()

    fun noteCorrectionUndone() { tally(NlpTallyEvent.UNDONE); noteTypingChanged() }

    fun noteUndo() { positions.event(TapPositions.UNDO); tally(NlpTallyEvent.UNDO); noteTypingChanged() }

    private fun noteTypingChanged() {
        if (positions.aimChanged() or WireFlag.decode(state.call(NlpOp.TALLY_CHANGED)[0])) saveLearnedWordsSoon()
    }

    fun resetLearning() { state.field(); learner.reset(); positions.clear() }

    private inline fun learn(content: EditorContent, change: (WordLearner) -> Boolean) {
        if (!WireFlag.decode(policy(content)[NlpPolicy.LEARN])) return
        if (!WireFlag.decode(policy(content, ready = environment.loaded)[NlpPolicy.LEARN])) return
        if (change(learner)) saveLearnedWordsSoon()
    }

    private fun saveLearnedWordsSoon() {
        val delayMs = state.call(NlpOp.SAVE, now().toString())[0].toLong()
        if (delayMs >= 0) effects.save(delayMs)
    }
}
