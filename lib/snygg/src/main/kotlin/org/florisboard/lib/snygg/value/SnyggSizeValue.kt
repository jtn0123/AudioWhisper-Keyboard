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

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

sealed interface SnyggSizeValue : SnyggValue

data class SnyggDpSizeValue(val dp: Dp) : SnyggSizeValue {
    companion object : SnyggNativeValueEncoder<SnyggDpSizeValue>(
        nativeId = 4, valueClass = SnyggDpSizeValue::class,
        construct = { SnyggDpSizeValue(it[0].float().dp) },
        destruct = { listOf(it.dp.value.bits()) },
    )

    override fun encoder() = Companion
}

data class SnyggSpSizeValue(val sp: TextUnit) : SnyggSizeValue {
    companion object : SnyggNativeValueEncoder<SnyggSpSizeValue>(
        nativeId = 5, valueClass = SnyggSpSizeValue::class,
        construct = { SnyggSpSizeValue(it[0].float().sp) },
        destruct = { listOf(it.sp.value.bits()) },
    )

    override fun encoder() = Companion
}

data class SnyggPercentageSizeValue(val percentage: Float) : SnyggSizeValue {
    companion object : SnyggNativeValueEncoder<SnyggPercentageSizeValue>(
        nativeId = 6, valueClass = SnyggPercentageSizeValue::class,
        construct = { SnyggPercentageSizeValue(it[0].float()) },
        destruct = { listOf(it.percentage.bits()) },
    )

    override fun encoder() = Companion
}
