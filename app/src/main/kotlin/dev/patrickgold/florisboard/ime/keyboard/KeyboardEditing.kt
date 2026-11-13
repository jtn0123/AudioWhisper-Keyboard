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
import dev.patrickgold.florisboard.ime.text.key.UtilityKeyAction
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import java.lang.ref.WeakReference
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showLongToast

internal fun KeyboardManager.handleArrow(code: Int, count: Int = 1) = editorInstance.apply {
    val isShiftPressed = activeState.isManualSelectionMode || inputEventDispatcher.isPressed(KeyCode.SHIFT)
    val content = activeContent
    val selection = content.selection
    when (code) {
        KeyCode.ARROW_LEFT -> {
            if (!selection.isSelectionMode && activeState.isManualSelectionMode) {
                activeState.isManualSelectionModeStart = true
                activeState.isManualSelectionModeEnd = false
            }
            sendDownUpKeyEvent(KeyEvent.KEYCODE_DPAD_LEFT, meta(shift = isShiftPressed), count)
        }
        KeyCode.ARROW_RIGHT -> {
            if (!selection.isSelectionMode && activeState.isManualSelectionMode) {
                activeState.isManualSelectionModeStart = false
                activeState.isManualSelectionModeEnd = true
            }
            sendDownUpKeyEvent(KeyEvent.KEYCODE_DPAD_RIGHT, meta(shift = isShiftPressed), count)
        }
        KeyCode.ARROW_UP -> {
            if (!selection.isSelectionMode && activeState.isManualSelectionMode) {
                activeState.isManualSelectionModeStart = true
                activeState.isManualSelectionModeEnd = false
            }
            sendDownUpKeyEvent(KeyEvent.KEYCODE_DPAD_UP, meta(shift = isShiftPressed), count)
        }
        KeyCode.ARROW_DOWN -> {
            if (!selection.isSelectionMode && activeState.isManualSelectionMode) {
                activeState.isManualSelectionModeStart = false
                activeState.isManualSelectionModeEnd = true
            }
            sendDownUpKeyEvent(KeyEvent.KEYCODE_DPAD_DOWN, meta(shift = isShiftPressed), count)
        }
        KeyCode.MOVE_START_OF_PAGE -> {
            if (!selection.isSelectionMode && activeState.isManualSelectionMode) {
                activeState.isManualSelectionModeStart = true
                activeState.isManualSelectionModeEnd = false
            }
            sendDownUpKeyEvent(KeyEvent.KEYCODE_DPAD_UP, meta(alt = true, shift = isShiftPressed), count)
        }
        KeyCode.MOVE_END_OF_PAGE -> {
            if (!selection.isSelectionMode && activeState.isManualSelectionMode) {
                activeState.isManualSelectionModeStart = false
                activeState.isManualSelectionModeEnd = true
            }
            sendDownUpKeyEvent(KeyEvent.KEYCODE_DPAD_DOWN, meta(alt = true, shift = isShiftPressed), count)
        }
        KeyCode.MOVE_START_OF_LINE -> {
            if (!selection.isSelectionMode && activeState.isManualSelectionMode) {
                activeState.isManualSelectionModeStart = true
                activeState.isManualSelectionModeEnd = false
            }
            sendDownUpKeyEvent(KeyEvent.KEYCODE_DPAD_LEFT, meta(alt = true, shift = isShiftPressed), count)
        }
        KeyCode.MOVE_END_OF_LINE -> {
            if (!selection.isSelectionMode && activeState.isManualSelectionMode) {
                activeState.isManualSelectionModeStart = false
                activeState.isManualSelectionModeEnd = true
            }
            sendDownUpKeyEvent(KeyEvent.KEYCODE_DPAD_RIGHT, meta(alt = true, shift = isShiftPressed), count)
        }
    }
}

/**
 * Handles a [KeyCode.CLIPBOARD_SELECT] event.
 */
internal fun KeyboardManager.handleClipboardSelect() {
    val activeSelection = editorInstance.activeContent.selection
    activeState.isManualSelectionMode = if (activeSelection.isSelectionMode) {
        if (activeState.isManualSelectionMode && activeState.isManualSelectionModeStart) {
            editorInstance.setSelection(activeSelection.start, activeSelection.start)
        } else {
            editorInstance.setSelection(activeSelection.end, activeSelection.end)
        }
        false
    } else {
        !activeState.isManualSelectionMode
    }
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
internal fun KeyboardManager.handleBackwardDelete(unit: OperationUnit) {
    if (inputEventDispatcher.isPressed(KeyCode.SHIFT)) {
        return handleForwardDelete(unit)
    }
    nlpManager.dropSwipedWord()
    activeState.batchEdit {
        it.isManualSelectionMode = false
        it.isManualSelectionModeStart = false
        it.isManualSelectionModeEnd = false
    }
    if (unit == OperationUnit.CHARACTERS && (undoAutoCorrection() || editorInstance.undoDictation())) return
    revertPreviouslyAcceptedCandidate()
    // AudioWhisper: a letter of the word being typed goes; so does where it was tapped.
    if (unit == OperationUnit.CHARACTERS && editorInstance.activeContent.composingText.isNotEmpty()) {
        nlpManager.tapTrail.deleted()
    }
    editorInstance.deleteBackwards(unit)
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
    return editorInstance.revertAutoCorrection(revert)
}

/**
 * Handles a [KeyCode.FORWARD_DELETE] event.
 */
internal fun KeyboardManager.handleForwardDelete(unit: OperationUnit) {
    activeState.batchEdit {
        it.isManualSelectionMode = false
        it.isManualSelectionModeStart = false
        it.isManualSelectionModeEnd = false
    }
    revertPreviouslyAcceptedCandidate()
    editorInstance.deleteForwards(unit)
}

/**
 * Handles a [KeyCode.ENTER] event.
 */
internal fun KeyboardManager.handleEnter() {
    nlpManager.learnTypedWord(editorInstance.activeContent)
    val info = editorInstance.activeInfo
    val isShiftPressed = inputEventDispatcher.isPressed(KeyCode.SHIFT)
    if (editorInstance.tryPerformEnterCommitRaw()) {
        return
    }
    if (info.imeOptions.flagNoEnterAction || info.inputAttributes.flagTextMultiLine && isShiftPressed) {
        editorInstance.performEnter()
    } else {
        when (val action = info.imeOptions.action) {
            ImeOptions.Action.DONE,
            ImeOptions.Action.GO,
            ImeOptions.Action.NEXT,
            ImeOptions.Action.PREVIOUS,
            ImeOptions.Action.SEARCH,
            ImeOptions.Action.SEND -> {
                editorInstance.performEnterAction(action)
            }
            else -> editorInstance.performEnter()
        }
    }
}

/**
 * Handles a [KeyCode.LANGUAGE_SWITCH] event. Also handles if the language switch should cycle
 * FlorisBoard internal or system-wide.
 */
internal fun KeyboardManager.handleLanguageSwitch() {
    when (prefs.keyboard.utilityKeyAction.get()) {
        UtilityKeyAction.DYNAMIC_SWITCH_LANGUAGE_EMOJIS,
        UtilityKeyAction.SWITCH_LANGUAGE -> subtypeManager.switchToNextSubtype()
        else -> FlorisImeService.switchToNextInputMethod()
    }
}

/**
 * Handles a [KeyCode.SHIFT] down event.
 */
internal fun KeyboardManager.handleShiftDown(data: KeyData) {
    activeState.inputShiftState = ShiftLogic.shiftDown(
        activeState.inputShiftState,
        prefs.keyboard.capitalizationBehavior.get(),
        consecutiveDown = inputEventDispatcher.isConsecutiveDown(data),
    )
}

/**
 * Handles a [KeyCode.SHIFT] up event.
 */
internal fun KeyboardManager.handleShiftUp(data: KeyData) {
    activeState.inputShiftState = ShiftLogic.shiftUp(
        activeState.inputShiftState,
        anyPressed = inputEventDispatcher.isAnyPressed(),
        uninterrupted = inputEventDispatcher.isUninterruptedEventSequence(data),
    )
}

/**
 * Handles a [KeyCode.CAPS_LOCK] event.
 */
internal fun KeyboardManager.handleCapsLock() {
    activeState.inputShiftState = ShiftLogic.capsLock(activeState.inputShiftState)
}

/**
 * Handles a [KeyCode.SHIFT] cancel event.
 */
internal fun KeyboardManager.handleShiftCancel() {
    activeState.inputShiftState = ShiftLogic.shiftCancel(activeState.inputShiftState)
}

/**
 * Handles a hardware [KeyEvent.KEYCODE_SPACE] event. Same as [handleSpace],
 * but skips handling changing to characters keyboard and double space periods.
 */
internal fun KeyboardManager.handleHardwareKeyboardSpace() {
    val candidate = autoCorrect()
    // Skip handling changing to characters keyboard and double space periods
    // TODO: this is whether we commit space after selecting candidate. Should be determined by SuggestionProvider
    if (!subtypeManager.activeSubtype.primaryLocale.supportsAutoSpace &&
            candidate != null) { /* Do nothing */ } else {
        editorInstance.commitText(KeyCode.SPACE.toChar().toString())
    }
}

/**
 * Handles a [KeyCode.SPACE] event. Also handles the auto-correction of two space taps if
 * enabled by the user.
 */
internal fun KeyboardManager.handleSpace(data: KeyData) {
    val candidate = autoCorrect()
    if (prefs.keyboard.spaceBarSwitchesToCharacters.get()) {
        when (activeState.keyboardMode) {
            KeyboardMode.NUMERIC_ADVANCED,
            KeyboardMode.SYMBOLS,
            KeyboardMode.SYMBOLS2 -> {
                activeState.keyboardMode = KeyboardMode.CHARACTERS
            }
            else -> { /* Do nothing */ }
        }
    }
    if (prefs.correction.doubleSpacePeriod.get()) {
        if (inputEventDispatcher.isConsecutiveUp(data)) {
            val text = editorInstance.run { activeContent.getTextBeforeCursor(2) }
            if (ShiftLogic.isDoubleSpacePeriod(text)) {
                editorInstance.deleteBackwards(OperationUnit.CHARACTERS)
                editorInstance.commitText(". ")
                return
            }
        }
    }
    // TODO: this is whether we commit space after selecting candidate. Should be determined by SuggestionProvider
    if (!subtypeManager.activeSubtype.primaryLocale.supportsAutoSpace &&
            candidate != null) { /* Do nothing */ } else {
        editorInstance.commitText(KeyCode.SPACE.toChar().toString())
    }
}

/**
 * Handles a [KeyCode.TOGGLE_INCOGNITO_MODE] event.
 */
internal suspend fun KeyboardManager.handleToggleIncognitoMode() {
    prefs.suggestion.forceIncognitoModeFromDynamic.set(!prefs.suggestion.forceIncognitoModeFromDynamic.get())
    val newState = !activeState.isIncognitoMode
    activeState.isIncognitoMode = newState
    lastToastReference.get()?.cancel()
    lastToastReference = WeakReference(
        if (newState) {
            appContext.showLongToast(
                R.string.incognito_mode__toast_after_enabled,
                "app_name" to appContext.getString(R.string.floris_app_name),
            )
        } else {
            appContext.showLongToast(
                R.string.incognito_mode__toast_after_disabled,
                "app_name" to appContext.getString(R.string.floris_app_name),
            )
        }
    )
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

/**
 * Handles a [KeyCode.KANA_SWITCHER] event
 */
internal fun KeyboardManager.handleKanaSwitch() {
    activeState.batchEdit {
        it.isKanaKata = !it.isKanaKata
        it.isCharHalfWidth = false
    }
}

/**
 * Handles a [KeyCode.KANA_HIRA] event
 */
internal fun KeyboardManager.handleKanaHira() {
    activeState.batchEdit {
        it.isKanaKata = false
        it.isCharHalfWidth = false
    }
}

/**
 * Handles a [KeyCode.KANA_KATA] event
 */
internal fun KeyboardManager.handleKanaKata() {
    activeState.batchEdit {
        it.isKanaKata = true
        it.isCharHalfWidth = false
    }
}

/**
 * Handles a [KeyCode.KANA_HALF_KATA] event
 */
internal fun KeyboardManager.handleKanaHalfKata() {
    activeState.batchEdit {
        it.isKanaKata = true
        it.isCharHalfWidth = true
    }
}

/**
 * Handles a [KeyCode.CHAR_WIDTH_SWITCHER] event
 */
internal fun KeyboardManager.handleCharWidthSwitch() {
    activeState.isCharHalfWidth = !activeState.isCharHalfWidth
}

/**
 * Handles a [KeyCode.CHAR_WIDTH_SWITCHER] event
 */
internal fun KeyboardManager.handleCharWidthFull() {
    activeState.isCharHalfWidth = false
}

/**
 * Handles a [KeyCode.CHAR_WIDTH_SWITCHER] event
 */
internal fun KeyboardManager.handleCharWidthHalf() {
    activeState.isCharHalfWidth = true
}
