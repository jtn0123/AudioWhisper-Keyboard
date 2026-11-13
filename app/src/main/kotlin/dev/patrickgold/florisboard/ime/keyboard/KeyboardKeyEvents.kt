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

import android.icu.lang.UCharacter
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import com.jtn0123.audiowhisper.reader.ReadAloud
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.editor.OperationUnit
import dev.patrickgold.florisboard.ime.onehanded.OneHandedMode
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.lib.devtools.LogTopic
import dev.patrickgold.florisboard.lib.devtools.flogError
import dev.patrickgold.florisboard.lib.util.InputMethodUtils
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showShortToast

internal fun KeyboardManager.handleInputKeyDown(data: KeyData) {
    if (heldByGifSearch(data)) return
    when (data.code) {
        KeyCode.ARROW_DOWN,
        KeyCode.ARROW_LEFT,
        KeyCode.ARROW_RIGHT,
        KeyCode.ARROW_UP,
        KeyCode.MOVE_START_OF_PAGE,
        KeyCode.MOVE_END_OF_PAGE,
        KeyCode.MOVE_START_OF_LINE,
        KeyCode.MOVE_END_OF_LINE -> {
            editorInstance.massSelection.begin()
        }
        KeyCode.SHIFT -> handleShiftDown(data)
        // AudioWhisper: dictating and reading aloud never overlap; starting one ends the other.
        KeyCode.VOICE_INPUT -> {
            ReadAloud.stop()
            FlorisImeService.pressMic()
        }
        KeyCode.READ_ALOUD -> {
            FlorisImeService.stopDictating()
            ReadAloud.keyDown()
        }
    }
}

internal fun KeyboardManager.handleInputKeyUp(data: KeyData) = activeState.batchEdit {
    // Only a backspace straight after an auto-correction or a dictated phrase undoes it.
    // The mic key is let through: stopping dictation types the last phrase after it.
    if (data.code != KeyCode.DELETE) autoCorrectionUndo = null
    if (data.code != KeyCode.DELETE && data.code != KeyCode.VOICE_INPUT) editorInstance.forgetDictation()
    if (isTypingIntoGifSearch && typeIntoGifSearch(data)) return@batchEdit
    when (data.code) {
        KeyCode.ARROW_DOWN,
        KeyCode.ARROW_LEFT,
        KeyCode.ARROW_RIGHT,
        KeyCode.ARROW_UP,
        KeyCode.MOVE_START_OF_PAGE,
        KeyCode.MOVE_END_OF_PAGE,
        KeyCode.MOVE_START_OF_LINE,
        KeyCode.MOVE_END_OF_LINE -> {
            editorInstance.massSelection.end()
            handleArrow(data.code)
        }
        KeyCode.CAPS_LOCK -> handleCapsLock()
        KeyCode.CHAR_WIDTH_SWITCHER -> handleCharWidthSwitch()
        KeyCode.CHAR_WIDTH_FULL -> handleCharWidthFull()
        KeyCode.CHAR_WIDTH_HALF -> handleCharWidthHalf()
        KeyCode.CLIPBOARD_CUT -> editorInstance.performClipboardCut()
        KeyCode.CLIPBOARD_COPY -> editorInstance.performClipboardCopy()
        KeyCode.CLIPBOARD_PASTE -> editorInstance.performClipboardPaste()
        KeyCode.CLIPBOARD_SELECT -> handleClipboardSelect()
        KeyCode.CLIPBOARD_SELECT_ALL -> editorInstance.performClipboardSelectAll()
        KeyCode.CLIPBOARD_CLEAR_HISTORY -> clipboardManager.clearHistory()
        KeyCode.CLIPBOARD_CLEAR_FULL_HISTORY -> clipboardManager.clearFullHistory()
        KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP -> {
            if (prefs.clipboard.clearPrimaryClipAffectsHistoryIfUnpinned.get()) {
                clipboardManager.primaryClip?.let { clipboardManager.deleteClip(it, onlyIfUnpinned = true) }
            }
            clipboardManager.updatePrimaryClip(null)
            scope.launch { appContext.showShortToast(R.string.clipboard__cleared_primary_clip) }
        }
        KeyCode.TOGGLE_COMPACT_LAYOUT -> scope.launch { toggleOneHandedMode() }
        KeyCode.COMPACT_LAYOUT_TO_LEFT -> scope.launch {
            prefs.keyboard.oneHandedMode.set(OneHandedMode.START)
            toggleOneHandedMode()
        }
        KeyCode.COMPACT_LAYOUT_TO_RIGHT -> scope.launch {
            prefs.keyboard.oneHandedMode.set(OneHandedMode.END)
            toggleOneHandedMode()
        }
        KeyCode.DELETE -> handleBackwardDelete(OperationUnit.CHARACTERS)
        KeyCode.DELETE_WORD -> handleBackwardDelete(OperationUnit.WORDS)
        KeyCode.ENTER -> handleEnter()
        KeyCode.FORWARD_DELETE -> handleForwardDelete(OperationUnit.CHARACTERS)
        KeyCode.FORWARD_DELETE_WORD -> handleForwardDelete(OperationUnit.WORDS)
        KeyCode.IME_SHOW_UI -> FlorisImeService.showUi()
        KeyCode.IME_HIDE_UI -> FlorisImeService.hideUi()
        KeyCode.IME_PREV_SUBTYPE -> subtypeManager.switchToPrevSubtype()
        KeyCode.IME_NEXT_SUBTYPE -> subtypeManager.switchToNextSubtype()
        KeyCode.IME_UI_MODE_TEXT -> activeState.imeUiMode = ImeUiMode.TEXT
        KeyCode.IME_UI_MODE_MEDIA -> activeState.imeUiMode = ImeUiMode.MEDIA
        KeyCode.IME_UI_MODE_CLIPBOARD -> activeState.imeUiMode = ImeUiMode.CLIPBOARD
        KeyCode.VOICE_INPUT -> FlorisImeService.releaseMic()
        KeyCode.READ_ALOUD -> ReadAloud.keyUp(appContext)
        KeyCode.LANGUAGE_SWITCH -> handleLanguageSwitch()
        KeyCode.REDO -> editorInstance.performRedo()
        KeyCode.SETTINGS -> FlorisImeService.launchSettings()
        KeyCode.SHIFT -> handleShiftUp(data)
        KeyCode.SPACE -> handleSpace(data)
        KeyCode.SYSTEM_INPUT_METHOD_PICKER -> InputMethodUtils.showImePicker(appContext)
        KeyCode.SHOW_SUBTYPE_PICKER -> {
            appContext.keyboardManager.value.activeState.isSubtypeSelectionVisible = true
        }
        KeyCode.SYSTEM_PREV_INPUT_METHOD -> FlorisImeService.switchToPrevInputMethod()
        KeyCode.SYSTEM_NEXT_INPUT_METHOD -> FlorisImeService.switchToNextInputMethod()
        KeyCode.TOGGLE_SMARTBAR_VISIBILITY -> scope.launch {
            prefs.smartbar.enabled.let { it.set(!it.get()) }
        }
        KeyCode.TOGGLE_ACTIONS_OVERFLOW -> {
            activeState.isActionsOverflowVisible = !activeState.isActionsOverflowVisible
        }
        KeyCode.TOGGLE_ACTIONS_EDITOR -> {
            activeState.isActionsEditorVisible = !activeState.isActionsEditorVisible
        }
        KeyCode.TOGGLE_INCOGNITO_MODE -> scope.launch { handleToggleIncognitoMode() }
        KeyCode.TOGGLE_AUTOCORRECT -> handleToggleAutocorrect()
        KeyCode.UNDO -> editorInstance.performUndo()
        KeyCode.VIEW_CHARACTERS -> activeState.keyboardMode = KeyboardMode.CHARACTERS
        KeyCode.VIEW_NUMERIC -> activeState.keyboardMode = KeyboardMode.NUMERIC
        KeyCode.VIEW_NUMERIC_ADVANCED -> activeState.keyboardMode = KeyboardMode.NUMERIC_ADVANCED
        KeyCode.VIEW_PHONE -> activeState.keyboardMode = KeyboardMode.PHONE
        KeyCode.VIEW_PHONE2 -> activeState.keyboardMode = KeyboardMode.PHONE2
        KeyCode.VIEW_SYMBOLS -> activeState.keyboardMode = KeyboardMode.SYMBOLS
        KeyCode.VIEW_SYMBOLS2 -> activeState.keyboardMode = KeyboardMode.SYMBOLS2
        else -> {
            if (activeState.imeUiMode == ImeUiMode.MEDIA) {
                autoCorrect()
                editorInstance.commitText(data.asString(isForDisplay = false))
                return@batchEdit
            }
            when (activeState.keyboardMode) {
                KeyboardMode.NUMERIC,
                KeyboardMode.NUMERIC_ADVANCED,
                KeyboardMode.PHONE,
                KeyboardMode.PHONE2 -> when (data.type) {
                    KeyType.CHARACTER,
                    KeyType.NUMERIC -> {
                        val text = data.asString(isForDisplay = false)
                        editorInstance.commitText(text)
                    }
                    else -> when (data.code) {
                        KeyCode.PHONE_PAUSE,
                        KeyCode.PHONE_WAIT -> {
                            val text = data.asString(isForDisplay = false)
                            editorInstance.commitText(text)
                        }
                    }
                }
                else -> when (data.type) {
                    KeyType.CHARACTER, KeyType.NUMERIC ->{
                        val text = data.asString(isForDisplay = false)
                        val codePoint = UCharacter.codePointAt(text, 0)
                        if (!UCharacter.isUAlphabetic(codePoint)) {
                            autoCorrect(triggerCodePoint = codePoint)
                        }
                        nlpManager.tapTrail.typed(text)
                        editorInstance.commitChar(text)
                    }
                    else -> {
                        flogError(LogTopic.KEY_EVENTS) { "Received unknown key" }
                    }
                }
            }
            activeState.inputShiftState = ShiftLogic.keyTyped(
                activeState.inputShiftState, shiftPressed = inputEventDispatcher.isPressed(KeyCode.SHIFT),
            )
        }
    }
}

internal fun KeyboardManager.handleInputKeyCancel(data: KeyData) {
    if (heldByGifSearch(data)) return
    when (data.code) {
        KeyCode.ARROW_DOWN,
        KeyCode.ARROW_LEFT,
        KeyCode.ARROW_RIGHT,
        KeyCode.ARROW_UP,
        KeyCode.MOVE_START_OF_PAGE,
        KeyCode.MOVE_END_OF_PAGE,
        KeyCode.MOVE_START_OF_LINE,
        KeyCode.MOVE_END_OF_LINE -> {
            editorInstance.massSelection.end()
        }
        KeyCode.SHIFT -> handleShiftCancel()
        KeyCode.VOICE_INPUT -> FlorisImeService.releaseMic()
        KeyCode.READ_ALOUD -> ReadAloud.keyCancel()
    }
}

internal fun KeyboardManager.handleInputKeyRepeat(data: KeyData) {
    FlorisImeService.inputFeedbackController()?.keyRepeatedAction(data)
    // AudioWhisper: a held arrow would move the app's cursor; onInputKeyUp asks the GIF search.
    if (heldByGifSearch(data)) return onInputKeyUp(data)
    when (data.code) {
        KeyCode.ARROW_DOWN,
        KeyCode.ARROW_LEFT,
        KeyCode.ARROW_RIGHT,
        KeyCode.ARROW_UP,
        KeyCode.MOVE_START_OF_PAGE,
        KeyCode.MOVE_END_OF_PAGE,
        KeyCode.MOVE_START_OF_LINE,
        KeyCode.MOVE_END_OF_LINE -> handleArrow(data.code)
        else -> onInputKeyUp(data)
    }
}
