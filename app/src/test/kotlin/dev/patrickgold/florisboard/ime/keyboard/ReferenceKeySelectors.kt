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
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.key.KeyVariation

/**
 * Allows to select an [AbstractKeyData] based on the current caps state. Note that this type of selector only really
 * makes sense in a text context, though technically speaking it can be used anywhere, so this implementation allows
 * for any [AbstractKeyData] to be used here. The JSON class identifier for this selector is `case_selector`.
 *
 * Example usage in a layout JSON file:
 * ```
 * { "$": "case_selector",
 *   "lower": { "code":   59, "label": ";" },
 *   "upper": { "code":   58, "label": ":" }
 * }
 * ```
 *
 * @property lower The key data to use if the current caps state is lowercase.
 * @property upper The key data to use if the current caps state is uppercase.
 */
class ReferenceCaseSelector(
    val lower: AbstractKeyData,
    val upper: AbstractKeyData,
) : AbstractKeyData {
    override fun compute(evaluator: ComputingEvaluator): KeyData? {
        return (if (evaluator.state.isUppercase) { upper } else { lower }).compute(evaluator)
    }

    override fun asString(isForDisplay: Boolean): String {
        return ""
    }
}

/**
 * Allows to select an [AbstractKeyData] based on the current shift state. Note that this type of selector only really
 * makes sense in a text context, though technically speaking it can be used anywhere, so this implementation allows
 * for any [AbstractKeyData] to be used here. The JSON class identifier for this selector is `shift_state_selector`.
 *
 * Example usage in a layout JSON file:
 * ```
 * { "$": "shift_state_selector",
 *   "shiftedManual": { "code":   59, "label": ";" },
 *   "default": { "code":   58, "label": ":" }
 * }
 * ```
 *
 * @property unshifted The key data to use if the current shift state is [InputShiftState.UNSHIFTED], falling back to
 *  [default] if unspecified.
 * @property shifted The key data to use if the current shift state is either [InputShiftState.SHIFTED_MANUAL] or
 *  [InputShiftState.SHIFTED_AUTOMATIC]. Is overridden if [shiftedManual] or [shiftedAutomatic] is specified.
 * @property shiftedManual The key data to use if the current shift state is [InputShiftState.SHIFTED_MANUAL],
 *  falling back to [shifted] or [default] if unspecified.
 * @property shiftedAutomatic The key data to use if the current shift state is [InputShiftState.SHIFTED_AUTOMATIC],
 *  falling back to [shifted] or [default] if unspecified.
 * @property capsLock The key data to use if the current shift state is [InputShiftState.CAPS_LOCK], falling back to
 *  [default] if unspecified.
 * @property default The key data to use if the current shift state is set to a value not specified by this selector.
 *  If a key data is provided for all shift states possible this key data will never be used.
 */
class ReferenceShiftStateSelector(
    val unshifted: AbstractKeyData? = null,
    val shifted: AbstractKeyData? = null,
    val shiftedManual: AbstractKeyData? = null,
    val shiftedAutomatic: AbstractKeyData? = null,
    val capsLock: AbstractKeyData? = null,
    val default: AbstractKeyData? = null,
) : AbstractKeyData {
    override fun compute(evaluator: ComputingEvaluator): KeyData? {
        return when (evaluator.state.inputShiftState) {
            InputShiftState.UNSHIFTED -> unshifted ?: default
            InputShiftState.SHIFTED_MANUAL -> shiftedManual ?: shifted ?: default
            InputShiftState.SHIFTED_AUTOMATIC -> shiftedAutomatic ?: shifted ?: default
            InputShiftState.CAPS_LOCK -> capsLock ?: default
        }?.compute(evaluator)
    }

    override fun asString(isForDisplay: Boolean): String {
        return ""
    }
}

/**
 * Allows to select an [AbstractKeyData] based on the current variation. Note that this type of selector only really
 * makes sense in a text context, though technically speaking it can be used anywhere, so this implementation allows
 * for any [AbstractKeyData] to be used here. The JSON class identifier for this selector is `variation_selector`.
 *
 * Example usage in a layout JSON file:
 * ```
 * { "$": "variation_selector",
 *   "default":  { "code":   44, "label": "," },
 *   "email":    { "code":   64, "label": "@" },
 *   "uri":      { "code":   47, "label": "/" }
 * }
 * ```
 *
 * @property default The default key data which should be used in case no key variation is known or for the current
 *  key variation no override key is defined. Can be null, in this case this may mean the variation selector hides
 *  the key if no direct match is present.
 * @property email The key data to use if [KeyVariation.EMAIL_ADDRESS] is the active key variation. If this value is
 *  null, [default] will be used instead.
 * @property uri The key data to use if [KeyVariation.URI] is the active key variation. If this value is null,
 *  [default] will be used instead.
 * @property normal The key data to use if [KeyVariation.NORMAL] is the active key variation. If this value is null,
 *  [default] will be used instead.
 * @property password The key data to use if [KeyVariation.PASSWORD] is the active key variation. If this value is
 *  null, [default] will be used instead.
 */
data class ReferenceVariationSelector(
    val default: AbstractKeyData? = null,
    val email: AbstractKeyData? = null,
    val uri: AbstractKeyData? = null,
    val normal: AbstractKeyData? = null,
    val password: AbstractKeyData? = null,
) : AbstractKeyData {
    override fun compute(evaluator: ComputingEvaluator): KeyData? {
        return when (evaluator.state.keyVariation) {
            KeyVariation.ALL -> default
            KeyVariation.EMAIL_ADDRESS -> email ?: default
            KeyVariation.NORMAL -> normal ?: default
            KeyVariation.PASSWORD -> password ?: default
            KeyVariation.URI -> uri ?: default
        }?.compute(evaluator)
    }

    override fun asString(isForDisplay: Boolean): String {
        return ""
    }
}

/**
 * Allows to select an [AbstractKeyData] based on the current layout direction. Note that this type of selector only
 * really makes sense in a text context, though technically speaking it can be used anywhere, so this implementation
 * allows for any [AbstractKeyData] to be used here. The JSON class identifier for this selector is
 * `layout_direction_selector`.
 *
 * Example usage in a layout JSON file:
 * ```
 * { "$": "layout_direction_selector",
 *   "ltr": { "code":   59, "label": ";" },
 *   "rtl": { "code":   58, "label": ":" }
 * }
 * ```
 *
 * @property ltr The key data to use if the current layout direction is LTR.
 * @property rtl The key data to use if the current layout direction is RTL.
 */
class ReferenceLayoutDirectionSelector(
    val ltr: AbstractKeyData,
    val rtl: AbstractKeyData,
) : AbstractKeyData {
    override fun compute(evaluator: ComputingEvaluator): KeyData? {
        val isRtl = evaluator.state.layoutDirection == LayoutDirection.Rtl
        return (if (isRtl) { rtl } else { ltr }).compute(evaluator)
    }

    override fun asString(isForDisplay: Boolean): String {
        return ""
    }
}

/**
 * Allows to select an [AbstractKeyData] based on the character's width. Note that this type of selector only really
 * makes sense in a text context, though technically speaking it can be used anywhere, so this implementation allows
 * for any [AbstractKeyData] to be used here. The JSON class identifier for this selector is `char_width_selector`.
 *
 * Example usage in a layout JSON file:
 * ```
 * { "$": "char_width_selector",
 *   "full": { "code": 12450, "label": "ア" },
 *   "half": { "code": 65393, "label": "ｱ" }
 * }
 * ```
 *
 * @property full The key data to use if the current character width is full.
 * @property half The key data to use if the current character width is half.
 */
class ReferenceCharWidthSelector(
    val full: AbstractKeyData?,
    val half: AbstractKeyData?,
) : AbstractKeyData {
    override fun compute(evaluator: ComputingEvaluator): KeyData? {
        val data = if (evaluator.state.isCharHalfWidth) { half } else { full }
        return data?.compute(evaluator)
    }

    override fun asString(isForDisplay: Boolean): String {
        return ""
    }
}
