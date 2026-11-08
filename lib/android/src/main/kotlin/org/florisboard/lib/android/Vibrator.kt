/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

package org.florisboard.lib.android

import android.content.Context
import android.os.VibrationAttributes
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.util.Log

fun Context.systemVibratorOrNull(): Vibrator? {
    return this.systemServiceOrNull(VibratorManager::class)?.defaultVibrator?.takeIf { it.hasVibrator() }
}

/**
 * Vibrates once. A [touchFeedback] vibration is tagged as touch feedback, which the system drops
 * while the user has touch feedback turned off; otherwise it plays regardless.
 */
fun Vibrator.vibrate(duration: Int, strength: Int, factor: Double = 1.0, touchFeedback: Boolean = false) {
    if (duration == 0 || strength == 0) return
    val effectiveDuration = (duration * factor).toLong().coerceAtLeast(1L)
    val effectiveStrength = when {
        this.hasAmplitudeControl() -> (255.0 * ((strength * factor) / 100.0)).toInt().coerceIn(1, 255)
        else -> VibrationEffect.DEFAULT_AMPLITUDE
    }
    Log.d("Vibrator", "Perform haptic with duration=$effectiveDuration and strength=$effectiveStrength")
    val effect = VibrationEffect.createOneShot(effectiveDuration, effectiveStrength)
    if (touchFeedback) {
        this.vibrate(effect, VibrationAttributes.createForUsage(VibrationAttributes.USAGE_TOUCH))
    } else {
        this.vibrate(effect)
    }
}
