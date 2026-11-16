/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package dev.patrickgold.florisboard.ime.keyboard

import com.jtn0123.audiowhisper.rust.InputProtocol.*
import com.jtn0123.audiowhisper.rust.InputRules
import com.jtn0123.audiowhisper.rust.KeyModels
import android.view.KeyEvent
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import com.jtn0123.audiowhisper.suggest.AutoCorrectionUndo
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.editor.ImeOptions
import dev.patrickgold.florisboard.ime.editor.OperationUnit
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import java.lang.ref.WeakReference
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showLongToast

internal fun KeyboardManager.handleArrow(code: Int, count: Int = 1) = editorInstance.apply {
    val isShiftPressed = activeState.isManualSelectionMode || inputEventDispatcher.isPressed(KeyCode.SHIFT)
    val content = activeContent
    val selection = content.selection
    val plan = com.jtn0123.audiowhisper.rust.PanelRules.selection(intArrayOf(
        if (activeState.isManualSelectionMode) 1 else 0,
        if (activeState.isManualSelectionModeStart) 1 else 0,
        if (activeState.isManualSelectionModeEnd) 1 else 0, selection.start, selection.end, code))
    activeState.isManualSelectionModeStart = plan[1] != 0
    activeState.isManualSelectionModeEnd = plan[2] != 0
    sendArrow(code, count, isShiftPressed)
}

/**
 * Handles a [KeyCode.CLIPBOARD_SELECT] event.
 */
internal fun KeyboardManager.handleClipboardSelect() {
    val selection = editorInstance.activeContent.selection
    val plan = com.jtn0123.audiowhisper.rust.PanelRules.selection(intArrayOf(
        if (activeState.isManualSelectionMode) 1 else 0,
        if (activeState.isManualSelectionModeStart) 1 else 0,
        if (activeState.isManualSelectionModeEnd) 1 else 0, selection.start, selection.end, KeyCode.CLIPBOARD_SELECT))
    if (plan[3] >= 0) editorInstance.setSelection(plan[3], plan[3])
    activeState.isManualSelectionMode = plan[0] != 0

}

internal fun KeyboardManager.revertPreviouslyAcceptedCandidate() {
    editorInstance.phantomSpace.candidateForRevert?.let { candidateForRevert ->
        candidateForRevert.sourceProvider?.let { sourceProvider ->
            scope.launch {
                sourceProvider.notifySuggestionReverted(
                    subtype = subtypeManager.activeSubtype,
                    candidate = candidateForRevert,
                )
            }
        }
    }
}

/**
 * Handles a [KeyCode.DELETE] event.
 */
internal fun KeyboardManager.handleBackwardDelete(unit: OperationUnit) = executeDelete(unit, false)

/** Only effects and query replies: native continuation chooses whether deletion falls through. */
private fun KeyboardManager.executeDelete(unit: OperationUnit, forward: Boolean) {
    var stage = 0L
    var reply = false
    while (stage >= 0) {
        val plan = InputRules.control(longArrayOf(CONTROL_DELETE.toLong(), stage, unit.ordinal.toLong(),
            if(reply) 1 else 0, if(editorInstance.activeContent.composingText.isNotEmpty()) 1 else 0,
            if(forward) 1 else 0))
        stage = plan[0]
        for (offset in (1 until plan.size).step(2)) {
            val arg = plan[offset+1]
            when (plan[offset].toInt()) {
                DELETE_DROP -> nlpManager.dropSwipedWord()
                DELETE_STATE -> activeState.rawValue = KeyModels.write(activeState.rawValue.toLong(),
                    (arg ushr 32).toInt(), arg.toInt()).toULong()
                DELETE_UNDO_AUTO -> reply = undoAutoCorrection()
                DELETE_UNDO_DICTATION -> reply = editorInstance.undoDictation()
                DELETE_NOTIFY -> revertPreviouslyAcceptedCandidate()
                DELETE_TAP -> nlpManager.tapTrail.deleted()
                DELETE_BACKWARD_EDITOR -> editorInstance.deleteBackwards(OperationUnit.entries[arg.toInt()])
                DELETE_FORWARD_EDITOR -> editorInstance.deleteForwards(OperationUnit.entries[arg.toInt()])
                else -> error("Unknown native delete effect")
            }
        }
    }
}

/**
 * AudioWhisper: commits the auto-correction for the word being typed, if there is one, and
 * remembers it so that a backspace straight after can undo it.
 */
internal fun KeyboardManager.autoCorrect(triggerCodePoint: Int? = null): SuggestionCandidate? {
    val content = editorInstance.activeContent
    val typed = content.composingText
    val start = content.composing.start
    val candidate = nlpManager.getAutoCommitCandidate(triggerCodePoint)
    if (candidate == null) {
        nlpManager.learnTypedWord(content, triggerCodePoint)
        return null
    }
    commitCandidate(candidate)
    val corrected = candidate.text.toString()
    if (typed.isNotEmpty() && start >= 0 && corrected != typed) {
        autoCorrectionUndo = AutoCorrectionUndo(typed, corrected, start)
    }
    return candidate
}

/** AudioWhisper: puts back what was typed if the last key auto-corrected it. */
internal fun KeyboardManager.undoAutoCorrection(): Boolean {
    val undo = autoCorrectionUndo ?: return false
    autoCorrectionUndo = null
    val content = editorInstance.activeContent
    if (!content.selection.isCursorMode) return false
    val revert = undo.revertAt(content.selection.start, content.textBeforeSelection) ?: return false
    // Not corrected again this session if the user goes back into the word and finishes it.
    nlpManager.keepAsTyped(undo.typed)
    nlpManager.unlearnAutoCorrection(undo.start)
    return editorInstance.revertAutoCorrection(revert).also { if (it) nlpManager.noteCorrectionUndone() }
}

/**
 * Handles a [KeyCode.FORWARD_DELETE] event.
 */
internal fun KeyboardManager.handleForwardDelete(unit: OperationUnit) = executeDelete(unit, true)

/**
 * Handles a [KeyCode.ENTER] event.
 */
internal fun KeyboardManager.handleEnter() {
    nlpManager.noteKey()
    nlpManager.learnTypedWord(editorInstance.activeContent)
    val info = editorInstance.activeInfo
    val isShiftPressed = inputEventDispatcher.isPressed(KeyCode.SHIFT)
    if (editorInstance.tryPerformEnterCommitRaw()) {
        return
    }
    val action = com.jtn0123.audiowhisper.rust.EditorAttributes.enterAction(
        info.inputAttributes.raw, info.imeOptions.raw, isShiftPressed)
    if (action < 0) editorInstance.performEnter()
    else editorInstance.performEnterAction(ImeOptions.Action.fromInt(action))
}

/**
 * Handles a hardware [KeyEvent.KEYCODE_SPACE] event. Same as [handleSpace],
 * but skips handling changing to characters keyboard and double space periods.
 */
internal fun KeyboardManager.handleHardwareKeyboardSpace() = handleSpace(null)

/**
 * Handles a [KeyCode.SPACE] event. Also handles the auto-correction of two space taps if
 * enabled by the user.
 */
internal fun KeyboardManager.handleSpace(data: KeyData?) {
    val candidate = autoCorrect()
    val plan = com.jtn0123.audiowhisper.rust.InputRules.control(longArrayOf(
        com.jtn0123.audiowhisper.rust.InputProtocol.CONTROL_SPACE.toLong(), activeState.keyboardMode.toInt().toLong(),
        if(data != null && prefs.keyboard.spaceBarSwitchesToCharacters.get()) 1 else 0,
        if(data != null && prefs.correction.doubleSpacePeriod.get()) 1 else 0,
        if(data != null && inputEventDispatcher.isConsecutiveUp(data)) 1 else 0,
        if(subtypeManager.activeSubtype.primaryLocale.supportsAutoSpace) 1 else 0,
        if(candidate != null) 1 else 0), editorInstance.run {activeContent.getTextBeforeCursor(2)})
    activeState.keyboardMode = KeyboardMode.fromInt(plan[0].toInt())
    if(plan[1] != 0L) editorInstance.commitDoubleSpacePeriod()
    if(plan[2] != 0L) editorInstance.commitText(" ")
}

/**
 * Handles a [KeyCode.TOGGLE_INCOGNITO_MODE] event.
 */
internal suspend fun KeyboardManager.handleToggleIncognitoMode(force: Boolean, toast: Int) {
    prefs.suggestion.forceIncognitoModeFromDynamic.set(force)
    activeState.isIncognitoMode = toast != 0
    lastToastReference.get()?.cancel()
    val messages = intArrayOf(R.string.incognito_mode__toast_after_disabled, R.string.incognito_mode__toast_after_enabled)
    lastToastReference = WeakReference(appContext.showLongToast(messages[toast],
        "app_name" to appContext.getString(R.string.floris_app_name)))
}

/**
 * Handles a [KeyCode.TOGGLE_AUTOCORRECT] event.
 */
internal fun KeyboardManager.handleToggleAutocorrect() {
    lastToastReference.get()?.cancel()
    scope.launch {
        lastToastReference = WeakReference(
            appContext.showLongToast("Autocorrect toggle is a placeholder and not yet implemented")
        )
    }
}
