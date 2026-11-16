/* Copyright (C) 2022-2025 The FlorisBoard Contributors; Apache-2.0. */
package dev.patrickgold.florisboard.ime.editor

import android.content.Context
import android.view.inputmethod.InputConnection
import com.jtn0123.audiowhisper.rust.EditorSession
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

    protected fun executeEditor(operation: String, vararg fields: String): Boolean {
        val ic = FlorisImeService.currentInputConnection() ?: run {
            editorSession.call("unavailable", operation)
            return false
        }
        configure()
        val result = runBlocking { executor.execute(editorSession, ic, operation, fields) }
        return result
    }
    private fun configure() {
        editorSession.call("config", activeInfo.isRawInputEditor.toStringBit(),
            determineComposingEnabled().toStringBit(), shouldDetermineComposingRegion(activeInfo).toStringBit(),
            isSpacePending.toStringBit(), activeInfo.inputAttributes.raw.toString())
    }
    private fun publish() {
        val state = EditorSnapshot(editorSession.call("published"))
        _activeContentFlow.value = state.content
        _activeCursorCapsModeFlow.value = InputAttributes.CapsMode.fromInt(state.caps)
        keyboardManager.reevaluateInputShiftState()
    }
    open fun handleStartInput(editorInfo: FlorisEditorInfo) {
        activeInfo = editorInfo
        editorSession.call("inputInfo", editorInfo.initialCapsMode.toFlags().toString())
        _activeCursorCapsModeFlow.value = editorInfo.initialCapsMode
        _activeContentFlow.value = EditorContent.Unspecified
        FlorisImeService.currentInputConnection()?.requestCursorUpdates(
            InputConnection.CURSOR_UPDATE_MONITOR or InputConnection.CURSOR_UPDATE_IMMEDIATE)
    }
    open fun handleStartInputView(editorInfo: FlorisEditorInfo, isRestart: Boolean) {
        if (isRestart) reset()
        activeInfo = editorInfo
        val selection = editorInfo.initialSelection
        executeEditor("start", selection.start.toString(), selection.end.toString(), "1")
        publish()
    }
    protected fun handleMassSelectionUpdate(newSelection: EditorRange, composing: EditorRange) {
        executeEditor("mass", newSelection.start.toString(), newSelection.end.toString(), composing.start.toString(), composing.end.toString())
        publish()
    }
    /** AudioWhisper: [newSelection] reports one of the keyboard's own edits that it has since followed with more. */
    protected fun isLateUpdate(newSelection: EditorRange, composing: EditorRange): Boolean =
        editorSession.isLate(newSelection.start, newSelection.end, composing.start, composing.end)
    open fun handleSelectionUpdate(oldSelection: EditorRange, newSelection: EditorRange, composing: EditorRange) {
        executeEditor("update", newSelection.start.toString(), newSelection.end.toString(), composing.start.toString(), composing.end.toString(), "1")
        publish()
    }
    fun handleExtractedTextUpdate(extracted: android.view.inputmethod.ExtractedText) {
        val spans = extracted.text as? android.text.Spannable
        executeEditor("extracted", extracted.selectionStart.toString(), extracted.selectionEnd.toString(),
            extracted.startOffset.toString(), extracted.partialStartOffset.toString(), extracted.partialEndOffset.toString(),
            extracted.text?.toString().orEmpty(),
            (spans?.let(android.view.inputmethod.BaseInputConnection::getComposingSpanStart) ?: -1).toString(),
            (spans?.let(android.view.inputmethod.BaseInputConnection::getComposingSpanEnd) ?: -1).toString())
        publish()
    }
    open fun handleFinishInputView() = reset()
    open fun handleFinishInput() {
        reset()
        FlorisImeService.currentInputConnection()?.requestCursorUpdates(0)
    }
    protected open fun reset() {
        editorSession.call("reset")
        activeInfo = FlorisEditorInfo.Unspecified
        _activeCursorCapsModeFlow.value = InputAttributes.CapsMode.NONE
        _activeContentFlow.value = EditorContent.Unspecified
    }
    protected fun setSelection(selection: EditorRange): Boolean =
        executeEditor("select", selection.start.toString(), selection.end.toString())
    open fun commitChar(char: String): Boolean = commitChar(char, false, false, false)
    protected fun commitChar(char: String, deletePreviousSpace: Boolean, insertSpaceBeforeChar: Boolean, insertSpaceAfterChar: Boolean): Boolean =
        executeEditor("char", char, deletePreviousSpace.toStringBit(), insertSpaceBeforeChar.toStringBit(), insertSpaceAfterChar.toStringBit())
    open fun commitText(text: String): Boolean = executeEditor("commit", text)
    open fun finalizeComposingText(text: String): Boolean = executeEditor("finalize", text)
    protected fun replaceTextBeforeCursor(length: Int, text: String): Boolean = executeEditor("replace", length.toString(), text)
    protected suspend fun deleteAroundCursor(unit: OperationUnit, scope: OperationScope, n: Int = 0): Boolean =
        executeEditor("delete", (scope == OperationScope.AFTER_CURSOR).toStringBit(), (unit == OperationUnit.WORDS).toStringBit(), n.toString())
    fun refreshComposing() { executeEditor("refresh") }
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
        executeEditor("arrow", code.toString(), count.toString(), shift.toStringBit())
    fun sendDownUpKeyEvent(keyEventCode: Int, metaState: Int = meta(), count: Int = 1): Boolean =
        executeEditor("keys", keyEventCode.toString(), metaState.toString(), count.toString())
    fun updateLastCommitPosition() { editorSession.call("lastCommit") }
    fun deleteMoveLastCommitPosition() { editorSession.call("lastDelete") }
    data class LastCommitPosition(val pos: Int = -1)
    private fun snapshot() = EditorSnapshot(editorSession.call("snapshot"))
    private fun Boolean.toStringBit() = if (this) "1" else "0"
}

internal class EditorSnapshot(fields: Array<String>) {
    val content = EditorContent(fields[0], fields[1].toInt(), EditorRange(fields[2].toInt(), fields[3].toInt()),
        EditorRange(fields[4].toInt(), fields[5].toInt()), EditorRange(fields[6].toInt(), fields[7].toInt()))
    val caps = fields[8].toInt()
    val last = fields[9].toInt()
    val expected = fields[10] == "1"
}
