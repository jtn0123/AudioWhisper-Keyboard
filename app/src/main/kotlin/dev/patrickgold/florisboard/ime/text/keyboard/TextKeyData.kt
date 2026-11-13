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

package dev.patrickgold.florisboard.ime.text.keyboard

import android.icu.lang.UCharacter
import com.jtn0123.audiowhisper.rust.KeyModels
import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.Unicode
import dev.patrickgold.florisboard.lib.lowercase
import dev.patrickgold.florisboard.lib.uppercase
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient

/**
 * Data class which describes a single key and its attributes.
 *
 * @property type The type of the key. Some actions require both [code] and [type] to match in order
 *  to be successfully executed. Defaults to [KeyType.CHARACTER].
 * @property code The UTF-8 encoded code of the character. The code defined here is used as the
 *  data passed to the system. Defaults to 0.
 * @property label The string used to display the key in the UI. Is not used for the actual data
 *  passed to the system. Should normally be the exact same as the [code]. Defaults to an empty
 *  string.
 */
@Serializable
@SerialName("text_key")
data class TextKeyData(
    override val type: KeyType = KeyType.CHARACTER,
    override val code: Int = KeyCode.UNSPECIFIED,
    override val label: String = "",
    override val groupId: Int = KeyData.GROUP_DEFAULT,
    override val popup: PopupSet<AbstractKeyData>? = null
) : KeyData {
    override fun compute(evaluator: ComputingEvaluator): KeyData? {
        return if (evaluator.isSlot(this)) {
            evaluator.slotData(this)?.let { data ->
                TextKeyData(type, data.code, data.label, groupId, popup)
            }
        } else {
            this
        }
    }

    override fun asString(isForDisplay: Boolean): String {
        return asString(this, isForDisplay)
    }

    override fun toString(): String {
        return "${TextKeyData::class.simpleName} { type=$type code=$code label=\"$label\" groupId=$groupId }"
    }

    companion object {
        private val predefined = KeyModels.predefined().associate { entry ->
            val (code, type, label) = entry.split(',', limit = 3)
            code.toInt().let { it to TextKeyData(KeyType.entries[type.toInt()], it, label) }
        }

        fun getCodeInfoAsTextKeyData(code: Int): TextKeyData? =
            when (KeyModels.codeKind(code)) {
                1 -> predefined[code]
                2 -> TextKeyData(code = code, label = KeyModels.multiText(intArrayOf(code), "", false))
                else -> null
            }

        val UNSPECIFIED = predefined.getValue(KeyCode.UNSPECIFIED)
        val SPACE = predefined.getValue(KeyCode.SPACE)
        val CTRL = predefined.getValue(KeyCode.CTRL)
        val CTRL_LOCK = predefined.getValue(KeyCode.CTRL_LOCK)
        val ALT = predefined.getValue(KeyCode.ALT)
        val ALT_LOCK = predefined.getValue(KeyCode.ALT_LOCK)
        val FN = predefined.getValue(KeyCode.FN)
        val FN_LOCK = predefined.getValue(KeyCode.FN_LOCK)
        val DELETE = predefined.getValue(KeyCode.DELETE)
        val DELETE_WORD = predefined.getValue(KeyCode.DELETE_WORD)
        val FORWARD_DELETE = predefined.getValue(KeyCode.FORWARD_DELETE)
        val FORWARD_DELETE_WORD = predefined.getValue(KeyCode.FORWARD_DELETE_WORD)
        val SHIFT = predefined.getValue(KeyCode.SHIFT)
        val CAPS_LOCK = predefined.getValue(KeyCode.CAPS_LOCK)
        val ARROW_LEFT = predefined.getValue(KeyCode.ARROW_LEFT)
        val ARROW_RIGHT = predefined.getValue(KeyCode.ARROW_RIGHT)
        val ARROW_UP = predefined.getValue(KeyCode.ARROW_UP)
        val ARROW_DOWN = predefined.getValue(KeyCode.ARROW_DOWN)
        val MOVE_START_OF_PAGE = predefined.getValue(KeyCode.MOVE_START_OF_PAGE)
        val MOVE_END_OF_PAGE = predefined.getValue(KeyCode.MOVE_END_OF_PAGE)
        val MOVE_START_OF_LINE = predefined.getValue(KeyCode.MOVE_START_OF_LINE)
        val MOVE_END_OF_LINE = predefined.getValue(KeyCode.MOVE_END_OF_LINE)
        val CLIPBOARD_COPY = predefined.getValue(KeyCode.CLIPBOARD_COPY)
        val CLIPBOARD_CUT = predefined.getValue(KeyCode.CLIPBOARD_CUT)
        val CLIPBOARD_PASTE = predefined.getValue(KeyCode.CLIPBOARD_PASTE)
        val CLIPBOARD_SELECT = predefined.getValue(KeyCode.CLIPBOARD_SELECT)
        val CLIPBOARD_SELECT_ALL = predefined.getValue(KeyCode.CLIPBOARD_SELECT_ALL)
        val CLIPBOARD_CLEAR_HISTORY = predefined.getValue(KeyCode.CLIPBOARD_CLEAR_HISTORY)
        val CLIPBOARD_CLEAR_FULL_HISTORY = predefined.getValue(KeyCode.CLIPBOARD_CLEAR_FULL_HISTORY)
        val CLIPBOARD_CLEAR_PRIMARY_CLIP = predefined.getValue(KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP)
        val TOGGLE_COMPACT_LAYOUT = predefined.getValue(KeyCode.TOGGLE_COMPACT_LAYOUT)
        val COMPACT_LAYOUT_TO_LEFT = predefined.getValue(KeyCode.COMPACT_LAYOUT_TO_LEFT)
        val COMPACT_LAYOUT_TO_RIGHT = predefined.getValue(KeyCode.COMPACT_LAYOUT_TO_RIGHT)
        val UNDO = predefined.getValue(KeyCode.UNDO)
        val REDO = predefined.getValue(KeyCode.REDO)
        val VIEW_CHARACTERS = predefined.getValue(KeyCode.VIEW_CHARACTERS)
        val VIEW_SYMBOLS = predefined.getValue(KeyCode.VIEW_SYMBOLS)
        val VIEW_SYMBOLS2 = predefined.getValue(KeyCode.VIEW_SYMBOLS2)
        val VIEW_NUMERIC_ADVANCED = predefined.getValue(KeyCode.VIEW_NUMERIC_ADVANCED)
        val IME_UI_MODE_TEXT = predefined.getValue(KeyCode.IME_UI_MODE_TEXT)
        val IME_UI_MODE_MEDIA = predefined.getValue(KeyCode.IME_UI_MODE_MEDIA)
        val IME_UI_MODE_CLIPBOARD = predefined.getValue(KeyCode.IME_UI_MODE_CLIPBOARD)
        val SYSTEM_INPUT_METHOD_PICKER = predefined.getValue(KeyCode.SYSTEM_INPUT_METHOD_PICKER)
        val SHOW_SUBTYPE_PICKER = predefined.getValue(KeyCode.SHOW_SUBTYPE_PICKER)
        val SYSTEM_PREV_INPUT_METHOD = predefined.getValue(KeyCode.SYSTEM_PREV_INPUT_METHOD)
        val SYSTEM_NEXT_INPUT_METHOD = predefined.getValue(KeyCode.SYSTEM_NEXT_INPUT_METHOD)
        val IME_SUBTYPE_PICKER = predefined.getValue(KeyCode.IME_SUBTYPE_PICKER)
        val IME_PREV_SUBTYPE = predefined.getValue(KeyCode.IME_PREV_SUBTYPE)
        val IME_NEXT_SUBTYPE = predefined.getValue(KeyCode.IME_NEXT_SUBTYPE)
        val LANGUAGE_SWITCH = predefined.getValue(KeyCode.LANGUAGE_SWITCH)
        val IME_SHOW_UI = predefined.getValue(KeyCode.IME_SHOW_UI)
        val IME_HIDE_UI = predefined.getValue(KeyCode.IME_HIDE_UI)
        val SETTINGS = predefined.getValue(KeyCode.SETTINGS)
        val VOICE_INPUT = predefined.getValue(KeyCode.VOICE_INPUT)
        val READ_ALOUD = predefined.getValue(KeyCode.READ_ALOUD)
        val TOGGLE_SMARTBAR_VISIBILITY = predefined.getValue(KeyCode.TOGGLE_SMARTBAR_VISIBILITY)
        val TOGGLE_ACTIONS_OVERFLOW = predefined.getValue(KeyCode.TOGGLE_ACTIONS_OVERFLOW)
        val TOGGLE_ACTIONS_EDITOR = predefined.getValue(KeyCode.TOGGLE_ACTIONS_EDITOR)
        val TOGGLE_INCOGNITO_MODE = predefined.getValue(KeyCode.TOGGLE_INCOGNITO_MODE)
        val TOGGLE_AUTOCORRECT = predefined.getValue(KeyCode.TOGGLE_AUTOCORRECT)
    }
}

@Serializable
@SerialName("auto_text_key")
class AutoTextKeyData(
    override val type: KeyType = KeyType.CHARACTER,
    override val code: Int = KeyCode.UNSPECIFIED,
    override val label: String = "",
    override val groupId: Int = KeyData.GROUP_DEFAULT,
    override val popup: PopupSet<AbstractKeyData>? = null
) : KeyData {
    @Transient private val state = AutoLetterState()

    override fun compute(evaluator: ComputingEvaluator): KeyData? {
        return if (evaluator.isSlot(this)) {
            evaluator.slotData(this)?.let { data ->
                TextKeyData(type, data.code, data.label, groupId, popup)
            }
        } else {
            state.recomputeIfNecessary(evaluator.subtype.primaryLocale)
            if (KeyModels.select(0, evaluator.state.rawValue.toLong(), 3) == 0) state.lower else state.upper
        }
    }

    override fun asString(isForDisplay: Boolean): String {
        return asString(this, isForDisplay)
    }

    override fun toString(): String {
        return "${AutoTextKeyData::class.simpleName} { type=$type code=$code label=\"$label\" groupId=$groupId }"
    }

    private inner class AutoLetterState {
        private var locale: FlorisLocale = FlorisLocale.ROOT
        var lower: TextKeyData = TextKeyData.UNSPECIFIED
            private set
        var upper: TextKeyData = TextKeyData.UNSPECIFIED
            private set

        fun recomputeIfNecessary(locale: FlorisLocale) {
            if (this.locale == locale) return
            this.locale = locale
            lower = TextKeyData(
                type,
                UCharacter.toString(code).lowercase(locale).codePointAt(0),
                label.lowercase(locale),
                groupId,
                popup,
            )
            upper = TextKeyData(
                type,
                UCharacter.toString(code).uppercase(locale).codePointAt(0),
                label.uppercase(locale),
                groupId,
                popup,
            )
        }
    }
}

@Serializable
@SerialName("multi_text_key")
class MultiTextKeyData(
    override val type: KeyType = KeyType.CHARACTER,
    val codePoints: IntArray = intArrayOf(),
    override val label: String = "",
    override val groupId: Int = KeyData.GROUP_DEFAULT,
    override val popup: PopupSet<AbstractKeyData>? = null
) : KeyData {
    @Transient override val code: Int = KeyCode.MULTIPLE_CODE_POINTS

    override fun compute(evaluator: ComputingEvaluator): KeyData {
        return this
    }

    override fun asString(isForDisplay: Boolean): String {
        return KeyModels.multiText(codePoints, label, isForDisplay)
    }

    override fun toString(): String {
        return "${MultiTextKeyData::class.simpleName} { type=$type code=$code label=\"$label\" groupId=$groupId }"
    }
}

internal fun asString(data: KeyData, isForDisplay: Boolean): String =
    KeyModels.text(data.code, data.label, isForDisplay, Unicode.isNonSpacingMark(data.code))
