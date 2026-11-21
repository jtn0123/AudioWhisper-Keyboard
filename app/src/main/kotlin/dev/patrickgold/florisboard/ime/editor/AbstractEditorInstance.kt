/* Copyright (C) 2022-2025 The FlorisBoard Contributors; Apache-2.0. */
package dev.patrickgold.florisboard.ime.editor

import android.content.Context
import android.view.inputmethod.InputConnection
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.ime.nlp.BreakIteratorGroup
import dev.patrickgold.florisboard.ime.text.composing.Composer
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.runBlocking

enum class OperationUnit { CHARACTERS, WORDS }
enum class OperationScope { BEFORE_CURSOR, AFTER_CURSOR }

/** Android lifecycle and observable DTOs. Rust owns the cache, reconciliation and operation order. */
abstract class AbstractEditorInstance(context: Context) {
    private val keyboardManager by context.keyboardManager()
    private val subtypeManager by context.subtypeManager()
    private val nlpManager by context.nlpManager()
    protected val scope = MainScope()
    protected val breakIterators = BreakIteratorGroup()
    protected val editorSession = EditorSession()
    private val executor = EditorPlanExecutor(
        breakIterators,
        { subtypeManager.activeSubtype.primaryLocale },
        { determineComposer(subtypeManager.activeSubtype.composer) },
        { before, last -> nlpManager.determineLocalComposing(before, breakIterators, last) },
    )
    private val _activeInfoFlow = MutableStateFlow(FlorisEditorInfo.Unspecified)
    val activeInfoFlow = _activeInfoFlow.asStateFlow()
    var activeInfo: FlorisEditorInfo
        get() = _activeInfoFlow.value
        private set(value) { _activeInfoFlow.value = value }
    private val _activeCursorCapsModeFlow = MutableStateFlow(InputAttributes.CapsMode.NONE)
    val activeCursorCapsModeFlow = _activeCursorCapsModeFlow.asStateFlow()
    val activeCursorCapsMode get() = _activeCursorCapsModeFlow.value
    private val _activeContentFlow = MutableStateFlow(EditorContent.Unspecified)
    val activeContentFlow = _activeContentFlow.asStateFlow()
    val activeContent: EditorContent get() = snapshot().content
    val lastCommitPosition get() = LastCommitPosition(snapshot().last)
    fun expectedContent(): EditorContent? = snapshot().let { if (it.expected) it.content else null }

    protected open val isSpacePending: Boolean get() = false
    abstract fun determineComposingEnabled(): Boolean
    abstract fun determineComposer(composerName: ExtensionComponentName): Composer
    protected open fun shouldDetermineComposingRegion(editorInfo: FlorisEditorInfo): Boolean =
        editorInfo.isRichInputEditor && !editorInfo.inputAttributes.flagTextNoSuggestions

    protected fun executeEditor(operation: EditorOp, vararg fields: String): Boolean {
        val ic = FlorisImeService.currentInputConnection() ?: run {
            editorSession.call(EditorOp.UNAVAILABLE, operation.code)
            return false
        }
        configure()
        val result = runBlocking { executor.execute(editorSession, ic, operation, fields) }
        return result
    }
    private fun configure() {
        editorSession.call(EditorOp.CONFIG, *with(EditorConfig) { row(RAW to activeInfo.isRawInputEditor.toStringBit(),
            COMPOSING to determineComposingEnabled().toStringBit(), DETERMINE_COMPOSING to shouldDetermineComposingRegion(activeInfo).toStringBit(),
            SPACE_PENDING to isSpacePending.toStringBit(), INPUT_TYPE to activeInfo.inputAttributes.raw.toString()) })
    }
    private fun publish() {
        val state = EditorSnapshot(editorSession.call(EditorOp.PUBLISHED))
        _activeContentFlow.value = state.content
        _activeCursorCapsModeFlow.value = InputAttributes.CapsMode.fromInt(state.caps)
        keyboardManager.reevaluateInputShiftState()
    }
    open fun handleStartInput(editorInfo: FlorisEditorInfo) {
        activeInfo = editorInfo
        editorSession.call(EditorOp.INPUT_INFO, editorInfo.initialCapsMode.toFlags().toString())
        _activeCursorCapsModeFlow.value = editorInfo.initialCapsMode
        _activeContentFlow.value = EditorContent.Unspecified
        FlorisImeService.currentInputConnection()?.requestCursorUpdates(
            InputConnection.CURSOR_UPDATE_MONITOR or InputConnection.CURSOR_UPDATE_IMMEDIATE)
    }
    open fun handleStartInputView(editorInfo: FlorisEditorInfo, isRestart: Boolean) {
        if (isRestart) reset()
        activeInfo = editorInfo
        val selection = editorInfo.initialSelection
        executeEditor(EditorOp.START, selection.start.toString(), selection.end.toString(), WireFlag.TRUE)
        publish()
    }
    protected fun handleMassSelectionUpdate(newSelection: EditorRange, composing: EditorRange) {
        executeEditor(EditorOp.MASS, *EditorSession.cursor(newSelection.start, newSelection.end, composing.start, composing.end))
        publish()
    }
    /** AudioWhisper: [newSelection] reports one of the keyboard's own edits that it has since followed with more. */
    protected fun isLateUpdate(newSelection: EditorRange, composing: EditorRange): Boolean =
        editorSession.isLate(newSelection.start, newSelection.end, composing.start, composing.end)
    open fun handleSelectionUpdate(oldSelection: EditorRange, newSelection: EditorRange, composing: EditorRange) {
        executeEditor(EditorOp.UPDATE, *EditorSession.cursor(newSelection.start, newSelection.end, composing.start, composing.end), WireFlag.TRUE)
        publish()
    }
    fun handleExtractedTextUpdate(extracted: android.view.inputmethod.ExtractedText) {
        val spans = extracted.text as? android.text.Spannable
        executeEditor(EditorOp.EXTRACTED, *with(EditorExtracted) { row(SELECTION_START to "${extracted.selectionStart}", SELECTION_END to "${extracted.selectionEnd}",
            START_OFFSET to "${extracted.startOffset}", PARTIAL_START to "${extracted.partialStartOffset}", PARTIAL_END to "${extracted.partialEndOffset}",
            TEXT to extracted.text?.toString().orEmpty(),
            COMPOSING_START to "${spans?.let(android.view.inputmethod.BaseInputConnection::getComposingSpanStart) ?: -1}",
            COMPOSING_END to "${spans?.let(android.view.inputmethod.BaseInputConnection::getComposingSpanEnd) ?: -1}") })
        publish()
    }
    open fun handleFinishInputView() = reset()
    open fun handleFinishInput() {
        reset()
        FlorisImeService.currentInputConnection()?.requestCursorUpdates(0)
    }
    protected open fun reset() {
        editorSession.call(EditorOp.RESET)
        activeInfo = FlorisEditorInfo.Unspecified
        _activeCursorCapsModeFlow.value = InputAttributes.CapsMode.NONE
        _activeContentFlow.value = EditorContent.Unspecified
    }
    protected fun setSelection(selection: EditorRange): Boolean =
        executeEditor(EditorOp.SELECT, selection.start.toString(), selection.end.toString())
    open fun commitChar(char: String): Boolean = commitChar(char, false, false, false)
    protected fun commitChar(char: String, deletePreviousSpace: Boolean, insertSpaceBeforeChar: Boolean, insertSpaceAfterChar: Boolean): Boolean =
        executeEditor(EditorOp.CHAR, *with(EditorChar) { row(TEXT to char, DELETE_SPACE_BEFORE to deletePreviousSpace.toStringBit(),
            SPACE_BEFORE to insertSpaceBeforeChar.toStringBit(), SPACE_AFTER to insertSpaceAfterChar.toStringBit()) })
    open fun commitText(text: String): Boolean = executeEditor(EditorOp.COMMIT, text)
    open fun finalizeComposingText(text: String): Boolean = executeEditor(EditorOp.FINALIZE, text)
    protected fun replaceTextBeforeCursor(length: Int, text: String): Boolean = executeEditor(EditorOp.REPLACE, length.toString(), text)
    protected suspend fun deleteAroundCursor(unit: OperationUnit, scope: OperationScope, n: Int = 0): Boolean =
        executeEditor(EditorOp.DELETE, (scope == OperationScope.AFTER_CURSOR).toStringBit(), (unit == OperationUnit.WORDS).toStringBit(), n.toString())
    fun refreshComposing() { executeEditor(EditorOp.REFRESH) }
    fun EditorContent.getTextBeforeCursor(n: Int): String = runBlocking {
        val text = textBeforeSelection
        text.takeLast(breakIterators.measureLastUChars(text, n, subtypeManager.activeSubtype.primaryLocale))
    }
    fun EditorContent.getTextAfterCursor(n: Int): String = runBlocking {
        val text = textAfterSelection
        text.take(breakIterators.measureUChars(text, n, subtypeManager.activeSubtype.primaryLocale))
    }
    fun meta(ctrl: Boolean = false, alt: Boolean = false, shift: Boolean = false): Int =
        (if (ctrl) 0x3000 else 0) or (if (alt) 0x12 else 0) or (if (shift) 0x41 else 0)
    fun sendArrow(code: Int, count: Int, shift: Boolean): Boolean =
        executeEditor(EditorOp.ARROW, code.toString(), count.toString(), shift.toStringBit())
    fun sendDownUpKeyEvent(keyEventCode: Int, metaState: Int = meta(), count: Int = 1): Boolean =
        executeEditor(EditorOp.KEYS, keyEventCode.toString(), metaState.toString(), count.toString())
    fun updateLastCommitPosition() { editorSession.call(EditorOp.LAST_COMMIT) }
    fun deleteMoveLastCommitPosition() { editorSession.call(EditorOp.LAST_DELETE) }
    data class LastCommitPosition(val pos: Int = -1)
    private fun snapshot() = EditorSnapshot(editorSession.call(EditorOp.SNAPSHOT))
    private fun Boolean.toStringBit() = WireFlag.encode(this)
}

/** An [EditorState] row. */
internal class EditorSnapshot(private val fields: Array<String>) {
    private fun int(field: Int) = fields[field].toInt()
    val content = with(EditorState) { EditorContent(fields[TEXT], int(OFFSET), EditorRange(int(SELECTION_START), int(SELECTION_END)),
        EditorRange(int(COMPOSING_START), int(COMPOSING_END)), EditorRange(int(WORD_START), int(WORD_END))) }
    val caps = int(EditorState.CAPS)
    val last = int(EditorState.LAST_COMMIT)
    val expected = WireFlag.decode(fields[EditorState.EXPECTED])
}
