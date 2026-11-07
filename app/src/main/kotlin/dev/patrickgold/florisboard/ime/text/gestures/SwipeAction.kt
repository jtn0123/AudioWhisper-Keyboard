/*
 * Copyright (C) 2020-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.text.gestures

import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData

/**
 * Enum for declaring the possible actions for swipe gestures.
 */
enum class SwipeAction {
    NO_ACTION,
    CYCLE_TO_PREVIOUS_KEYBOARD_MODE,
    CYCLE_TO_NEXT_KEYBOARD_MODE,
    DELETE_CHARACTER,
    DELETE_CHARACTERS_PRECISELY,
    DELETE_WORD,
    DELETE_WORDS_PRECISELY,
    HIDE_KEYBOARD,
    INSERT_SPACE,
    MOVE_CURSOR_UP,
    MOVE_CURSOR_DOWN,
    MOVE_CURSOR_LEFT,
    MOVE_CURSOR_RIGHT,
    MOVE_CURSOR_START_OF_LINE,
    MOVE_CURSOR_END_OF_LINE,
    MOVE_CURSOR_START_OF_PAGE,
    MOVE_CURSOR_END_OF_PAGE,
    REDO,
    SELECT_CHARACTERS_PRECISELY,
    SELECT_WORDS_PRECISELY,
    SHIFT,
    SHOW_INPUT_METHOD_PICKER,
    SHOW_SUBTYPE_PICKER,
    SWITCH_TO_PREV_SUBTYPE,
    SWITCH_TO_NEXT_SUBTYPE,
    SWITCH_TO_CLIPBOARD_CONTEXT,
    SWITCH_TO_MEDIA_CONTEXT,
    SWITCH_TO_PREV_KEYBOARD,
    TOGGLE_SMARTBAR_VISIBILITY,
    UNDO;
}

/**
 * The key a swipe action sends, in keyboard [mode]; null for actions the touch
 * code carries out itself (the precise deletes and selects) and for none.
 *
 * AudioWhisper: moved out of KeyboardManager.executeSwipeAction so a test can
 * check what every action does while the GIF search box is open.
 */
fun SwipeAction.keyData(mode: KeyboardMode): TextKeyData? = when (this) {
    SwipeAction.CYCLE_TO_PREVIOUS_KEYBOARD_MODE -> when (mode) {
        KeyboardMode.CHARACTERS -> TextKeyData.VIEW_NUMERIC_ADVANCED
        KeyboardMode.NUMERIC_ADVANCED -> TextKeyData.VIEW_SYMBOLS2
        KeyboardMode.SYMBOLS2 -> TextKeyData.VIEW_SYMBOLS
        else -> TextKeyData.VIEW_CHARACTERS
    }
    SwipeAction.CYCLE_TO_NEXT_KEYBOARD_MODE -> when (mode) {
        KeyboardMode.CHARACTERS -> TextKeyData.VIEW_SYMBOLS
        KeyboardMode.SYMBOLS -> TextKeyData.VIEW_SYMBOLS2
        KeyboardMode.SYMBOLS2 -> TextKeyData.VIEW_NUMERIC_ADVANCED
        else -> TextKeyData.VIEW_CHARACTERS
    }
    SwipeAction.DELETE_WORD -> TextKeyData.DELETE_WORD
    SwipeAction.HIDE_KEYBOARD -> TextKeyData.IME_HIDE_UI
    SwipeAction.INSERT_SPACE -> TextKeyData.SPACE
    SwipeAction.MOVE_CURSOR_DOWN -> TextKeyData.ARROW_DOWN
    SwipeAction.MOVE_CURSOR_UP -> TextKeyData.ARROW_UP
    SwipeAction.MOVE_CURSOR_LEFT -> TextKeyData.ARROW_LEFT
    SwipeAction.MOVE_CURSOR_RIGHT -> TextKeyData.ARROW_RIGHT
    SwipeAction.MOVE_CURSOR_START_OF_LINE -> TextKeyData.MOVE_START_OF_LINE
    SwipeAction.MOVE_CURSOR_END_OF_LINE -> TextKeyData.MOVE_END_OF_LINE
    SwipeAction.MOVE_CURSOR_START_OF_PAGE -> TextKeyData.MOVE_START_OF_PAGE
    SwipeAction.MOVE_CURSOR_END_OF_PAGE -> TextKeyData.MOVE_END_OF_PAGE
    SwipeAction.SHIFT -> TextKeyData.SHIFT
    SwipeAction.REDO -> TextKeyData.REDO
    SwipeAction.UNDO -> TextKeyData.UNDO
    SwipeAction.SHOW_INPUT_METHOD_PICKER -> TextKeyData.SYSTEM_INPUT_METHOD_PICKER
    SwipeAction.SHOW_SUBTYPE_PICKER -> TextKeyData.SHOW_SUBTYPE_PICKER
    SwipeAction.SWITCH_TO_CLIPBOARD_CONTEXT -> TextKeyData.IME_UI_MODE_CLIPBOARD
    SwipeAction.SWITCH_TO_MEDIA_CONTEXT -> TextKeyData.IME_UI_MODE_MEDIA
    SwipeAction.SWITCH_TO_PREV_SUBTYPE -> TextKeyData.IME_PREV_SUBTYPE
    SwipeAction.SWITCH_TO_NEXT_SUBTYPE -> TextKeyData.IME_NEXT_SUBTYPE
    SwipeAction.SWITCH_TO_PREV_KEYBOARD -> TextKeyData.SYSTEM_PREV_INPUT_METHOD
    SwipeAction.TOGGLE_SMARTBAR_VISIBILITY -> TextKeyData.TOGGLE_SMARTBAR_VISIBILITY
    else -> null
}
