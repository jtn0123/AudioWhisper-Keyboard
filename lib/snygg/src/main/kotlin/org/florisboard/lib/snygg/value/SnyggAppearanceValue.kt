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

package org.florisboard.lib.snygg.value

import androidx.compose.ui.graphics.Color

sealed interface SnyggAppearanceValue : SnyggValue

object RgbaColor {
    const val HexId = "hex"
    val Hex6Matcher = NativeSnyggValues.patterns[0].toRegex()
    val Hex8Matcher = NativeSnyggValues.patterns[1].toRegex()

    const val TransparentId = "transparent"
    val TransparentMatcher = NativeSnyggValues.patterns[2].toRegex()

    const val RedId = "r"
    const val GreenId = "g"
    const val BlueId = "b"
    const val AlphaId = "a"

    const val ColorRangeMin = 0
    const val ColorRangeMax = 255
    val ColorRange = ColorRangeMin..ColorRangeMax
    val ColorRangePattern = NativeSnyggValues.patterns[3].toRegex()

    const val AlphaRangeMin = 0f
    const val AlphaRangeMax = 1f
    val AlphaRange = AlphaRangeMin..AlphaRangeMax
    val AlphaRangePattern = NativeSnyggValues.patterns[4].toRegex()
}

data class SnyggStaticColorValue(val color: Color) : SnyggAppearanceValue {
    companion object : SnyggNativeValueEncoder<SnyggStaticColorValue>(
        nativeId = 7, valueClass = SnyggStaticColorValue::class,
        construct = { SnyggStaticColorValue(Color(it[0].float(), it[1].float(), it[2].float(), it[3].float())) },
        destruct = { listOf(it.color.red.bits(), it.color.green.bits(), it.color.blue.bits(), it.color.alpha.bits()) },
    )

    override fun encoder() = Companion
}

/// Dynamic Color Value

sealed interface SnyggDynamicColorValue : SnyggAppearanceValue {
    val colorName: String
}

data class SnyggDynamicLightColorValue(override val colorName: String) : SnyggDynamicColorValue {
    companion object : SnyggNativeValueEncoder<SnyggDynamicLightColorValue>(
        nativeId = 8, valueClass = SnyggDynamicLightColorValue::class,
        construct = { SnyggDynamicLightColorValue(it[0]) },
        destruct = { listOf(it.colorName) },
    )

    override fun encoder() = Companion
}

data class SnyggDynamicDarkColorValue(override val colorName: String) : SnyggDynamicColorValue {
    companion object : SnyggNativeValueEncoder<SnyggDynamicDarkColorValue>(
        nativeId = 9, valueClass = SnyggDynamicDarkColorValue::class,
        construct = { SnyggDynamicDarkColorValue(it[0]) },
        destruct = { listOf(it.colorName) },
    )

    override fun encoder() = Companion
}

//data class LinearGradient(
//    val dummy: Int,
//) : SnyggAppearanceValue()
//
//data class RadialGradient(
//    val dummy: Int,
//) : SnyggAppearanceValue()
//
