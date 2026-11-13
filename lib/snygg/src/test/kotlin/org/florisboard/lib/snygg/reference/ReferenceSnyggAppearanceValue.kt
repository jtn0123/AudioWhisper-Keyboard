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

package org.florisboard.lib.snygg.reference

import androidx.compose.ui.graphics.Color
import org.florisboard.lib.color.ColorPalette
import kotlin.math.roundToInt

sealed interface ReferenceSnyggAppearanceValue : ReferenceSnyggValue

object RgbaColor {
    const val HexId = "hex"
    val Hex6Matcher = """#[a-fA-F0-9]{6}""".toRegex()
    val Hex8Matcher = """#[a-fA-F0-9]{8}""".toRegex()

    const val TransparentId = "transparent"
    val TransparentMatcher = """transparent""".toRegex()

    const val RedId = "r"
    const val GreenId = "g"
    const val BlueId = "b"
    const val AlphaId = "a"

    const val ColorRangeMin = 0
    const val ColorRangeMax = 255
    val ColorRange = ColorRangeMin..ColorRangeMax
    val ColorRangePattern = """25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9]""".toRegex()

    const val AlphaRangeMin = 0f
    const val AlphaRangeMax = 1f
    val AlphaRange = AlphaRangeMin..AlphaRangeMax
    val AlphaRangePattern = """1(?:[.]0)?|0(?:[.][0-9]*)?|[.][0-9]+""".toRegex()
}

data class ReferenceSnyggStaticColorValue(val color: Color) : ReferenceSnyggAppearanceValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            function(name = "rgba") {
                commaList {
                    +int(id = RgbaColor.RedId, numberPattern = RgbaColor.ColorRangePattern)
                    +int(id = RgbaColor.GreenId, numberPattern = RgbaColor.ColorRangePattern)
                    +int(id = RgbaColor.BlueId, numberPattern = RgbaColor.ColorRangePattern)
                    +float(id = RgbaColor.AlphaId, numberPattern = RgbaColor.AlphaRangePattern)
                }
            }
        }

        override val alternativeSpecs = listOf(
            ReferenceSnyggValueSpec {
                function(name = "rgb") {
                    commaList {
                        +int(id = RgbaColor.RedId, numberPattern = RgbaColor.ColorRangePattern)
                        +int(id = RgbaColor.GreenId, numberPattern = RgbaColor.ColorRangePattern)
                        +int(id = RgbaColor.BlueId, numberPattern = RgbaColor.ColorRangePattern)
                    }
                }
            },
            ReferenceSnyggValueSpec {
                string(id = RgbaColor.TransparentId, regex = RgbaColor.TransparentMatcher)
            },
            ReferenceSnyggValueSpec {
                string(id = RgbaColor.HexId, regex = RgbaColor.Hex6Matcher)
            },
            ReferenceSnyggValueSpec {
                string(id = RgbaColor.HexId, regex = RgbaColor.Hex8Matcher)
            },
        )

        override fun defaultValue() = ReferenceSnyggStaticColorValue(Color.Black)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggStaticColorValue)
            val map = referenceSnyggIdToValueMapOf(
                RgbaColor.RedId to (v.color.red * RgbaColor.ColorRangeMax).roundToInt(),
                RgbaColor.GreenId to (v.color.green * RgbaColor.ColorRangeMax).roundToInt(),
                RgbaColor.BlueId to (v.color.blue * RgbaColor.ColorRangeMax).roundToInt(),
                RgbaColor.AlphaId to v.color.alpha,
            )
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            runCatching { spec.parse(v, map) }.onSuccess {
                val r = map.getInt(RgbaColor.RedId).toFloat() / RgbaColor.ColorRangeMax
                val g = map.getInt(RgbaColor.GreenId).toFloat() / RgbaColor.ColorRangeMax
                val b = map.getInt(RgbaColor.BlueId).toFloat() / RgbaColor.ColorRangeMax
                val a = map.getFloat(RgbaColor.AlphaId)
                return@runCatching ReferenceSnyggStaticColorValue(Color(r, g, b, a))
            }
            runCatching { alternativeSpecs[0].parse(v, map) }.onSuccess {
                val r = map.getInt(RgbaColor.RedId).toFloat() / RgbaColor.ColorRangeMax
                val g = map.getInt(RgbaColor.GreenId).toFloat() / RgbaColor.ColorRangeMax
                val b = map.getInt(RgbaColor.BlueId).toFloat() / RgbaColor.ColorRangeMax
                return@runCatching ReferenceSnyggStaticColorValue(Color(r, g, b, 1.0f))
            }
            runCatching { alternativeSpecs[1].parse(v, map) }.onSuccess {
                return@runCatching ReferenceSnyggStaticColorValue(Color(0.0f, 0.0f, 0.0f, 0.0f))
            }
            runCatching { alternativeSpecs[2].parse(v, map) }.onSuccess {
                val hexStr = map.getString(RgbaColor.HexId)
                val r = hexStr.substring(1..2).toInt(16).toFloat() / RgbaColor.ColorRangeMax
                val g = hexStr.substring(3..4).toInt(16).toFloat() / RgbaColor.ColorRangeMax
                val b = hexStr.substring(5..6).toInt(16).toFloat() / RgbaColor.ColorRangeMax
                return@runCatching ReferenceSnyggStaticColorValue(Color(r, g, b, 1.0f))
            }
            runCatching { alternativeSpecs[3].parse(v, map) }.onSuccess {
                val hexStr = map.getString(RgbaColor.HexId)
                val r = hexStr.substring(1..2).toInt(16).toFloat() / RgbaColor.ColorRangeMax
                val g = hexStr.substring(3..4).toInt(16).toFloat() / RgbaColor.ColorRangeMax
                val b = hexStr.substring(5..6).toInt(16).toFloat() / RgbaColor.ColorRangeMax
                val a = hexStr.substring(7..8).toInt(16).toFloat() / 0xFF
                return@runCatching ReferenceSnyggStaticColorValue(Color(r, g, b, a))
            }
            error("No matching spec found for \"$v\"")
        }
    }

    override fun encoder() = Companion
}

/// Dynamic Color Value

private const val ColorNameId = "name"
private val ColorName = ColorPalette.colorNames.joinToString("|").toRegex()

sealed interface ReferenceSnyggDynamicColorValue : ReferenceSnyggAppearanceValue {
    val colorName: String
}

data class ReferenceSnyggDynamicLightColorValue(override val colorName: String) : ReferenceSnyggDynamicColorValue {
    companion object : ReferenceSnyggValueEncoder {
        private const val FunctionName = "dynamic-light-color"

        override val spec = ReferenceSnyggValueSpec {
            function(name = FunctionName) {
                commaList {
                    +string(id = ColorNameId, regex = ColorName)
                }
            }
        }

        override fun defaultValue() = ReferenceSnyggDynamicLightColorValue(ColorPalette.Primary.id)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggDynamicLightColorValue)
            val map = referenceSnyggIdToValueMapOf(ColorNameId to v.colorName)
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val colorName = map.getString(ColorNameId)
            return@runCatching ReferenceSnyggDynamicLightColorValue(colorName)
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggDynamicDarkColorValue(override val colorName: String) : ReferenceSnyggDynamicColorValue {
    companion object : ReferenceSnyggValueEncoder {
        private const val FunctionName = "dynamic-dark-color"

        override val spec = ReferenceSnyggValueSpec {
            function(name = FunctionName) {
                commaList {
                    +string(id = ColorNameId, regex = ColorName)
                }
            }
        }

        override fun defaultValue() = ReferenceSnyggDynamicDarkColorValue(ColorPalette.Primary.id)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggDynamicDarkColorValue)
            val map = referenceSnyggIdToValueMapOf(ColorNameId to v.colorName)
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val colorName = map.getString(ColorNameId)
            return@runCatching ReferenceSnyggDynamicDarkColorValue(colorName)
        }
    }

    override fun encoder() = Companion
}

//data class LinearGradient(
//    val dummy: Int,
//) : ReferenceSnyggAppearanceValue()
//
//data class RadialGradient(
//    val dummy: Int,
//) : ReferenceSnyggAppearanceValue()
//
