
package dev.patrickgold.florisboard.ime.editor
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import org.florisboard.lib.kotlin.guardedByLock
import kotlin.math.max
import kotlin.math.min
open class ReferenceEditorState(val ic: FakeInputConnection) {
    companion object {
        const val NumCharsBeforeCursor: Int = 256
        const val NumCharsAfterCursor: Int = 128
        const val NumCharsSafeMarginBeforeCursor: Int = 128
        const val CursorUpdateAll: Int =
            FakeInputConnection.CURSOR_UPDATE_MONITOR or FakeInputConnection.CURSOR_UPDATE_IMMEDIATE
        const val CursorUpdateNone: Int = 0
    }
    val keyboardManager = ReferenceKeyboardManager()
    val subtypeManager = ReferenceSubtypeManager()
    val nlpManager = ReferenceNlpManager()
    val scope = CoroutineScope(Dispatchers.Unconfined)
    val breakIterators = ReferenceBreaks()
    var composeEnabled = true
    var determine = true
    var pendingSpace = false
    val testComposer = ReferenceComposer()
    val _activeInfoFlow = MutableStateFlow(ReferenceFlorisEditorInfo.Unspecified)
    val activeInfoFlow = _activeInfoFlow.asStateFlow()
    inline var activeInfo: ReferenceFlorisEditorInfo
        get() = activeInfoFlow.value
        set(v) {
            _activeInfoFlow.value = v
        }
    val _activeCursorCapsModeFlow = MutableStateFlow(ReferenceInputAttributes.CapsMode.NONE)
    val activeCursorCapsModeFlow = _activeCursorCapsModeFlow.asStateFlow()
    inline var activeCursorCapsMode: ReferenceInputAttributes.CapsMode
        get() = activeCursorCapsModeFlow.value
        set(v) {
            _activeCursorCapsModeFlow.value = v
        }
    val _activeContentFlow = MutableStateFlow(ReferenceEditorContent.Unspecified)
    val activeContentFlow = _activeContentFlow.asStateFlow()
    inline var activeContent: ReferenceEditorContent
        get() = expectedContent() ?: activeContentFlow.value
        set(v) {
            _activeContentFlow.value = v
        }
    val expectedContentQueue = ExpectedContentQueue()
    val _lastCommitPosition = LastCommitPosition()
    val lastCommitPosition
        get() = LastCommitPosition(_lastCommitPosition)
    fun expectedContent(): ReferenceEditorContent? {
        return runBlocking { expectedContentQueue.peekNewestOrNull() }
    }
    fun currentFakeInputConnection(): FakeInputConnection? = ic
    open fun handleStartInput(editorInfo: ReferenceFlorisEditorInfo) {
        activeInfo = editorInfo
        activeCursorCapsMode = editorInfo.initialCapsMode
        activeContent = ReferenceEditorContent.Unspecified
        currentFakeInputConnection()?.requestCursorUpdates(CursorUpdateAll)
    }
    open fun handleStartInputView(editorInfo: ReferenceFlorisEditorInfo, isRestart: Boolean) {
        if (isRestart) {
            reset() // Just to make sure our state is correct after a restart
        }
        val ic = currentFakeInputConnection()
        activeInfo = editorInfo
        var selection = editorInfo.initialSelection
        if (ic == null || selection.isNotValid || editorInfo.isRawInputEditor) {
            activeCursorCapsMode = ReferenceInputAttributes.CapsMode.NONE
            activeContent = ReferenceEditorContent.Unspecified
            keyboardManager.reevaluateInputShiftState()
            return
        }
        val textBeforeSelection = ic.getTextBeforeCursor(NumCharsBeforeCursor, 0) ?: ""
        val textAfterSelection = ic.getTextAfterCursor(NumCharsAfterCursor, 0) ?: ""
        val selectedText = ic.getSelectedText(0) ?: ""
        if (selection.length != selectedText.length) {
            selection = ReferenceEditorRange(textBeforeSelection.length, textBeforeSelection.length + selectedText.length)
        } else if (selection.start > 0 || selection.end > 0) {
            if (textBeforeSelection.isEmpty() && textAfterSelection.isEmpty() && selectedText.isEmpty()) {
                selection = ReferenceEditorRange(0, 0)
            }
        }
        scope.launch {
            val content = generateContent(
                editorInfo,
                selection,
                textBeforeSelection,
                textAfterSelection,
                selectedText,
            )
            activeCursorCapsMode = content.cursorCapsMode()
            activeContent = content
            keyboardManager.reevaluateInputShiftState()
            ic.setComposingRegion(content.composing)
        }
    }
    fun handleMassSelectionUpdate(newSelection: ReferenceEditorRange, composing: ReferenceEditorRange) {
        activeCursorCapsMode = ReferenceInputAttributes.CapsMode.NONE
        activeContent = ReferenceEditorContent.selectionOnly(newSelection)
        if (composing.isValid) {
            currentFakeInputConnection()?.setComposingRegion(ReferenceEditorRange.Unspecified)
        }
        _lastCommitPosition.handleUpdateSelection(newSelection)
    }
    open fun handleSelectionUpdate(oldSelection: ReferenceEditorRange, newSelection: ReferenceEditorRange, composing: ReferenceEditorRange) {
        val ic = currentFakeInputConnection()
        val editorInfo = activeInfo
        if (ic == null || newSelection.isNotValid || editorInfo.isRawInputEditor) {
            activeCursorCapsMode = ReferenceInputAttributes.CapsMode.NONE
            activeContent = ReferenceEditorContent.Unspecified
            keyboardManager.reevaluateInputShiftState()
            return
        }
        _lastCommitPosition.handleUpdateSelection(newSelection)
        val expected = runBlocking {
            expectedContentQueue.popUntilOrNull {
                it.selection == newSelection && it.composing == composing &&
                    it.textBeforeSelection.length >= NumCharsSafeMarginBeforeCursor.coerceAtMost(it.selection.start)
            }
        }
        if (expected != null) {
            activeCursorCapsMode = expected.cursorCapsMode()
            activeContent = expected
            keyboardManager.reevaluateInputShiftState()
            return
        }
        val textBeforeSelection =
            if (newSelection.start > 0) ic.getTextBeforeCursor(NumCharsBeforeCursor, 0) ?: "" else ""
        val textAfterSelection = ic.getTextAfterCursor(NumCharsAfterCursor, 0) ?: ""
        val selectedText = if (newSelection.isSelectionMode) ic.getSelectedText(0) ?: "" else ""
        scope.launch {
            val content = generateContent(
                editorInfo,
                newSelection,
                textBeforeSelection,
                textAfterSelection,
                selectedText,
            )
            activeCursorCapsMode = content.cursorCapsMode()
            activeContent = content
            keyboardManager.reevaluateInputShiftState()
            if (content.composing != composing) {
                ic.setComposingRegion(content.composing)
            }
        }
    }
    open fun handleFinishInputView() {
        reset()
    }
    open fun handleFinishInput() {
        reset()
        currentFakeInputConnection()?.requestCursorUpdates(CursorUpdateNone)
    }
    open fun reset() {
        activeInfo = ReferenceFlorisEditorInfo.Unspecified
        activeCursorCapsMode = ReferenceInputAttributes.CapsMode.NONE
        activeContent = ReferenceEditorContent.Unspecified
        runBlocking { expectedContentQueue.clear() }
        _lastCommitPosition.reset()
    }
    suspend fun generateContent(
        editorInfo: ReferenceFlorisEditorInfo,
        selection: ReferenceEditorRange,
        textBeforeSelection: CharSequence,
        textAfterSelection: CharSequence,
        selectedText: CharSequence,
    ): ReferenceEditorContent {
        val offset = selection.start - textBeforeSelection.length
        val localSelection = ReferenceEditorRange(
            start = textBeforeSelection.length,
            end = textBeforeSelection.length + selectedText.length,
        )
        if (offset < 0 || selection.translatedBy(-offset) != localSelection) {
            return ReferenceEditorContent.Unspecified
        }
        val localCurrentWord =
            if (shouldDetermineComposingRegion(editorInfo) && localSelection.isCursorMode && textBeforeSelection.isNotEmpty()) {
                determineLocalComposing(textBeforeSelection, _lastCommitPosition.pos - offset)
            } else {
                ReferenceEditorRange.Unspecified
            }
        val localComposing = if (determineComposingEnabled()) localCurrentWord else ReferenceEditorRange.Unspecified
        val text = buildString {
            append(textBeforeSelection)
            append(selectedText)
            append(textAfterSelection)
        }
        return ReferenceEditorContent(text, offset, localSelection, localComposing, localCurrentWord)
    }
    suspend fun ReferenceEditorContent.generateCopy(
        editorInfo: ReferenceFlorisEditorInfo = activeInfo,
        selection: ReferenceEditorRange = this.selection,
        textBeforeSelection: CharSequence = this.textBeforeSelection,
        textAfterSelection: CharSequence = this.textAfterSelection,
        selectedText: CharSequence = this.selectedText,
    ): ReferenceEditorContent {
        return generateContent(
            editorInfo, selection, textBeforeSelection, textAfterSelection, selectedText
        )
    }
    open val isSpacePending: Boolean
        get() = pendingSpace
    fun ReferenceEditorContent.cursorCapsMode(): ReferenceInputAttributes.CapsMode {
        if (localSelection.isNotValid) return ReferenceInputAttributes.CapsMode.NONE
        val cursor = localSelection.start
        val (before, offset) = if (isSpacePending) "${text.subSequence(0, cursor)} " to cursor + 1 else text to cursor
        return ReferenceInputAttributes.CapsMode.fromFlags(ReferenceCaps.getCapsMode(before, offset, activeInfo.inputAttributes.raw))
    }
    fun determineComposingEnabled(): Boolean = composeEnabled
    fun determineComposer(composerName: Unit): ReferenceComposer = testComposer
    open fun shouldDetermineComposingRegion(editorInfo: ReferenceFlorisEditorInfo): Boolean {
        return determine
    }
    suspend fun determineLocalComposing(
        textBeforeSelection: CharSequence, localLastCommitPosition: Int
    ): ReferenceEditorRange {
        return nlpManager.determineLocalComposing(textBeforeSelection, breakIterators, localLastCommitPosition)
    }
    fun FakeInputConnection.setComposingRegion(composing: ReferenceEditorRange) {
        if (composing.isValid) {
            this.setComposingRegion(composing.start, composing.end)
        } else {
            this.finishComposingText()
        }
    }
    class ExpectedContentQueue {
        val list = guardedByLock { mutableListOf<ReferenceEditorContent>() }
        suspend fun popUntilOrNull(predicate: (ReferenceEditorContent) -> Boolean): ReferenceEditorContent? {
            return list.withLock { list ->
                while (list.isNotEmpty()) {
                    val item = list.removeAt(0)
                    if (predicate(item)) return@withLock item
                }
                return@withLock null
            }
        }
        suspend fun push(item: ReferenceEditorContent) {
            list.withLock { list ->
                list.add(item)
            }
        }
        suspend fun peekNewestOrNull(): ReferenceEditorContent? {
            return list.withLock { list ->
                list.lastOrNull()
            }
        }
        suspend fun clear() {
            list.withLock { list ->
                list.clear()
            }
        }
    }
    fun updateLastCommitPosition() = _lastCommitPosition.handleCommit(activeContent.selection)
    fun deleteMoveLastCommitPosition() = _lastCommitPosition.handleDelete(activeContent.selection)
    data class LastCommitPosition(var pos: Int = -1) {
        constructor(other: LastCommitPosition): this(other.pos)
        fun reset() {
            pos = -1
        }
        fun handleCommit(selection: ReferenceEditorRange) {
            if (selection.isValid) {
                pos = max(selection.start, selection.end)
            } else {
                reset()
            }
        }
        fun handleUpdateSelection(selection: ReferenceEditorRange) {
            val start = min(selection.start, selection.end)
            if (start < pos) {
                reset()
            }
        }
        fun handleDelete(selection: ReferenceEditorRange) {
            val start = min(selection.start, selection.end)
            if (start < pos) {
                pos = start
            }
        }
    }
}
