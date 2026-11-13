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

import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.CutCornerShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

sealed interface SnyggShapeValue : SnyggValue {
    val shape: Shape
}

sealed interface SnyggDpShapeValue : SnyggShapeValue {
    override val shape: Shape
    val topStart: Dp
    val topEnd: Dp
    val bottomEnd: Dp
    val bottomStart: Dp
}

sealed interface SnyggPercentShapeValue : SnyggShapeValue {
    override val shape: Shape
    val topStart: Int
    val topEnd: Int
    val bottomEnd: Int
    val bottomStart: Int
}

data class SnyggRectangleShapeValue(override val shape: Shape = RectangleShape) : SnyggShapeValue {
    companion object : SnyggNativeValueEncoder<SnyggRectangleShapeValue>(
        nativeId = 15, valueClass = SnyggRectangleShapeValue::class,
        construct = { SnyggRectangleShapeValue() },
        destruct = { List(4) { 0f.bits() } },
    )

    override fun encoder() = Companion
}

data class SnyggCircleShapeValue(override val shape: Shape = CircleShape) : SnyggShapeValue {
    companion object : SnyggNativeValueEncoder<SnyggCircleShapeValue>(
        nativeId = 16, valueClass = SnyggCircleShapeValue::class,
        construct = { SnyggCircleShapeValue() },
        destruct = { List(4) { 0f.bits() } },
    )

    override fun encoder() = Companion
}

data class SnyggCutCornerDpShapeValue(
    override val topStart: Dp,
    override val topEnd: Dp,
    override val bottomEnd: Dp,
    override val bottomStart: Dp,
    override val shape: CutCornerShape = CutCornerShape(topStart, topEnd, bottomEnd, bottomStart),
) : SnyggDpShapeValue {
    companion object : SnyggNativeValueEncoder<SnyggCutCornerDpShapeValue>(
        nativeId = 17, valueClass = SnyggCutCornerDpShapeValue::class,
        construct = { SnyggCutCornerDpShapeValue(it[0].float().dp, it[1].float().dp, it[2].float().dp, it[3].float().dp) },
        destruct = { listOf(it.topStart.value.bits(), it.topEnd.value.bits(), it.bottomEnd.value.bits(), it.bottomStart.value.bits()) },
    )

    override fun encoder() = Companion
}

data class SnyggCutCornerPercentShapeValue(
    override val topStart: Int,
    override val topEnd: Int,
    override val bottomEnd: Int,
    override val bottomStart: Int,
    override val shape: CutCornerShape = CutCornerShape(topStart, topEnd, bottomEnd, bottomStart),
) : SnyggPercentShapeValue {
    companion object : SnyggNativeValueEncoder<SnyggCutCornerPercentShapeValue>(
        nativeId = 18, valueClass = SnyggCutCornerPercentShapeValue::class,
        construct = { SnyggCutCornerPercentShapeValue(it[0].toInt(), it[1].toInt(), it[2].toInt(), it[3].toInt()) },
        destruct = { listOf(it.topStart.toString(), it.topEnd.toString(), it.bottomEnd.toString(), it.bottomStart.toString()) },
    )

    override fun encoder() = Companion
}

data class SnyggRoundedCornerDpShapeValue(
    override val topStart: Dp,
    override val topEnd: Dp,
    override val bottomEnd: Dp,
    override val bottomStart: Dp,
    override val shape: RoundedCornerShape = RoundedCornerShape(topStart, topEnd, bottomEnd, bottomStart),
) : SnyggDpShapeValue {
    companion object : SnyggNativeValueEncoder<SnyggRoundedCornerDpShapeValue>(
        nativeId = 19, valueClass = SnyggRoundedCornerDpShapeValue::class,
        construct = { SnyggRoundedCornerDpShapeValue(it[0].float().dp, it[1].float().dp, it[2].float().dp, it[3].float().dp) },
        destruct = { listOf(it.topStart.value.bits(), it.topEnd.value.bits(), it.bottomEnd.value.bits(), it.bottomStart.value.bits()) },
    )

    override fun encoder() = Companion
}

data class SnyggRoundedCornerPercentShapeValue(
    override val topStart: Int,
    override val topEnd: Int,
    override val bottomEnd: Int,
    override val bottomStart: Int,
    override val shape: RoundedCornerShape = RoundedCornerShape(topStart, topEnd, bottomEnd, bottomStart),
) : SnyggPercentShapeValue {
    companion object : SnyggNativeValueEncoder<SnyggRoundedCornerPercentShapeValue>(
        nativeId = 20, valueClass = SnyggRoundedCornerPercentShapeValue::class,
        construct = { SnyggRoundedCornerPercentShapeValue(it[0].toInt(), it[1].toInt(), it[2].toInt(), it[3].toInt()) },
        destruct = { listOf(it.topStart.toString(), it.topEnd.toString(), it.bottomEnd.toString(), it.bottomStart.toString()) },
    )

    override fun encoder() = Companion
}
