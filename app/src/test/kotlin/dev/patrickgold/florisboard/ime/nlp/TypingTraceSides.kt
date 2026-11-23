package dev.patrickgold.florisboard.ime.nlp

import com.jtn0123.audiowhisper.rust.KeptWordStore
import com.jtn0123.audiowhisper.rust.Learner
import com.jtn0123.audiowhisper.rust.NlpContext
import com.jtn0123.audiowhisper.rust.NlpOp
import com.jtn0123.audiowhisper.rust.SuggestRules
import com.jtn0123.audiowhisper.rust.TapPositions
import com.jtn0123.audiowhisper.rust.WireFlag
import com.jtn0123.audiowhisper.suggest.LatinDictionaryEngine
import com.jtn0123.audiowhisper.suggest.LearnedWords
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.editor.EditorRange

/** The settings and editor facts a trace event reads, as [NlpEnvironment] reads them on a phone. */
internal class TraceEnvironment {
    var inputType = 1
    var learn = true
    var incognito = false
    var suggestions = true
    var aim = true
    /** Whether the dictionary is open, so the provider suggests from it. */
    var loaded = true

    /** An [NlpContext] row for [content]. */
    fun fields(content: EditorContent, ready: Boolean = true): Array<String> = with(NlpContext) {
        val row = Array(SIZE) { "" }
        val flags = listOf(RAW_EDITOR to false, SUGGESTIONS to suggestions, EMOJI_SUGGESTIONS to true, LEARN to learn,
            INCOGNITO to incognito, SMARTBAR to true, SELECTION_VALID to content.localSelection.isValid, READY to ready,
            INLINE_SUGGESTIONS to false, BLOCK_OFFENSIVE to true)
        for ((field, on) in flags) row[field] = WireFlag.encode(on)
        row[INPUT_TYPE] = "$inputType"; row[PROVIDER] = "org.florisboard.nlp.providers.latin"; row[LANGUAGE] = "en"
        row[SELECTION_START] = "${content.selection.start}"; row[SELECTION_END] = "${content.selection.end}"
        row[COMPOSING_START] = "${content.composing.start}"; row[COMPOSING_END] = "${content.composing.end}"
        row[TEXT] = content.text; row[SUBTYPE] = "en-US/qwerty"; row[EMOJI_PREFIX] = ":"
        row[OFFSET] = "${content.offset}"; row[IME_OPTIONS] = "0"
        row
    }
}

/** What a side asked of Android in one event, in order, and the save timers it started. */
internal class TraceEffects(private val now: () -> Long) {
    val log = mutableListOf<String>()
    val timers = mutableListOf<Long>()
    var keptPending = false
    fun kept() { log += "kept"; keptPending = true }
    fun save(delayMs: Long) { log += "save $delayMs"; timers += now() + delayMs }
}

/** What a side has written to its private files: what a new process starts from. */
internal data class TraceDisk(var learned: String = "", var aim: String = "", var tally: String = "", var kept: String = "")

/**
 * The keyboard's side of the trace: the calls it makes into NLP, as NlpManager and KeyboardManager
 * make them, through the production [TypingEvents] and [NlpSessionState], sending each event as one
 * call, over native stores loaded from [disk]. Each event's send does what
 * LatinLanguageProvider.typingEvent does, lending the dictionary while it is open.
 */
internal class TypingSide(private val disk: TraceDisk, environment: TraceEnvironment, engine: LatinDictionaryEngine, now: () -> Long) {
    val effects = TraceEffects(now)
    private val learnedWords = LearnedWords(null).apply { store.merge(disk.learned) }
    private val positions = TapPositions(SuggestRules.tapTrailCapacity).apply { loadAim(disk.aim) }
    private val kept = KeptWordStore().apply { merge(disk.kept) }
    private val state = NlpSessionState().apply { call(NlpOp.TALLY_LOAD, disk.tally) }
    private val stores = TypingStores(Learner(learnedWords.store), learnedWords.store, positions, kept)
    private val typing = TypingEvents(
        send = { event, context, candidate, lends ->
            state.event(stores, if (lends && environment.loaded) engine.session else null, event, context, candidate)
        },
        context = { environment.fields(it) },
        now = now,
        saveAfter = { if (it >= 0) effects.save(it) },
        saveKept = effects::kept,
    )
    private val aim = { environment.aim }

    private fun call(op: NlpOp, vararg fields: String) = state.call(op, *fields)
    fun configure(fields: Array<String>) = state.configure(fields)
    fun show(values: List<SuggestionCandidate>) = state.directly(values)
    fun clear() = state.clear()
    val candidates get() = state.candidates.value

    fun clearUndo() = typing.disarm()
    fun tapTyped(text: String) = typing.typed(text)
    fun noteKey() = typing.key()
    fun autoCorrect(content: EditorContent, trigger: Int?, candidate: SuggestionCandidate?): Boolean {
        if (candidate == null) { typing.finished(content, trigger, aim()); return false }
        return typing.picked(candidate, content, commit = true, corrects = true)
    }
    fun commit(candidate: SuggestionCandidate, content: EditorContent) = typing.picked(candidate, content, commit = true, corrects = false)
    fun remove(candidate: SuggestionCandidate, content: EditorContent) = typing.picked(candidate, content, commit = false, corrects = false)
    fun enter(content: EditorContent) { typing.key(); typing.finished(content, null, aim()) }
    fun swipe(word: String, content: EditorContent) = typing.swiped(word, content)
    fun backspace(selection: EditorRange) = typing.backspace(selection.start, selection.end)
    fun undoCorrection(content: EditorContent) = typing.undoCorrection(content)
    fun correctionUndone() = typing.correctionUndone()
    fun tapDeleted() = typing.tapDeleted()
    fun undoKey() = typing.undo()
    fun field() = typing.field()

    fun pressed(codePoint: Int, dx: Float, dy: Float) = positions.pressed(codePoint, true, dx, dy)
    fun landscape(landscape: Boolean) = positions.landscape(landscape)

    /** getAutoCommitCandidate, given what the provider offered. */
    fun autoCommit(requested: Array<String>, trigger: Int?, phantom: Boolean, offered: SuggestionCandidate?,
                   current: Array<String>): SuggestionCandidate? {
        configure(requested)
        if (!WireFlag.decode(call(NlpOp.AUTO, "${trigger ?: -1}", WireFlag.encode(phantom))[0])) return null
        return offered?.takeIf { state.autoCandidate(it, requested, current) }
    }

    /** A save timer fired at [at]: the learned words, the tap aim and the counters are written. */
    fun fire(at: Long) {
        call(NlpOp.SAVE, "$at")
        disk.learned = learnedWords.store.savedText(); disk.aim = positions.aimText(); disk.tally = call(NlpOp.TALLY_TEXT)[0]
    }

    /** The kept words' write, launched at once, finished. */
    fun flushKept() { if (effects.keptPending) disk.kept = kept.savedText(); effects.keptPending = false }

    /** What this event left: the Smartbar's row, every store, and what is on disk. */
    fun observe(): List<Pair<String, String>> = listOf(
        "effects" to effects.log.joinToString(","),
        "candidates" to candidates.joinToString("|") { it.text },
        "smartbar" to "${state.wordsShown.value} ${state.expanded.value}",
        "learned" to learnedWords.store.savedText(),
        "kept" to kept.savedText(),
        "aim" to positions.aimText(),
        "counters" to call(NlpOp.TALLY_TEXT)[0],
        "disk" to disk.toString(),
    )
}
