/* Original editor operations, with Android/manager I/O replaced by deterministic test adapters. Apache-2.0. */
package dev.patrickgold.florisboard.ime.editor
import kotlinx.coroutines.runBlocking
open class ReferenceAbstractEditorInstance(ic: FakeInputConnection) : ReferenceEditorState(ic) {
    fun setSelection(selection: ReferenceEditorRange): Boolean {
        if (activeInfo.isRawInputEditor) return false
        val content = activeContent
        if (content.selection == selection) return true
        val ic = currentFakeInputConnection() ?: return false
        ic.beginBatchEdit()
        runBlocking {
            val newContent = content
                .copy(localSelection = selection.translatedBy(-content.offset))
                .generateCopy(selection = selection)
            expectedContentQueue.push(newContent)
            ic.setSelection(selection.start, selection.end)
            ic.setComposingRegion(newContent.composing)
        }
        ic.endBatchEdit()
        return true
    }
    open fun commitChar(char: String): Boolean {
        return commitChar(
            char = char,
            deletePreviousSpace = false,
            insertSpaceBeforeChar = false,
            insertSpaceAfterChar = false,
        )
    }
    fun commitChar(
        char: String,
        deletePreviousSpace: Boolean,
        insertSpaceBeforeChar: Boolean,
        insertSpaceAfterChar: Boolean,
    ): Boolean {
        val content = activeContent
        val selection = content.selection
        val isSingleChar = runBlocking {
            breakIterators.measureUChars(char, 1, subtypeManager.activeSubtype.primaryLocale)
        } == char.length
        if (!isSingleChar || selection.isNotValid || selection.isSelectionMode || activeInfo.isRawInputEditor) {
            return commitTextInternal(char)
        }
        val ic = currentFakeInputConnection() ?: return false
        val composer = determineComposer(subtypeManager.activeSubtype.composer)
        val previous = content.textBeforeSelection.takeLast(composer.toRead.coerceAtLeast(if (deletePreviousSpace) 1 else 0))
        val (tempRm, tempText) = composer.getActions(previous, char)
        val rm = if (deletePreviousSpace && previous.isNotEmpty() && previous.last() == ' ') tempRm + 1 else tempRm
        val finalText = buildString(tempText.length + 2) {
            if (insertSpaceBeforeChar) append(' ')
            append(tempText)
            if (insertSpaceAfterChar) append(' ')
        }
        if (rm <= 0) {
            commitTextInternal(finalText)
        } else runBlocking {
            ic.beginBatchEdit()
            val newSelection = ReferenceEditorRange.cursor(selection.start - rm + finalText.length)
            val newContent = content.generateCopy(
                selection = newSelection,
                textBeforeSelection = buildString {
                    append(content.textBeforeSelection.dropLast(rm))
                    append(finalText)
                },
                selectedText = "",
            )
            expectedContentQueue.push(newContent)
            ic.setComposingRegion(content.selection.start - rm, content.selection.start)
            ic.setComposingText(finalText, 1)
            ic.setComposingRegion(newContent.composing)
            ic.endBatchEdit()
        }
        return true
    }
    open fun commitText(text: String): Boolean = commitTextInternal(text)
    fun commitTextInternal(text: String): Boolean {
        val ic = currentFakeInputConnection() ?: return false
        val content = activeContent
        val selection = content.selection
        ic.beginBatchEdit()
        ic.finishComposingText()
        if (activeInfo.isRawInputEditor) {
            ic.commitText(text, 1)
        } else runBlocking {
            val newSelection = ReferenceEditorRange.cursor(selection.start + text.length)
            val newContent = content.generateCopy(
                selection = newSelection,
                textBeforeSelection = buildString {
                    append(content.textBeforeSelection)
                    append(text)
                },
                selectedText = "",
            )
            expectedContentQueue.push(newContent)
            ic.commitText(text, 1)
            ic.setComposingRegion(newContent.composing)
        }
        ic.endBatchEdit()
        return true
    }
    open fun finalizeComposingText(text: String): Boolean {
        val ic = currentFakeInputConnection() ?: return false
        val content = activeContent
        val composing = content.composing
        ic.beginBatchEdit()
        if (activeInfo.isRawInputEditor || composing.isNotValid) {
            // Deliberate divergence: never leak the legacy invalid-finalize batch.
            ic.endBatchEdit()
            return false
        } else runBlocking {
            val newSelection = ReferenceEditorRange.cursor(composing.end + (text.length - content.composingText.length))
            val newContent = content.generateCopy(
                selection = newSelection,
                textBeforeSelection = buildString {
                    append(content.textBeforeSelection.removeSuffix(content.composingText))
                    append(text)
                },
                selectedText = "",
            )
            expectedContentQueue.push(newContent)
            ic.setComposingText(text, 1)
            ic.finishComposingText()
            _lastCommitPosition.handleCommit(newContent.selection)
        }
        ic.endBatchEdit()
        return true
    }
    fun replaceTextBeforeCursor(length: Int, text: String): Boolean {
        val ic = currentFakeInputConnection() ?: return false
        val content = activeContent
        val before = content.textBeforeSelection
        if (activeInfo.isRawInputEditor || !content.selection.isCursorMode || before.length < length) return false
        runBlocking {
            val newContent = content.generateCopy(
                selection = ReferenceEditorRange.cursor(content.selection.start - length + text.length),
                textBeforeSelection = buildString {
                    append(before, 0, before.length - length)
                    append(text)
                },
            )
            expectedContentQueue.push(newContent)
            ic.beginBatchEdit()
            ic.finishComposingText()
            ic.deleteSurroundingText(length, 0)
            ic.commitText(text, 1)
            ic.setComposingRegion(newContent.composing)
            ic.endBatchEdit()
            _lastCommitPosition.handleCommit(newContent.selection)
        }
        return true
    }
    suspend fun deleteAroundCursor(unit: OperationUnit, scope: OperationScope, n: Int = 0): Boolean {
        val ic = currentFakeInputConnection()
        if (ic == null || n < 1) return false
        val content = activeContent
        val scopeText = when (scope) {
            OperationScope.BEFORE_CURSOR -> content.textBeforeSelection
            OperationScope.AFTER_CURSOR -> content.textAfterSelection
        }
        return (if (activeInfo.isRawInputEditor || scopeText.isEmpty()) {
            val keyEventCode = when (scope) {
                OperationScope.BEFORE_CURSOR -> 67
                OperationScope.AFTER_CURSOR -> 112
            }
            val metaState = when (unit) {
                OperationUnit.CHARACTERS -> meta()
                OperationUnit.WORDS -> meta(ctrl = true)
            }
            sendDownUpKeyEvent(keyEventCode, metaState, count = n)
        } else {
            val locale = subtypeManager.activeSubtype.primaryLocale
            when (scope) {
                OperationScope.BEFORE_CURSOR -> {
                    val length = when (unit) {
                        OperationUnit.CHARACTERS -> breakIterators.measureLastUChars(scopeText, n, locale)
                        OperationUnit.WORDS -> breakIterators.measureLastUWords(scopeText, n, locale)
                    }
                    val selection = content.selection
                    val newSelection = selection.translatedBy(-length)
                    val newContent = content.generateCopy(
                        selection = newSelection,
                        textBeforeSelection = scopeText.dropLast(length),
                    )
                    expectedContentQueue.push(newContent)
                    ic.beginBatchEdit()
                    ic.finishComposingText()
                    ic.deleteSurroundingText(length, 0)
                    ic.setComposingRegion(newContent.composing)
                    ic.endBatchEdit()
                }
                OperationScope.AFTER_CURSOR -> {
                    val length = when (unit) {
                        OperationUnit.CHARACTERS -> breakIterators.measureUChars(scopeText, n, locale)
                        OperationUnit.WORDS -> breakIterators.measureUWords(scopeText, n, locale)
                    }
                    val selection = content.selection
                    val newSelection = selection.translatedBy(length)
                    val newContent = content.generateCopy(
                        selection = newSelection,
                        textAfterSelection = scopeText.drop(length),
                    )
                    expectedContentQueue.push(newContent)
                    ic.beginBatchEdit()
                    ic.finishComposingText()
                    ic.deleteSurroundingText(0, length)
                    ic.setComposingRegion(newContent.composing)
                    ic.endBatchEdit()
                }
            }
            true
        }).also {
            deleteMoveLastCommitPosition()
        }
    }
    fun refreshComposing() {
        val content = activeContent
        val ic = currentFakeInputConnection()
        if (activeInfo.isRawInputEditor || ic == null) return
        runBlocking {
            val newContent = content.generateCopy()
            if (newContent.composing != content.composing) {
                expectedContentQueue.push(newContent)
                ic.setComposingRegion(newContent.composing)
            }
        }
    }
    fun ReferenceEditorContent.getTextBeforeCursor(n: Int): String {
        if (n < 1 || text.isEmpty()) return ""
        return runBlocking {
            val text = textBeforeSelection
            val length = breakIterators.measureLastUChars(text, n, subtypeManager.activeSubtype.primaryLocale)
            text.takeLast(length)
        }
    }
    fun ReferenceEditorContent.getTextAfterCursor(n: Int): String {
        if (n < 1 || text.isEmpty()) return ""
        return runBlocking {
            val text = textAfterSelection
            val length = breakIterators.measureUChars(text, n, subtypeManager.activeSubtype.primaryLocale)
            text.take(length)
        }
    }
    fun meta(
        ctrl: Boolean = false,
        alt: Boolean = false,
        shift: Boolean = false,
    ): Int {
        var metaState = 0
        if (ctrl) {
            metaState = metaState or 0x1000 or 0x2000
        }
        if (alt) {
            metaState = metaState or 2 or 16
        }
        if (shift) {
            metaState = metaState or 1 or 64
        }
        return metaState
    }
    fun FakeInputConnection.sendDownKeyEvent(eventTime: Long, keyEventCode: Int, metaState: Int, repeat: Int = 0): Boolean = down(keyEventCode,metaState,repeat)
    fun FakeInputConnection.sendUpKeyEvent(eventTime: Long, keyEventCode: Int, metaState: Int): Boolean = up(keyEventCode,metaState)
    fun sendDownUpKeyEvent(keyEventCode: Int, metaState: Int = meta(), count: Int = 1): Boolean {
        if (count < 1) return false
        val ic = currentFakeInputConnection() ?: return false
        ic.beginBatchEdit()
        val eventTime = 0L
        if (metaState and 0x1000 != 0) {
            ic.sendDownKeyEvent(eventTime, 113, 0)
        }
        if (metaState and 2 != 0) {
            ic.sendDownKeyEvent(eventTime, 57, 0)
        }
        if (metaState and 1 != 0) {
            ic.sendDownKeyEvent(eventTime, 59, 0)
        }
        for (n in 0 until count) {
            ic.sendDownKeyEvent(eventTime, keyEventCode, metaState, n)
        }
        ic.sendUpKeyEvent(eventTime, keyEventCode, metaState)
        if (metaState and 1 != 0) {
            ic.sendUpKeyEvent(eventTime, 59, 0)
        }
        if (metaState and 2 != 0) {
            ic.sendUpKeyEvent(eventTime, 57, 0)
        }
        if (metaState and 0x1000 != 0) {
            ic.sendUpKeyEvent(eventTime, 113, 0)
        }
        ic.endBatchEdit()
        return true
    }
}
