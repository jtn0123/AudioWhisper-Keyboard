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

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import com.jtn0123.audiowhisper.dictation.DictationFields
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.ime.core.DisplayLanguageNamesIn
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.editor.FlorisEditorInfo
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.UtilityKeyAction
import dev.patrickgold.florisboard.subtypeManager
import org.florisboard.lib.android.AndroidKeyguardManager
import org.florisboard.lib.android.systemService

internal class ComputingEvaluatorImpl(
    private val manager: KeyboardManager,
    override val version: Int,
    override val keyboard: Keyboard,
    override val editorInfo: FlorisEditorInfo,
    override val state: KeyboardState,
    override val subtype: Subtype,
) : ComputingEvaluator {
    private val appContext get() = manager.appContext
    private val prefs get() = manager.prefs
    private val clipboardManager get() = manager.clipboardManager
    private val subtypeManager get() = manager.subtypeManager
    private fun shouldShowLanguageSwitch() = manager.shouldShowLanguageSwitch()

    override fun context(): Context = appContext

    val androidKeyguardManager = context().systemService(AndroidKeyguardManager::class)

    override fun displayLanguageNamesIn(): DisplayLanguageNamesIn {
        return prefs.localization.displayLanguageNamesIn.get()
    }

    override fun evaluateEnabled(data: KeyData): Boolean {
        return when (data.code) {
            KeyCode.CLIPBOARD_COPY,
            KeyCode.CLIPBOARD_CUT -> {
                state.isSelectionMode && editorInfo.isRichInputEditor
            }
            KeyCode.CLIPBOARD_PASTE -> {
                !androidKeyguardManager.let { it.isDeviceLocked || it.isKeyguardLocked }
                    && clipboardManager.canBePasted(clipboardManager.primaryClip)
            }
            KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP -> {
                clipboardManager.canBePasted(clipboardManager.primaryClip)
            }
            KeyCode.CLIPBOARD_SELECT_ALL -> {
                editorInfo.isRichInputEditor
            }
            KeyCode.TOGGLE_INCOGNITO_MODE -> when (prefs.suggestion.incognitoMode.get()) {
                IncognitoMode.FORCE_OFF, IncognitoMode.FORCE_ON -> false
                IncognitoMode.DYNAMIC_ON_OFF -> !editorInfo.imeOptions.flagNoPersonalizedLearning
            }
            KeyCode.LANGUAGE_SWITCH -> {
                subtypeManager.subtypes.size > 1
            }
            // AudioWhisper: no dictating a password, PIN or number.
            KeyCode.VOICE_INPUT -> DictationFields.allows(editorInfo.inputAttributes)
            else -> true
        }
    }

    override fun evaluateVisible(data: KeyData): Boolean {
        return when (data.code) {
            KeyCode.IME_UI_MODE_TEXT,
            KeyCode.IME_UI_MODE_MEDIA -> {
                val tempUtilityKeyAction = when {
                    prefs.keyboard.utilityKeyEnabled.get() -> prefs.keyboard.utilityKeyAction.get()
                    else -> UtilityKeyAction.DISABLED
                }
                when (tempUtilityKeyAction) {
                    UtilityKeyAction.DISABLED,
                    UtilityKeyAction.SWITCH_LANGUAGE,
                    UtilityKeyAction.SWITCH_KEYBOARD_APP -> false
                    UtilityKeyAction.SWITCH_TO_EMOJIS -> true
                    UtilityKeyAction.DYNAMIC_SWITCH_LANGUAGE_EMOJIS -> !shouldShowLanguageSwitch()
                }
            }
            KeyCode.LANGUAGE_SWITCH -> {
                val tempUtilityKeyAction = when {
                    prefs.keyboard.utilityKeyEnabled.get() -> prefs.keyboard.utilityKeyAction.get()
                    else -> UtilityKeyAction.DISABLED
                }
                when (tempUtilityKeyAction) {
                    UtilityKeyAction.DISABLED,
                    UtilityKeyAction.SWITCH_TO_EMOJIS -> false
                    UtilityKeyAction.SWITCH_LANGUAGE,
                    UtilityKeyAction.SWITCH_KEYBOARD_APP -> true
                    UtilityKeyAction.DYNAMIC_SWITCH_LANGUAGE_EMOJIS -> shouldShowLanguageSwitch()
                }
            }
            else -> true
        }
    }

    override fun isSlot(data: KeyData): Boolean {
        return CurrencySet.isCurrencySlot(data.code)
    }

    override fun slotData(data: KeyData): KeyData? {
        return subtypeManager.getCurrencySet(subtype).getSlot(data.code)
    }

    fun asSmartbarQuickActionsEvaluator(): ComputingEvaluatorImpl {
        return ComputingEvaluatorImpl(
            manager = manager,
            version = version,
            keyboard = SmartbarQuickActionsKeyboard,
            editorInfo = editorInfo,
            state = state,
            subtype = Subtype.DEFAULT,
        )
    }
}
