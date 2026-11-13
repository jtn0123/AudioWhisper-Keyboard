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

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private const val Size = "size"
private const val DpUnit = "dp"
private const val SpUnit = "sp"

sealed interface ReferenceSnyggSizeValue : ReferenceSnyggValue

data class ReferenceSnyggDpSizeValue(val dp: Dp) : ReferenceSnyggSizeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            float(id = Size, unit = DpUnit, numberPattern = """(?:0|[1-9][0-9]*)(?:[.][0-9]*)?|[.][0-9]+""".toRegex())
        }

        override fun defaultValue() = ReferenceSnyggDpSizeValue(0.dp)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggDpSizeValue)
            val map = referenceSnyggIdToValueMapOf(Size to v.dp.value)
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val size = map.getFloat(Size)
            return@runCatching ReferenceSnyggDpSizeValue(size.dp)
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggSpSizeValue(val sp: TextUnit) : ReferenceSnyggSizeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            float(id = Size, unit = SpUnit, numberPattern = """[1-9][0-9]*(?:[.][0-9]*)?""".toRegex())
        }

        override fun defaultValue() = ReferenceSnyggSpSizeValue(24.sp)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggSpSizeValue)
            val map = referenceSnyggIdToValueMapOf(Size to v.sp.value)
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val size = map.getFloat(Size)
            return@runCatching ReferenceSnyggSpSizeValue(size.sp)
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggPercentageSizeValue(val percentage: Float) : ReferenceSnyggSizeValue {
    companion object : ReferenceSnyggValueEncoder {
        override val spec = ReferenceSnyggValueSpec {
            percentageFloat(id = Size)
        }

        override fun defaultValue() = ReferenceSnyggPercentageSizeValue(0f)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggPercentageSizeValue)
            val map = referenceSnyggIdToValueMapOf(Size to v.percentage * 100.0f)
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val size = map.getFloat(Size) / 100.0f
            return@runCatching ReferenceSnyggPercentageSizeValue(size)
        }
    }

    override fun encoder() = Companion
}
