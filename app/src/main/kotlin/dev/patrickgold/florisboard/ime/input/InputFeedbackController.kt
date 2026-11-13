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

package dev.patrickgold.florisboard.ime.input

import android.inputmethodservice.InputMethodService
import android.media.AudioManager
import android.provider.Settings
import android.view.HapticFeedbackConstants
import androidx.compose.runtime.staticCompositionLocalOf
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import org.florisboard.lib.android.systemServiceOrNull
import org.florisboard.lib.android.systemVibratorOrNull
import org.florisboard.lib.android.vibrate
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

val LocalInputFeedbackController = staticCompositionLocalOf<InputFeedbackController> { error("not init") }

/**
 * Input feedback controller is responsible to process and perform audio and haptic
 * feedback for user interactions based on the system and floris preferences.
 */
class InputFeedbackController private constructor(private val ims: InputMethodService) {
    companion object {
        fun new(ims: InputMethodService) = InputFeedbackController(ims)
    }

    private val prefs by FlorisPreferenceStore

    private val audioManager = ims.systemServiceOrNull(AudioManager::class)
    private val vibrator = ims.systemVibratorOrNull()
    private val contentResolver = ims.contentResolver
    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    private var systemAudioEnabled: Boolean = false

    /**
     * Android no longer exposes the touch-feedback setting to apps; haptics instead ask the system
     * to apply it (see [performHapticFeedback]).
     */
    fun updateSystemPrefsState() {
        systemAudioEnabled = systemPref(Settings.System.SOUND_EFFECTS_ENABLED)
    }

    fun keyPress(data: KeyData = TextKeyData.UNSPECIFIED) = feedback(0, data)
    fun keyLongPress(data: KeyData = TextKeyData.UNSPECIFIED) = feedback(1, data)
    fun keyRepeatedAction(data: KeyData = TextKeyData.UNSPECIFIED) = feedback(2, data)
    fun gestureSwipe(data: KeyData = TextKeyData.UNSPECIFIED) = feedback(3, data)
    fun gestureMovingSwipe(data: KeyData = TextKeyData.UNSPECIFIED) = feedback(4, data)

    private fun feedback(event: Int, data: KeyData) {
        val audio = listOf(prefs.inputFeedback.audioFeatKeyPress, prefs.inputFeedback.audioFeatKeyLongPress,
            prefs.inputFeedback.audioFeatKeyRepeatedAction, prefs.inputFeedback.audioFeatGestureSwipe,
            prefs.inputFeedback.audioFeatGestureMovingSwipe)[event].get()
        val haptic = listOf(prefs.inputFeedback.hapticFeatKeyPress, prefs.inputFeedback.hapticFeatKeyLongPress,
            prefs.inputFeedback.hapticFeatKeyRepeatedAction, prefs.inputFeedback.hapticFeatGestureSwipe,
            prefs.inputFeedback.hapticFeatGestureMovingSwipe)[event].get()
        fun bit(value: Boolean) = if (value) 1L else 0L
        val plan = com.jtn0123.audiowhisper.rust.InputRules.feedback(longArrayOf(event.toLong(), data.code.toLong(),
            bit(audio), bit(haptic), bit(prefs.inputFeedback.audioEnabled.get()), bit(prefs.inputFeedback.hapticEnabled.get()),
            bit(prefs.inputFeedback.audioActivationMode.get() == InputFeedbackActivationMode.RESPECT_SYSTEM_SETTINGS),
            bit(systemAudioEnabled), prefs.inputFeedback.audioVolume.get().toLong(),
            bit(prefs.inputFeedback.hapticVibrationMode.get() == HapticVibrationMode.USE_HAPTIC_FEEDBACK_INTERFACE),
            bit(prefs.inputFeedback.hapticActivationMode.get() == InputFeedbackActivationMode.RESPECT_SYSTEM_SETTINGS),
            bit(vibrator != null)))
        scope.launch {
            if (plan[0] >= 0) audioManager?.playSoundEffect(plan[0].toInt(), Float.fromBits(plan[1].toInt()))
            if (plan[2] < 0) return@launch
            if (plan[2] > 0) {
                val view = ims.window?.window?.decorView ?: return@launch
                if (view.performHapticFeedback(plan[2].toInt(), HapticFeedbackConstants.FLAG_IGNORE_VIEW_SETTING)) return@launch
            }
            vibrator?.vibrate(duration = prefs.inputFeedback.hapticVibrationDuration.get(),
                strength = prefs.inputFeedback.hapticVibrationStrength.get(), factor = Double.fromBits(plan[3]),
                touchFeedback = plan[4] != 0L)
        }
    }

    private fun systemPref(id: String): Boolean {
        if (contentResolver == null) return false
        return Settings.System.getInt(contentResolver, id, 0) != 0
    }

}
