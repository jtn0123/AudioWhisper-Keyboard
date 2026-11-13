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

import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.CutCornerShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

private const val CornerSizeTopStart = "cornerSizeTopStart"
private const val CornerSizeTopEnd = "cornerSizeTopEnd"
private const val CornerSizeBottomEnd = "cornerSizeBottomEnd"
private const val CornerSizeBottomStart = "cornerSizeBottomStart"

private const val Rectangle = "rectangle"
private const val Circle = "circle"
private const val CutCorner = "cut-corner"
private const val RoundedCorner = "rounded-corner"

private const val DpUnit = "dp"

sealed interface ReferenceSnyggShapeValue : ReferenceSnyggValue {
    val shape: Shape
}

sealed interface ReferenceSnyggDpShapeValue : ReferenceSnyggShapeValue {
    override val shape: Shape
    val topStart: Dp
    val topEnd: Dp
    val bottomEnd: Dp
    val bottomStart: Dp
}

sealed interface ReferenceSnyggPercentShapeValue : ReferenceSnyggShapeValue {
    override val shape: Shape
    val topStart: Int
    val topEnd: Int
    val bottomEnd: Int
    val bottomStart: Int
}

data class ReferenceSnyggRectangleShapeValue(override val shape: Shape = RectangleShape) : ReferenceSnyggShapeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            function(Rectangle) { nothing() }
        }

        override fun defaultValue() = ReferenceSnyggRectangleShapeValue()

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggRectangleShapeValue)
            val map = referenceSnyggIdToValueMapOf()
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            return@runCatching ReferenceSnyggRectangleShapeValue()
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggCircleShapeValue(override val shape: Shape = CircleShape) : ReferenceSnyggShapeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            function(Circle) { nothing() }
        }

        override fun defaultValue() = ReferenceSnyggCircleShapeValue()

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggCircleShapeValue)
            val map = referenceSnyggIdToValueMapOf()
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            return@runCatching ReferenceSnyggCircleShapeValue()
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggCutCornerDpShapeValue(
    override val topStart: Dp,
    override val topEnd: Dp,
    override val bottomEnd: Dp,
    override val bottomStart: Dp,
    override val shape: CutCornerShape = CutCornerShape(topStart, topEnd, bottomEnd, bottomStart),
) : ReferenceSnyggDpShapeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            function(CutCorner) {
                commaList {
                    +float(id = CornerSizeTopStart, unit = DpUnit)
                    +float(id = CornerSizeTopEnd, unit = DpUnit)
                    +float(id = CornerSizeBottomEnd, unit = DpUnit)
                    +float(id = CornerSizeBottomStart, unit = DpUnit)
                }
            }
        }

        override fun defaultValue() = ReferenceSnyggCutCornerDpShapeValue(0.dp, 0.dp, 0.dp, 0.dp)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggCutCornerDpShapeValue)
            val map = referenceSnyggIdToValueMapOf(
                CornerSizeTopStart to v.topStart.value,
                CornerSizeTopEnd to v.topEnd.value,
                CornerSizeBottomEnd to v.bottomEnd.value,
                CornerSizeBottomStart to v.bottomStart.value,
            )
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val topStart = map.getInt(CornerSizeTopStart).dp
            val topEnd = map.getInt(CornerSizeTopEnd).dp
            val bottomEnd = map.getInt(CornerSizeBottomEnd).dp
            val bottomStart = map.getInt(CornerSizeBottomStart).dp
            return@runCatching ReferenceSnyggCutCornerDpShapeValue(topStart, topEnd, bottomEnd, bottomStart)
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggCutCornerPercentShapeValue(
    override val topStart: Int,
    override val topEnd: Int,
    override val bottomEnd: Int,
    override val bottomStart: Int,
    override val shape: CutCornerShape = CutCornerShape(topStart, topEnd, bottomEnd, bottomStart),
) : ReferenceSnyggPercentShapeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            function(CutCorner) {
                commaList {
                    +percentageInt(id = CornerSizeTopStart)
                    +percentageInt(id = CornerSizeTopEnd)
                    +percentageInt(id = CornerSizeBottomEnd)
                    +percentageInt(id = CornerSizeBottomStart)
                }
            }
        }

        override fun defaultValue() = ReferenceSnyggCutCornerPercentShapeValue(0, 0, 0, 0)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggCutCornerPercentShapeValue)
            val map = referenceSnyggIdToValueMapOf(
                CornerSizeTopStart to v.topStart,
                CornerSizeTopEnd to v.topEnd,
                CornerSizeBottomEnd to v.bottomEnd,
                CornerSizeBottomStart to v.bottomStart,
            )
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val topStart = map.getInt(CornerSizeTopStart)
            val topEnd = map.getInt(CornerSizeTopEnd)
            val bottomEnd = map.getInt(CornerSizeBottomEnd)
            val bottomStart = map.getInt(CornerSizeBottomStart)
            return@runCatching ReferenceSnyggCutCornerPercentShapeValue(topStart, topEnd, bottomEnd, bottomStart)
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggRoundedCornerDpShapeValue(
    override val topStart: Dp,
    override val topEnd: Dp,
    override val bottomEnd: Dp,
    override val bottomStart: Dp,
    override val shape: RoundedCornerShape = RoundedCornerShape(topStart, topEnd, bottomEnd, bottomStart),
) : ReferenceSnyggDpShapeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            function(RoundedCorner) {
                commaList {
                    +float(id = CornerSizeTopStart, unit = DpUnit)
                    +float(id = CornerSizeTopEnd, unit = DpUnit)
                    +float(id = CornerSizeBottomEnd, unit = DpUnit)
                    +float(id = CornerSizeBottomStart, unit = DpUnit)
                }
            }
        }

        override fun defaultValue() = ReferenceSnyggRoundedCornerDpShapeValue(0.dp, 0.dp, 0.dp, 0.dp)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggRoundedCornerDpShapeValue)
            val map = referenceSnyggIdToValueMapOf(
                CornerSizeTopStart to v.topStart.value,
                CornerSizeTopEnd to v.topEnd.value,
                CornerSizeBottomEnd to v.bottomEnd.value,
                CornerSizeBottomStart to v.bottomStart.value,
            )
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val topStart = map.getFloat(CornerSizeTopStart).dp
            val topEnd = map.getFloat(CornerSizeTopEnd).dp
            val bottomEnd = map.getFloat(CornerSizeBottomEnd).dp
            val bottomStart = map.getFloat(CornerSizeBottomStart).dp
            return@runCatching ReferenceSnyggRoundedCornerDpShapeValue(topStart, topEnd, bottomEnd, bottomStart)
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggRoundedCornerPercentShapeValue(
    override val topStart: Int,
    override val topEnd: Int,
    override val bottomEnd: Int,
    override val bottomStart: Int,
    override val shape: RoundedCornerShape = RoundedCornerShape(topStart, topEnd, bottomEnd, bottomStart),
) : ReferenceSnyggPercentShapeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            function(RoundedCorner) {
                commaList {
                    +percentageInt(id = CornerSizeTopStart)
                    +percentageInt(id = CornerSizeTopEnd)
                    +percentageInt(id = CornerSizeBottomEnd)
                    +percentageInt(id = CornerSizeBottomStart)
                }
            }
        }

        override fun defaultValue() = ReferenceSnyggRoundedCornerPercentShapeValue(0, 0, 0, 0)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggRoundedCornerPercentShapeValue)
            val map = referenceSnyggIdToValueMapOf(
                CornerSizeTopStart to v.topStart,
                CornerSizeTopEnd to v.topEnd,
                CornerSizeBottomEnd to v.bottomEnd,
                CornerSizeBottomStart to v.bottomStart,
            )
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val topStart = map.getInt(CornerSizeTopStart)
            val topEnd = map.getInt(CornerSizeTopEnd)
            val bottomEnd = map.getInt(CornerSizeBottomEnd)
            val bottomStart = map.getInt(CornerSizeBottomStart)
            return@runCatching ReferenceSnyggRoundedCornerPercentShapeValue(topStart, topEnd, bottomEnd, bottomStart)
        }
    }

    override fun encoder() = Companion
}
