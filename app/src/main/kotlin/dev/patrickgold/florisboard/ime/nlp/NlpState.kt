package dev.patrickgold.florisboard.ime.nlp

import android.content.Context
import com.jtn0123.audiowhisper.rust.NlpSession
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
    fun fields(subtype: Subtype, content: EditorContent = editor.activeContent, ready: Boolean = true): Array<String> = arrayOf(
        editor.activeInfo.inputAttributes.raw.toString(), editor.activeInfo.isRawInputEditor.bit(),
        subtype.nlpProviders.suggestion, subtype.primaryLocale.language,
        prefs.suggestion.enabled.get().bit(), prefs.emoji.suggestionEnabled.get().bit(),
        prefs.suggestion.learnFromTyping.get().bit(), keyboard.activeState.isIncognitoMode.bit(),
        prefs.smartbar.enabled.get().bit(), content.localSelection.isValid.bit(),
        content.selection.start.toString(), content.selection.end.toString(),
        content.composing.start.toString(), content.composing.end.toString(), content.text,
        subtype.toString(), prefs.emoji.suggestionType.get().prefix, ready.bit(),
        prefs.suggestion.api30InlineSuggestionsEnabled.get().bit(), prefs.suggestion.blockPossiblyOffensive.get().bit(),
        content.offset.toString(), editor.activeInfo.imeOptions.raw.toString(),
    )
    fun policy(subtype: Subtype, content: EditorContent = editor.activeContent, ready: Boolean = true): Array<String> =
        NlpState.call("policy", *fields(subtype, content, ready))
}
internal fun Boolean.bit() = if (this) "1" else "0"

/** Retains platform objects by opaque ids. Rust alone owns their ordering and visibility. */
internal object NlpState {
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
    val expanded = MutableStateFlow(-1)
    @Synchronized fun call(op: String, vararg fields: String): Array<String> = native.call(op, *fields)
    @Synchronized fun configure(fields: Array<String>) { publish(native.call("context", *fields)) }
    @Synchronized fun begin(now: Long, current: Array<String>): Array<String> {
        val result = native.call("beginFor", *current, now.toString())
        publish(result.drop(4).toTypedArray())
        return result.take(4).toTypedArray()
    }
    @Synchronized fun field() { native.call("field"); publish() }
    @Synchronized fun clear() { native.call("clear"); words = emptyMap(); clips = emptyMap(); publish() }
    private fun pack(values: List<SuggestionCandidate>): Pair<Map<String, SuggestionCandidate>, Array<String>> {
        val objects = values.associateBy { (++nextId).toString() }
        val fields = objects.flatMap { (id, v) -> listOf(id, v.text.toString(), when (v) {
            is EmojiSuggestionCandidate -> "1"
            is ClipboardSuggestionCandidate -> "2"
            else -> "0"
        }, v.isEligibleForAutoCommit.bit()) }.toTypedArray()
        return objects to fields
    }
    @Synchronized fun complete(generation: String, values: List<SuggestionCandidate>, current: Array<String>) {
        val (objects, fields) = pack(values)
        val result = native.call("completeFor", *current, generation, *fields)
        if (result[0] == "1") words = objects
        publish(result.drop(1).toTypedArray())
    }
    @Synchronized fun autoCandidate(candidate: SuggestionCandidate, requested: Array<String>, current: Array<String>): Boolean {
        val (objects, fields) = pack(listOf(candidate))
        val accepted = native.call("autoCandidate", *requested, *current, *fields)[0] == "1"
        if (accepted) automatic = objects
        publish()
        return accepted
    }
    @Synchronized fun directly(values: List<SuggestionCandidate>) {
        val (objects, fields) = pack(values)
        val result = native.call("direct", *fields); words = objects; clips = emptyMap(); publish(result)
    }
    @Synchronized fun beginClipboard(current: Array<String>): String {
        val result = native.call("clipboardBeginFor", *current)
        publish(result.drop(1).toTypedArray())
        return result[0]
    }
    @Synchronized fun clipboard(generation: String, values: List<SuggestionCandidate>, requested: Array<String>, current: Array<String>) {
        val (objects, fields) = pack(values)
        val result = native.call("clipboardFor", *requested, *current, generation, *fields); clips = clips + objects; publish(result)
    }
    @Synchronized fun inlineBegin(count: Int): Array<String> = native.call("inlineBegin", count.toString()).also { publish() }
    @Synchronized fun inlineComplete(generation: String, values: List<NlpInlineAutofillSuggestion>) {
        val objects = values.associateBy { (++nextId).toString() }
        val fields = objects.flatMap { (id, v) -> listOf(id, v.info.isPinned.bit()) }.toTypedArray()
        if (native.call("inlineComplete", generation, *fields)[0] == "1") views = objects
        publish()
    }
    @Synchronized fun pick(candidate: SuggestionCandidate, current: Array<String>): Boolean {
        native.call("context", *current)
        val id = (words + clips + automatic).entries.firstOrNull { it.value === candidate }?.key
        if (id == null) { publish(); return false }
        val result = native.call("pick", id)[0] == "1"
        publish()
        return result
    }
    private fun publish(snapshot: Array<String> = native.call("snapshot")) {
        var at = 3
        val count = snapshot[at++].toInt()
        candidatesMutable.value = List(count) { (words + clips).getValue(snapshot[at++]) }
        val inlineCount = snapshot[at++].toInt()
        inlineMutable.value = List(inlineCount) { views.getValue(snapshot[at++]) }
        expanded.value = snapshot[1].toInt()
        // Bound retained Java objects after invalidation; in-flight results retain their own lists.
        val visibleIds = snapshot.slice(4 until 4 + count).toSet()
        if (count == 0) words = emptyMap()
        clips = clips.filterKeys { it in visibleIds }
        if (inlineCount == 0) views = emptyMap()
    }
}
