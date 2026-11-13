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

package dev.patrickgold.florisboard.ime.text.key

object KeyCode {
    private val codes = com.jtn0123.audiowhisper.rust.KeyModels.codes()
    object Spec {
        val CHARACTERS_MIN = codes[0]
        val CHARACTERS_MAX = codes[1]
        val CHARACTERS = CHARACTERS_MIN..CHARACTERS_MAX

        val INTERNAL_MIN = codes[2]
        val INTERNAL_MAX = codes[3]
        val INTERNAL = INTERNAL_MIN..INTERNAL_MAX
    }

    val UNSPECIFIED = codes[4]

    val PHONE_WAIT = codes[5] // ;
    val PHONE_PAUSE = codes[6] // ,

    val SPACE = codes[7]
    val ESCAPE = codes[8]
    val ENTER = codes[9]
    val TAB = codes[10]

    val CTRL = codes[11]
    val CTRL_LOCK = codes[12]
    val ALT = codes[13]
    val ALT_LOCK = codes[14]
    val FN = codes[15]
    val FN_LOCK = codes[16]
    val DELETE = codes[17]
    val DELETE_WORD = codes[18]
    val FORWARD_DELETE = codes[19]
    val FORWARD_DELETE_WORD = codes[20]
    val SHIFT = codes[21]
    val CAPS_LOCK = codes[22]

    val ARROW_LEFT = codes[23]
    val ARROW_RIGHT = codes[24]
    val ARROW_UP = codes[25]
    val ARROW_DOWN = codes[26]
    val MOVE_START_OF_PAGE = codes[27]
    val MOVE_END_OF_PAGE = codes[28]
    val MOVE_START_OF_LINE = codes[29]
    val MOVE_END_OF_LINE = codes[30]

    val CLIPBOARD_COPY = codes[31]
    val CLIPBOARD_CUT = codes[32]
    val CLIPBOARD_PASTE = codes[33]
    val CLIPBOARD_SELECT = codes[34]
    val CLIPBOARD_SELECT_ALL = codes[35]
    val CLIPBOARD_CLEAR_HISTORY = codes[36]
    val CLIPBOARD_CLEAR_FULL_HISTORY = codes[37]
    val CLIPBOARD_CLEAR_PRIMARY_CLIP = codes[38]

    val TOGGLE_COMPACT_LAYOUT = codes[39]
    val COMPACT_LAYOUT_TO_LEFT = codes[40]
    val COMPACT_LAYOUT_TO_RIGHT = codes[41]
    val SPLIT_LAYOUT = codes[42]
    val MERGE_LAYOUT = codes[43]

    val UNDO = codes[44]
    val REDO = codes[45]

    val VIEW_CHARACTERS = codes[46]
    val VIEW_SYMBOLS = codes[47]
    val VIEW_SYMBOLS2 = codes[48]
    val VIEW_NUMERIC = codes[49]
    val VIEW_NUMERIC_ADVANCED = codes[50]
    val VIEW_PHONE = codes[51]
    val VIEW_PHONE2 = codes[52]

    val IME_UI_MODE_TEXT = codes[53]
    val IME_UI_MODE_MEDIA = codes[54]
    val IME_UI_MODE_CLIPBOARD = codes[55]

    val SYSTEM_INPUT_METHOD_PICKER = codes[56]
    val SYSTEM_PREV_INPUT_METHOD = codes[57]
    val SYSTEM_NEXT_INPUT_METHOD = codes[58]
    val IME_SUBTYPE_PICKER = codes[59]
    val IME_PREV_SUBTYPE = codes[60]
    val IME_NEXT_SUBTYPE = codes[61]
    val LANGUAGE_SWITCH = codes[62]
    val SHOW_SUBTYPE_PICKER = codes[63]

    val IME_SHOW_UI = codes[64]
    val IME_HIDE_UI = codes[65]
    val VOICE_INPUT = codes[66]
    // AudioWhisper: the suggestion bar's speaker, which reads the screen aloud.
    val READ_ALOUD = codes[67]

    val TOGGLE_SMARTBAR_VISIBILITY = codes[68]
    val TOGGLE_ACTIONS_OVERFLOW = codes[69]
    val TOGGLE_ACTIONS_EDITOR = codes[70]
    val TOGGLE_INCOGNITO_MODE = codes[71]
    val TOGGLE_AUTOCORRECT = codes[72]

    val URI_COMPONENT_TLD = codes[73]

    val SETTINGS = codes[74]

    val CURRENCY_SLOT_1 = codes[75]
    val CURRENCY_SLOT_2 = codes[76]
    val CURRENCY_SLOT_3 = codes[77]
    val CURRENCY_SLOT_4 = codes[78]
    val CURRENCY_SLOT_5 = codes[79]
    val CURRENCY_SLOT_6 = codes[80]

    val MULTIPLE_CODE_POINTS = codes[81]
    val DRAG_MARKER = codes[82]
    val NOOP = codes[83]

    val CHAR_WIDTH_SWITCHER = codes[84]
    val CHAR_WIDTH_FULL = codes[85]
    val CHAR_WIDTH_HALF = codes[86]

    val KANA_SMALL = codes[87]
    val KANA_SWITCHER = codes[88]
    val KANA_HIRA = codes[89]
    val KANA_KATA = codes[90]
    val KANA_HALF_KATA = codes[91]

    val KESHIDA = codes[92]
    val HALF_SPACE = codes[93]

    val CJK_SPACE = codes[94]
}
