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

package dev.patrickgold.florisboard.ime.input.reference

import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.editor.OperationUnit
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import com.jtn0123.audiowhisper.rust.InputProtocol.*

// Frozen pre-port routing. Android calls become recorded boundary effects; decisions are unchanged.
internal fun ReferenceRoutingManager.handleInputKeyDown(data: KeyData) {
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
            record(MASS_BEGIN)
        }
        KeyCode.SHIFT -> handleShiftDown(data)
        // AudioWhisper: dictating and reading aloud never overlap; starting one ends the other.
        KeyCode.VOICE_INPUT -> {
            record(STOP_READING)
            record(MIC_DOWN)
        }
        KeyCode.READ_ALOUD -> {
            record(STOP_DICTATION)
            record(READ_DOWN)
        }
    }
}

internal fun ReferenceRoutingManager.handleInputKeyUp(data: KeyData) = activeState.batchEdit {
    // Only a backspace straight after an auto-correction or a dictated phrase undoes it.
    // The mic key is let through: stopping dictation types the last phrase after it.
    if (data.code != KeyCode.DELETE) record(CLEAR_UNDO)
    if (data.code != KeyCode.DELETE && data.code != KeyCode.VOICE_INPUT) record(FORGET_DICTATION)
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
            record(MASS_END)
            handleArrow(data.code)
        }
        KeyCode.CAPS_LOCK -> handleCapsLock()
        KeyCode.CHAR_WIDTH_SWITCHER -> handleCharWidthSwitch()
        KeyCode.CHAR_WIDTH_FULL -> handleCharWidthFull()
        KeyCode.CHAR_WIDTH_HALF -> handleCharWidthHalf()
        KeyCode.CLIPBOARD_CUT -> record(CUT)
        KeyCode.CLIPBOARD_COPY -> record(COPY)
        KeyCode.CLIPBOARD_PASTE -> record(PASTE)
        KeyCode.CLIPBOARD_SELECT -> handleClipboardSelect()
        KeyCode.CLIPBOARD_SELECT_ALL -> record(SELECT_ALL)
        KeyCode.CLIPBOARD_CLEAR_HISTORY -> record(CLEAR_HISTORY)
        KeyCode.CLIPBOARD_CLEAR_FULL_HISTORY -> record(CLEAR_FULL_HISTORY)
        KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP -> record(CLEAR_CLIP)
        KeyCode.TOGGLE_COMPACT_LAYOUT -> record(ONE_HANDED,-1)
        KeyCode.COMPACT_LAYOUT_TO_LEFT -> record(ONE_HANDED,0)
        KeyCode.COMPACT_LAYOUT_TO_RIGHT -> record(ONE_HANDED,1)
        KeyCode.DELETE -> handleBackwardDelete(OperationUnit.CHARACTERS)
        KeyCode.DELETE_WORD -> handleBackwardDelete(OperationUnit.WORDS)
        KeyCode.ENTER -> handleEnter()
        KeyCode.FORWARD_DELETE -> handleForwardDelete(OperationUnit.CHARACTERS)
        KeyCode.FORWARD_DELETE_WORD -> handleForwardDelete(OperationUnit.WORDS)
        KeyCode.IME_SHOW_UI -> record(SHOW_UI)
        KeyCode.IME_HIDE_UI -> record(HIDE_UI)
        KeyCode.IME_PREV_SUBTYPE -> record(SUBTYPE,-1)
        KeyCode.IME_NEXT_SUBTYPE -> record(SUBTYPE,1)
        KeyCode.IME_UI_MODE_TEXT -> activeState.imeUiMode = ImeUiMode.TEXT
        KeyCode.IME_UI_MODE_MEDIA -> activeState.imeUiMode = ImeUiMode.MEDIA
        KeyCode.IME_UI_MODE_CLIPBOARD -> activeState.imeUiMode = ImeUiMode.CLIPBOARD
        KeyCode.VOICE_INPUT -> record(MIC_UP)
        KeyCode.READ_ALOUD -> record(READ_UP)
        KeyCode.LANGUAGE_SWITCH -> handleLanguageSwitch()
        KeyCode.REDO -> record(REDO_KEY)
        KeyCode.SETTINGS -> record(SETTINGS_UI)
        KeyCode.SHIFT -> handleShiftUp(data)
        KeyCode.SPACE -> handleSpace(data)
        KeyCode.SYSTEM_INPUT_METHOD_PICKER -> record(IME_PICKER)
        KeyCode.SHOW_SUBTYPE_PICKER -> {
            activeState.isSubtypeSelectionVisible = true
        }
        KeyCode.SYSTEM_PREV_INPUT_METHOD -> record(PREV_IME)
        KeyCode.SYSTEM_NEXT_INPUT_METHOD -> record(NEXT_IME)
        KeyCode.TOGGLE_SMARTBAR_VISIBILITY -> record(SMARTBAR)
        KeyCode.TOGGLE_ACTIONS_OVERFLOW -> {
            activeState.isActionsOverflowVisible = !activeState.isActionsOverflowVisible
        }
        KeyCode.TOGGLE_ACTIONS_EDITOR -> {
            activeState.isActionsEditorVisible = !activeState.isActionsEditorVisible
        }
        KeyCode.TOGGLE_INCOGNITO_MODE -> record(INCOGNITO)
        KeyCode.TOGGLE_AUTOCORRECT -> record(AUTOCORRECT_TOGGLE)
        KeyCode.UNDO -> record(UNDO_KEY)
        KeyCode.VIEW_CHARACTERS -> activeState.keyboardMode = KeyboardMode.CHARACTERS
        KeyCode.VIEW_NUMERIC -> activeState.keyboardMode = KeyboardMode.NUMERIC
        KeyCode.VIEW_NUMERIC_ADVANCED -> activeState.keyboardMode = KeyboardMode.NUMERIC_ADVANCED
        KeyCode.VIEW_PHONE -> activeState.keyboardMode = KeyboardMode.PHONE
        KeyCode.VIEW_PHONE2 -> activeState.keyboardMode = KeyboardMode.PHONE2
        KeyCode.VIEW_SYMBOLS -> activeState.keyboardMode = KeyboardMode.SYMBOLS
        KeyCode.VIEW_SYMBOLS2 -> activeState.keyboardMode = KeyboardMode.SYMBOLS2
        else -> {
            if (activeState.imeUiMode == ImeUiMode.MEDIA) {
                record(AUTO_CORRECT,-1)
                record(COMMIT_TEXT)
                return@batchEdit
            }
            when (activeState.keyboardMode) {
                KeyboardMode.NUMERIC,
                KeyboardMode.NUMERIC_ADVANCED,
                KeyboardMode.PHONE,
                KeyboardMode.PHONE2 -> when (data.type) {
                    KeyType.CHARACTER,
                    KeyType.NUMERIC -> {
                        record(COMMIT_TEXT)
                    }
                    else -> when (data.code) {
                        KeyCode.PHONE_PAUSE,
                        KeyCode.PHONE_WAIT -> {
                                record(COMMIT_TEXT)
                        }
                    }
                }
                else -> when (data.type) {
                    KeyType.CHARACTER, KeyType.NUMERIC ->{
                        if (!alphabetic) {
                            record(AUTO_CORRECT,1)
                        }
                        record(TAP_TYPED)
                        record(COMMIT_CHAR)
                    }
                    else -> {
                        record(UNKNOWN_KEY)
                    }
                }
            }
            activeState.inputShiftState = keyTyped()
        }
    }
}

internal fun ReferenceRoutingManager.handleInputKeyCancel(data: KeyData) {
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
            record(MASS_END)
        }
        KeyCode.SHIFT -> handleShiftCancel()
        KeyCode.VOICE_INPUT -> record(MIC_UP)
        KeyCode.READ_ALOUD -> record(READ_CANCEL)
    }
}

internal fun ReferenceRoutingManager.handleInputKeyRepeat(data: KeyData) {
    record(FEEDBACK_REPEAT,data.code.toLong())
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
