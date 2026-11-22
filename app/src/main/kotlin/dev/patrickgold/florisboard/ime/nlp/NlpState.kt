package dev.patrickgold.florisboard.ime.nlp

import android.content.Context
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.keyboardManager
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

/** Android snapshots only. Rust interprets raw flags and all eligibility/settings combinations. */
internal class NlpEnvironment(context: Context) {
    private val prefs by FlorisPreferenceStore
    private val editor by context.editorInstance()
    private val keyboard by context.keyboardManager()
    /** An [NlpContext] row. */
    fun fields(subtype: Subtype, content: EditorContent = editor.activeContent, ready: Boolean = true): Array<String> = with(NlpContext) {
        val info = editor.activeInfo
        val flags = listOf(RAW_EDITOR to info.isRawInputEditor, SUGGESTIONS to prefs.suggestion.enabled.get(),
            EMOJI_SUGGESTIONS to prefs.emoji.suggestionEnabled.get(), LEARN to prefs.suggestion.learnFromTyping.get(),
            INCOGNITO to keyboard.activeState.isIncognitoMode, SMARTBAR to prefs.smartbar.enabled.get(),
            SELECTION_VALID to content.localSelection.isValid, READY to ready,
            INLINE_SUGGESTIONS to prefs.suggestion.api30InlineSuggestionsEnabled.get(),
            BLOCK_OFFENSIVE to prefs.suggestion.blockPossiblyOffensive.get()).map { (field, on) -> field to WireFlag.encode(on) }
        row(INPUT_TYPE to "${info.inputAttributes.raw}", PROVIDER to subtype.nlpProviders.suggestion,
            LANGUAGE to subtype.primaryLocale.language, SELECTION_START to "${content.selection.start}",
            SELECTION_END to "${content.selection.end}", COMPOSING_START to "${content.composing.start}",
            COMPOSING_END to "${content.composing.end}", TEXT to content.text, SUBTYPE to "$subtype",
            EMOJI_PREFIX to prefs.emoji.suggestionType.get().prefix, OFFSET to "${content.offset}",
            IME_OPTIONS to "${info.imeOptions.raw}", *flags.toTypedArray())
    }
    /** An [NlpPolicy] row. */
    fun policy(subtype: Subtype, content: EditorContent = editor.activeContent, ready: Boolean = true): Array<String> =
        NlpState.call(NlpOp.POLICY, *fields(subtype, content, ready))
}

/** The keyboard's one NLP session. */
internal object NlpState : NlpSessionState()

/**
 * Retains platform objects by opaque ids. Rust alone owns their ordering and visibility. A class
 * so a host test can start a session afresh, as a new process does.
 */
internal open class NlpSessionState {
    private val native = NlpSession()
    private var nextId = 0L
    private var automatic = emptyMap<String, SuggestionCandidate>()
    private var words = emptyMap<String, SuggestionCandidate>()
    private var clips = emptyMap<String, SuggestionCandidate>()
    private var views = emptyMap<String, NlpInlineAutofillSuggestion>()
    private val candidatesMutable = MutableStateFlow<List<SuggestionCandidate>>(emptyList())
    val candidates = candidatesMutable.asStateFlow()
    private val inlineMutable = MutableStateFlow<List<NlpInlineAutofillSuggestion>>(emptyList())
    val inline = inlineMutable.asStateFlow()
    private val wordsShownMutable = MutableStateFlow(false)
    /** Rust's one fact for the Smartbar: words show, or typing emptied them and the refill is due. */
    val wordsShown = wordsShownMutable.asStateFlow()
    val expanded = MutableStateFlow(-1)
    @Synchronized fun call(op: NlpOp, vararg fields: String): Array<String> = native.call(op, *fields)
    @Synchronized fun configure(fields: Array<String>) { publish(native.call(NlpOp.CONTEXT, *fields)) }
    /** An [NlpBegin] row. */
    @Synchronized fun begin(now: Long, current: Array<String>): Array<String> {
        val result = native.call(NlpOp.BEGIN_FOR, *current, now.toString())
        publish(result.drop(NlpBegin.SIZE).toTypedArray())
        return result.take(NlpBegin.SIZE).toTypedArray()
    }
    @Synchronized fun clear() { native.call(NlpOp.CLEAR); words = emptyMap(); clips = emptyMap(); publish() }
    /** An [NlpCandidate] row for each of [values]. */
    private fun pack(values: List<SuggestionCandidate>): Pair<Map<String, SuggestionCandidate>, Array<String>> {
        val objects = values.associateBy { (++nextId).toString() }
        val fields = objects.flatMap { (id, v) ->
            val kind = when (v) { is EmojiSuggestionCandidate -> NlpCandidateKind.EMOJI; is ClipboardSuggestionCandidate -> NlpCandidateKind.CLIPBOARD; else -> NlpCandidateKind.WORD }
            with(NlpCandidate) { row(ID to id, TEXT to v.text.toString(), KIND to "$kind", AUTO to WireFlag.encode(v.isEligibleForAutoCommit)).asList() }
        }.toTypedArray()
        return objects to fields
    }
    @Synchronized fun complete(generation: String, values: List<SuggestionCandidate>, current: Array<String>) {
        val (objects, fields) = pack(values)
        val result = native.call(NlpOp.COMPLETE_FOR, *current, generation, *fields)
        if (WireFlag.decode(result[0])) words = objects
        publish(result.drop(1).toTypedArray())
    }
    @Synchronized fun autoCandidate(candidate: SuggestionCandidate, requested: Array<String>, current: Array<String>): Boolean {
        val (objects, fields) = pack(listOf(candidate))
        val accepted = WireFlag.decode(native.call(NlpOp.AUTO_CANDIDATE, *requested, *current, *fields)[0])
        if (accepted) automatic = objects
        publish()
        return accepted
    }
    @Synchronized fun directly(values: List<SuggestionCandidate>) {
        val (objects, fields) = pack(values)
        val result = native.call(NlpOp.DIRECT, *fields); words = objects; clips = emptyMap(); publish(result)
    }
    @Synchronized fun beginClipboard(current: Array<String>): String {
        val result = native.call(NlpOp.CLIPBOARD_BEGIN_FOR, *current)
        publish(result.drop(1).toTypedArray())
        return result[0]
    }
    @Synchronized fun clipboard(generation: String, values: List<SuggestionCandidate>, requested: Array<String>, current: Array<String>) {
        val (objects, fields) = pack(values)
        val result = native.call(NlpOp.CLIPBOARD_FOR, *requested, *current, generation, *fields); clips = clips + objects; publish(result)
    }
    /** An [NlpInlineBegin] row. */
    @Synchronized fun inlineBegin(count: Int): Array<String> = native.call(NlpOp.INLINE_BEGIN, count.toString()).also { publish() }
    @Synchronized fun inlineComplete(generation: String, values: List<NlpInlineAutofillSuggestion>) {
        val objects = values.associateBy { (++nextId).toString() }
        val fields = objects.flatMap { (id, v) -> with(NlpInlineView) { row(ID to id, PINNED to WireFlag.encode(v.info.isPinned)).asList() } }
        if (WireFlag.decode(native.call(NlpOp.INLINE_COMPLETE, generation, *fields.toTypedArray())[0])) views = objects
        publish()
    }
    /**
     * The [NlpEventPlan] row for typing event [event], [candidate]'s id put in, with [context]
     * after it (see [TypingEvents]); the session changes the other stores itself, asking
     * [dictionary] if lent. Publishes the snapshot the plan carries.
     */
    @Synchronized fun event(stores: TypingStores, dictionary: LatinSession?, event: Array<String>, context: Array<String>,
                            candidate: SuggestionCandidate?): Array<String> {
        if (candidate != null) event[NlpEvent.CANDIDATE] = (words + clips + automatic).entries.firstOrNull { it.value === candidate }?.key.orEmpty()
        val plan = native.event(stores.learner, stores.store, stores.taps, stores.kept, dictionary, *event, *context)
        if (WireFlag.decode(plan[NlpEventPlan.PUBLISH])) publish(plan.copyOfRange(NlpEventPlan.SIZE, plan.size))
        return plan
    }
    /** An [NlpSnapshot] row, then the ids it counts. */
    private fun publish(snapshot: Array<String> = native.call(NlpOp.SNAPSHOT)) {
        val count = snapshot[NlpSnapshot.COUNT].toInt()
        var at = NlpSnapshot.SIZE
        candidatesMutable.value = List(count) { (words + clips).getValue(snapshot[at++]) }
        val inlineCount = snapshot[at++].toInt()
        inlineMutable.value = List(inlineCount) { views.getValue(snapshot[at++]) }
        expanded.value = snapshot[NlpSnapshot.EXPANDED].toInt()
        wordsShownMutable.value = WireFlag.decode(native.call(NlpOp.WORDS)[0])
        // Bound retained Java objects after invalidation; in-flight results retain their own lists.
        val visibleIds = snapshot.slice(NlpSnapshot.SIZE until NlpSnapshot.SIZE + count).toSet()
        if (count == 0) words = emptyMap()
        clips = clips.filterKeys { it in visibleIds }
        if (inlineCount == 0) views = emptyMap()
    }
}
