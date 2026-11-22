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
 * One side of the trace: the calls the keyboard makes into NLP, as NlpManager and KeyboardManager
 * make them, over native stores loaded from [disk].
 */
internal abstract class TypingSide(protected val disk: TraceDisk, now: () -> Long) {
    val effects = TraceEffects(now)
    protected val learnedWords = LearnedWords(null).apply { store.merge(disk.learned) }
    protected val positions = TapPositions(SuggestRules.tapTrailCapacity).apply { loadAim(disk.aim) }
    protected val kept = KeptWordStore().apply { merge(disk.kept) }

    abstract fun call(op: NlpOp, vararg fields: String): Array<String>
    abstract fun configure(fields: Array<String>)
    abstract fun show(values: List<SuggestionCandidate>)
    abstract fun clear()
    abstract fun autoCandidate(candidate: SuggestionCandidate, requested: Array<String>, current: Array<String>): Boolean
    abstract val candidates: List<SuggestionCandidate>
    abstract val wordsShown: Boolean
    abstract val expanded: Int

    abstract fun clearUndo()
    abstract fun tapTyped(text: String)
    abstract fun noteKey()
    abstract fun autoCorrect(content: EditorContent, trigger: Int?, candidate: SuggestionCandidate?): Boolean
    abstract fun commit(candidate: SuggestionCandidate, content: EditorContent): Boolean
    abstract fun remove(candidate: SuggestionCandidate, content: EditorContent): Boolean
    abstract fun enter(content: EditorContent)
    abstract fun swipe(word: String, content: EditorContent)
    abstract fun backspace(selection: EditorRange)
    abstract fun undoCorrection(content: EditorContent): TypingEvents.Revert?
    abstract fun correctionUndone()
    abstract fun tapDeleted()
    abstract fun undoKey()
    abstract fun field()

    fun pressed(codePoint: Int, dx: Float, dy: Float) = positions.pressed(codePoint, true, dx, dy)
    fun landscape(landscape: Boolean) = positions.landscape(landscape)

    /** getAutoCommitCandidate, given what the provider offered. */
    fun autoCommit(requested: Array<String>, trigger: Int?, phantom: Boolean, offered: SuggestionCandidate?,
                   current: Array<String>): SuggestionCandidate? {
        configure(requested)
        if (!WireFlag.decode(call(NlpOp.AUTO, "${trigger ?: -1}", WireFlag.encode(phantom))[0])) return null
        return offered?.takeIf { autoCandidate(it, requested, current) }
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
        "smartbar" to "$wordsShown $expanded",
        "learned" to learnedWords.store.savedText(),
        "kept" to kept.savedText(),
        "aim" to positions.aimText(),
        "counters" to call(NlpOp.TALLY_TEXT)[0],
        "disk" to disk.toString(),
    )
}

/** Main's fan-out, frozen in [ReferenceTypingFanOut]. */
internal class ReferenceSide(disk: TraceDisk, environment: TraceEnvironment, engine: LatinDictionaryEngine, now: () -> Long) :
    TypingSide(disk, now) {
    private val state = ReferenceNlpState().apply { call(NlpOp.TALLY_LOAD, disk.tally) }
    private val fanOut = ReferenceTypingFanOut(state, learnedWords, positions, kept, environment, engine, now, effects)

    override fun call(op: NlpOp, vararg fields: String) = state.call(op, *fields)
    override fun configure(fields: Array<String>) = state.configure(fields)
    override fun show(values: List<SuggestionCandidate>) = state.directly(values)
    override fun clear() = state.clear()
    override fun autoCandidate(candidate: SuggestionCandidate, requested: Array<String>, current: Array<String>) =
        state.autoCandidate(candidate, requested, current)
    override val candidates get() = state.candidates
    override val wordsShown get() = state.wordsShown
    override val expanded get() = state.expanded

    override fun clearUndo() = fanOut.clearUndo()
    override fun tapTyped(text: String) = fanOut.tapTyped(text)
    override fun noteKey() = fanOut.noteKey()
    override fun autoCorrect(content: EditorContent, trigger: Int?, candidate: SuggestionCandidate?) =
        fanOut.autoCorrect(content, trigger, candidate)
    override fun commit(candidate: SuggestionCandidate, content: EditorContent) = fanOut.commitCandidate(candidate, content)
    override fun remove(candidate: SuggestionCandidate, content: EditorContent) = fanOut.canCommit(candidate, content)
    override fun enter(content: EditorContent) { fanOut.noteKey(); fanOut.learnTypedWord(content) }
    override fun swipe(word: String, content: EditorContent) = fanOut.learnSwipedWord(word, content)
    override fun backspace(selection: EditorRange) = fanOut.dropSwipedWord(selection)
    override fun undoCorrection(content: EditorContent) =
        fanOut.undoAutoCorrection(content)?.let { TypingEvents.Revert(it.length, it.text, it.phantomSpace) }
    override fun correctionUndone() = fanOut.noteCorrectionUndone()
    override fun tapDeleted() = fanOut.tapDeleted()
    override fun undoKey() = fanOut.noteUndo()
    override fun field() = fanOut.resetLearning()
}

/**
 * The branch: the production [TypingEvents] and [NlpSessionState], sending each event as one call;
 * [send] does what LatinLanguageProvider.typingEvent does, lending the dictionary while it is open.
 */
internal class BranchSide(disk: TraceDisk, environment: TraceEnvironment, engine: LatinDictionaryEngine, now: () -> Long) :
    TypingSide(disk, now) {
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

    override fun call(op: NlpOp, vararg fields: String) = state.call(op, *fields)
    override fun configure(fields: Array<String>) = state.configure(fields)
    override fun show(values: List<SuggestionCandidate>) = state.directly(values)
    override fun clear() = state.clear()
    override fun autoCandidate(candidate: SuggestionCandidate, requested: Array<String>, current: Array<String>) =
        state.autoCandidate(candidate, requested, current)
    override val candidates get() = state.candidates.value
    override val wordsShown get() = state.wordsShown.value
    override val expanded get() = state.expanded.value

    override fun clearUndo() = typing.disarm()
    override fun tapTyped(text: String) = typing.typed(text)
    override fun noteKey() = typing.key()
    override fun autoCorrect(content: EditorContent, trigger: Int?, candidate: SuggestionCandidate?): Boolean {
        if (candidate == null) { typing.finished(content, trigger, aim()); return false }
        return typing.picked(candidate, content, commit = true, corrects = true)
    }
    override fun commit(candidate: SuggestionCandidate, content: EditorContent) = typing.picked(candidate, content, commit = true, corrects = false)
    override fun remove(candidate: SuggestionCandidate, content: EditorContent) = typing.picked(candidate, content, commit = false, corrects = false)
    override fun enter(content: EditorContent) { typing.key(); typing.finished(content, null, aim()) }
    override fun swipe(word: String, content: EditorContent) = typing.swiped(word, content)
    override fun backspace(selection: EditorRange) = typing.backspace(selection.start, selection.end)
    override fun undoCorrection(content: EditorContent) = typing.undoCorrection(content)
    override fun correctionUndone() = typing.correctionUndone()
    override fun tapDeleted() = typing.tapDeleted()
    override fun undoKey() = typing.undo()
    override fun field() = typing.field()
}
