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

import com.jtn0123.audiowhisper.rust.KeyGeometry
import com.jtn0123.audiowhisper.rust.KeyModels
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.AbstractKeyData
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.computeImageVector
import dev.patrickgold.florisboard.ime.keyboard.computeLabel
import dev.patrickgold.florisboard.ime.popup.MutablePopupSet
import dev.patrickgold.florisboard.ime.popup.PopupMapping
import dev.patrickgold.florisboard.ime.popup.PopupSet
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.lib.lowercase

class TextKey(override val data: AbstractKeyData) : Key(data) {
    var computedData: KeyData = TextKeyData.UNSPECIFIED
        private set
    val computedPopups: MutablePopupSet<KeyData> = MutablePopupSet()
    var computedSymbolHint: KeyData? = null
    var computedNumberHint: KeyData? = null
    var computedHintData: KeyData = TextKeyData.UNSPECIFIED

    // This should exclusively be set and used by the TextKeyboardLayout
    var computedDataOnDown: KeyData = TextKeyData.UNSPECIFIED

    fun compute(evaluator: ComputingEvaluator) {
        val keyboard = evaluator.keyboard as? TextKeyboard ?: return
        val keyboardMode = keyboard.mode
        val computed = data.compute(evaluator)

        if (computed == null || !evaluator.evaluateVisible(computed)) {
            computedData = TextKeyData.UNSPECIFIED
            computedPopups.clear()
            isEnabled = false
            isVisible = false

            flayShrink = 0.0f
            flayGrow = 0.0f
            flayWidthFactor = 0.0f
        } else {
            computedData = computed
            computedPopups.clear()
            mergePopups(computed, evaluator, computedPopups::merge)
            if (KeyModels.popupMode(keyboardMode.value)) {
                val computedLabel = computed.label.lowercase(evaluator.subtype.primaryLocale)
                val extLabel = KeyModels.popupGroup(computed.groupId) ?: computedLabel
                val extendedPopupsDefault = keyboard.extendedPopupMappingDefault
                val extendedPopups = keyboard.extendedPopupMapping
                val sources = KeyVariation.entries.flatMap { variation ->
                    listOf(extendedPopups?.get(variation)?.get(extLabel), extendedPopupsDefault?.get(variation)?.get(extLabel))
                }
                val present = sources.indices.fold(0) { bits, i -> if (sources[i] != null) bits or (1 shl i) else bits }
                val popupSet = sources.getOrNull(KeyModels.popupSource(evaluator.state.keyVariation.value, present))
                var keySpecificPopupSet: PopupSet<AbstractKeyData>? = null
                if (extLabel != computedLabel) {
                    keySpecificPopupSet = extendedPopups?.get(KeyVariation.ALL)?.get(computedLabel) ?:
                        extendedPopupsDefault?.get(KeyVariation.ALL)?.get(computedLabel)
                }
                computedPopups.apply {
                    keySpecificPopupSet?.let { merge(it, evaluator) }
                    popupSet?.let { merge(it, evaluator) }
                }
                if (computed.type == KeyType.CHARACTER) {
                    addComputedHints(computed.code, evaluator, extendedPopups, extendedPopupsDefault)
                }
            }
            isEnabled = evaluator.evaluateEnabled(computed)
            isVisible = true

            applyFlex(keyboardMode, computed.code, computed.type)
        }
    }

    /**
     * Sets the flex factors of this visible key, with [code] and [type] on a keyboard of
     * [keyboardMode], from the table in Rust (lib/rust/crates/keys).
     */
    internal fun applyFlex(keyboardMode: KeyboardMode, code: Int, type: KeyType) {
        val flex = KeyGeometry.flex(keyboardMode.value, code, type == KeyType.NUMERIC)
        flayShrink = flex.shrink
        flayGrow = flex.grow
        flayWidthFactor = flex.widthFactor
    }

    inline fun setPressed(state: Boolean, blockIfChanged: () -> Unit) {
        if (isPressed != state) {
            isPressed = state
            blockIfChanged()
        }
    }

    private fun addComputedHints(
        keyCode: Int,
        evaluator: ComputingEvaluator,
        extendedPopups: PopupMapping?,
        extendedPopupsDefault: PopupMapping?
    ) {
        val symbolHint = computedSymbolHint
        if (symbolHint != null) {
            val evaluatedSymbolHint = symbolHint.compute(evaluator)
            if (symbolHint.code != keyCode) {
                computedPopups.symbolHint = evaluatedSymbolHint
                mergePopups(evaluatedSymbolHint, evaluator, computedPopups::mergeSymbolHint)
                val hintSpecificPopupSet =
                    extendedPopups?.get(KeyVariation.ALL)?.get(symbolHint.label) ?: extendedPopupsDefault?.get(
                        KeyVariation.ALL
                    )?.get(symbolHint.label)
                hintSpecificPopupSet?.let { computedPopups.mergeSymbolHint(it, evaluator) }
            }
        }
        val numericHint = computedNumberHint
        if (numericHint != null) {
            val evaluatedNumberHint = numericHint.compute(evaluator)
            if (numericHint.code != keyCode) {
                computedPopups.numberHint = evaluatedNumberHint
                mergePopups(evaluatedNumberHint, evaluator, computedPopups::mergeNumberHint)
                val hintSpecificPopupSet =
                    extendedPopups?.get(KeyVariation.ALL)?.get(numericHint.label) ?: extendedPopupsDefault?.get(
                        KeyVariation.ALL
                    )?.get(numericHint.label)
                hintSpecificPopupSet?.let { computedPopups.mergeNumberHint(it, evaluator) }
            }
        }
    }

    private fun mergePopups(
        keyData: KeyData?,
        evaluator: ComputingEvaluator,
        merge: (popups: PopupSet<AbstractKeyData>, evaluator: ComputingEvaluator) -> Unit,
    ) {
        if (keyData?.popup != null) {
            merge(keyData.popup!!, evaluator)
        }
    }

    /**
     * Computes the label, hintedLabel and iconResId for [computedData] based on given [evaluator].
     */
    fun computeLabelsAndDrawables(evaluator: ComputingEvaluator) {
        label = evaluator.computeLabel(computedData)
        hintedLabel = null
        foregroundImageVector = evaluator.computeImageVector(computedData)

        val data = computedData
        if (data.type == KeyType.NUMERIC && evaluator.keyboard.mode == KeyboardMode.PHONE) {
            hintedLabel = KeyModels.phoneHint(data.code)
        } else if (!data.isSpaceKey() || data.type == KeyType.NUMERIC) {
            val prefs by FlorisPreferenceStore
            computedPopups.getPopupKeys(prefs.keyboard.keyHintConfiguration()).hint.let { hintData ->
                if (hintData?.isSpaceKey() == false) {
                    hintedLabel = hintData.asString(isForDisplay = true)
                    computedHintData = hintData
                } else {
                    hintedLabel = null
                    computedHintData = TextKeyData.UNSPECIFIED
                }
            }
        }
    }

    override fun toString(): String {
        return computedData.toString()
    }
}
