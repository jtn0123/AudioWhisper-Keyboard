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

import androidx.compose.ui.unit.LayoutDirection
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.text.key.KeyVariation

/**
 * This class is a helper managing the state of the text input logic which
 * affects the keyboard view in rendering and layouting the keys.
 *
 * The state class can hold flags or small unsigned integers, all added up
 * at max 64-bit though.
 *
 * The structure of this 8-byte state register is as follows: (Lower 4 bytes are pretty experimental rn)
 *
 * <Byte 3> | <Byte 2> | <Byte 1> | <Byte 0> | Description
 * ---------|----------|----------|----------|---------------------------------
 *          |          |          |     1111 | Active [KeyboardMode]
 *          |          |          | 1111     | Active [KeyVariation]
 *          |          |       11 |          | InputShiftState
 *          |          |      1   |          | Is selection active (length > 0)
 *          |          |     1    |          | Is manual selection mode
 *          |          |    1     |          | Is manual selection mode (start)
 *          |          |   1      |          | Is manual selection mode (end)
 *          |          | 1        |          | Is incognito mode
 *          |        1 |          |          | Is quick actions overflow visible
 *          |       1  |          |          | Is quick actions editor visible
 *          |    1     |          |          | Is composing enabled
 *          |   1      |          |          | Is character half-width enabled
 *      111 |          |          |          | Ime Ui Mode
 *     1    |          |          |          | Layout Direction (0=LTR, 1=RTL)
 *
 * <Byte 7> | <Byte 6> | <Byte 5> | <Byte 4> | Description
 * ---------|----------|----------|----------|---------------------------------
 *          |          |          |        1 | Subtype selection dialog visible
 *
 * The resulting structure is only relevant during a runtime lifespan and
 * thus can easily be changed without worrying about destroying some saved state.
 *
 * @property rawValue The internal register used to store the flags and region ints that
 *  this keyboard state represents.
 */
open class ReferenceKeyboardState protected constructor(open var rawValue: ULong) {
    companion object {
        const val M_KEYBOARD_MODE: ULong =                  0x0Fu
        const val O_KEYBOARD_MODE: Int =                    0
        const val M_KEY_VARIATION: ULong =                  0x0Fu
        const val O_KEY_VARIATION: Int =                    4
        const val M_INPUT_SHIFT_STATE: ULong =              0x03u
        const val O_INPUT_SHIFT_STATE: Int =                8
        const val M_IME_UI_MODE: ULong =                    0x07u
        const val O_IME_UI_MODE: Int =                      24

        const val F_IS_SELECTION_MODE: ULong =              0x00000400u
        const val F_IS_MANUAL_SELECTION_MODE: ULong =       0x00000800u
        const val F_IS_MANUAL_SELECTION_MODE_START: ULong = 0x00001000u
        const val F_IS_MANUAL_SELECTION_MODE_END: ULong =   0x00002000u
        const val F_IS_INCOGNITO_MODE: ULong =              0x00008000u
        const val F_IS_ACTIONS_OVERFLOW_VISIBLE: ULong =    0x00010000u
        const val F_IS_ACTIONS_EDITOR_VISIBLE: ULong =      0x00020000u
        const val F_IS_COMPOSING_ENABLED: ULong =           0x00100000u

        const val F_IS_CHAR_HALF_WIDTH: ULong =             0x00200000u

        const val F_IS_RTL_LAYOUT_DIRECTION: ULong =        0x08000000u

        const val F_IS_SUBTYPE_SELECTION_VISIBLE: ULong =   0x1_0000_0000u

        const val STATE_ALL_ZERO: ULong =                   0uL

        fun new(value: ULong = STATE_ALL_ZERO) = ReferenceKeyboardState(value)
    }

    fun snapshot(): ReferenceKeyboardState {
        return new(rawValue)
    }

    private fun getFlag(f: ULong): Boolean {
        return (rawValue and f) != STATE_ALL_ZERO
    }

    private fun setFlag(f: ULong, v: Boolean) {
        rawValue = if (v) { rawValue or f } else { rawValue and f.inv() }
    }

    private fun getRegion(m: ULong, o: Int): Int {
        return ((rawValue shr o) and m).toInt()
    }

    private fun setRegion(m: ULong, o: Int, v: Int) {
        rawValue = (rawValue and (m shl o).inv()) or ((v.toULong() and m) shl o)
    }

    override fun hashCode(): Int {
        return rawValue.hashCode()
    }

    override fun equals(other: Any?): Boolean {
        return other is ReferenceKeyboardState && other.rawValue == rawValue
    }

    override fun toString(): String {
        return "0x" + rawValue.toString(16).padStart(16, '0')
    }

    var keyVariation: KeyVariation
        get() = KeyVariation.fromInt(getRegion(M_KEY_VARIATION, O_KEY_VARIATION))
        set(v) { setRegion(M_KEY_VARIATION, O_KEY_VARIATION, v.toInt()) }

    var keyboardMode: KeyboardMode
        get() = KeyboardMode.fromInt(getRegion(M_KEYBOARD_MODE, O_KEYBOARD_MODE))
        set(v) { setRegion(M_KEYBOARD_MODE, O_KEYBOARD_MODE, v.toInt()) }

    var inputShiftState: InputShiftState
        get() = InputShiftState.fromInt(getRegion(M_INPUT_SHIFT_STATE, O_INPUT_SHIFT_STATE))
        set(v) { setRegion(M_INPUT_SHIFT_STATE, O_INPUT_SHIFT_STATE, v.toInt()) }

    var imeUiMode: ImeUiMode
        get() = ImeUiMode.fromInt(getRegion(M_IME_UI_MODE, O_IME_UI_MODE))
        set(v) { setRegion(M_IME_UI_MODE, O_IME_UI_MODE, v.toInt()) }

    var layoutDirection: LayoutDirection
        get() = if (getFlag(F_IS_RTL_LAYOUT_DIRECTION)) LayoutDirection.Rtl else LayoutDirection.Ltr
        set(v) { setFlag(F_IS_RTL_LAYOUT_DIRECTION, v == LayoutDirection.Rtl) }

    val isLowercase: Boolean
        get() = inputShiftState == InputShiftState.UNSHIFTED

    val isUppercase: Boolean
        get() = inputShiftState != InputShiftState.UNSHIFTED

    var isSelectionMode: Boolean
        get() = getFlag(F_IS_SELECTION_MODE)
        set(v) { setFlag(F_IS_SELECTION_MODE, v) }

    var isManualSelectionMode: Boolean
        get() = getFlag(F_IS_MANUAL_SELECTION_MODE)
        set(v) { setFlag(F_IS_MANUAL_SELECTION_MODE, v) }

    var isManualSelectionModeStart: Boolean
        get() = getFlag(F_IS_MANUAL_SELECTION_MODE_START)
        set(v) { setFlag(F_IS_MANUAL_SELECTION_MODE_START, v) }

    var isManualSelectionModeEnd: Boolean
        get() = getFlag(F_IS_MANUAL_SELECTION_MODE_END)
        set(v) { setFlag(F_IS_MANUAL_SELECTION_MODE_END, v) }

    var isCursorMode: Boolean
        get() = !isSelectionMode
        set(v) { isSelectionMode = !v }

    var isIncognitoMode: Boolean
        get() = getFlag(F_IS_INCOGNITO_MODE)
        set(v) { setFlag(F_IS_INCOGNITO_MODE, v) }

    var isActionsOverflowVisible: Boolean
        get() = getFlag(F_IS_ACTIONS_OVERFLOW_VISIBLE)
        set(v) { setFlag(F_IS_ACTIONS_OVERFLOW_VISIBLE, v) }

    var isActionsEditorVisible: Boolean
        get() = getFlag(F_IS_ACTIONS_EDITOR_VISIBLE)
        set(v) { setFlag(F_IS_ACTIONS_EDITOR_VISIBLE, v) }

    var isSubtypeSelectionVisible: Boolean
        get() = getFlag(F_IS_SUBTYPE_SELECTION_VISIBLE)
        set(v) { setFlag(F_IS_SUBTYPE_SELECTION_VISIBLE, v) }

    var isComposingEnabled: Boolean
        get() = getFlag(F_IS_COMPOSING_ENABLED)
        set(v) { setFlag(F_IS_COMPOSING_ENABLED, v) }

    var isCharHalfWidth: Boolean
        get() = getFlag(F_IS_CHAR_HALF_WIDTH)
        set(v) { setFlag(F_IS_CHAR_HALF_WIDTH, v) }

}
