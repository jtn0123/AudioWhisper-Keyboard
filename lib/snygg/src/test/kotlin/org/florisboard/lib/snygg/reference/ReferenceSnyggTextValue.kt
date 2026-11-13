/*
 * Copyright (C) 2025 The FlorisBoard Contributors
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

import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow

sealed interface ReferenceSnyggTextValue : ReferenceSnyggValue

data class ReferenceSnyggTextAlignValue(val textAlign: TextAlign) : ReferenceSnyggTextValue {
    companion object : ReferenceSnyggEnumLikeValueEncoder<TextAlign>(
        serializationId = "textAlign",
        serializationMapping = mapOf(
            "left" to TextAlign.Left,
            "right" to TextAlign.Right,
            "center" to TextAlign.Center,
            "justify" to TextAlign.Justify,
            "start" to TextAlign.Start,
            "end" to TextAlign.End,
        ),
        default = TextAlign.Left,
        construct = { ReferenceSnyggTextAlignValue(it) },
        destruct = { (it as ReferenceSnyggTextAlignValue).textAlign },
    )
    override fun encoder() = Companion
}

data class ReferenceSnyggTextDecorationLineValue(val textDecoration: TextDecoration) : ReferenceSnyggTextValue {
    companion object : ReferenceSnyggEnumLikeValueEncoder<TextDecoration>(
        serializationId = "textAlign",
        serializationMapping = mapOf(
            "none" to TextDecoration.None,
            "underline" to TextDecoration.Underline,
            "line-through" to TextDecoration.LineThrough,
        ),
        default = TextDecoration.None,
        construct = { ReferenceSnyggTextDecorationLineValue(it) },
        destruct = { (it as ReferenceSnyggTextDecorationLineValue).textDecoration },
    )
    override fun encoder() = Companion
}

data class ReferenceSnyggTextMaxLinesValue(val maxLines: Int) : ReferenceSnyggTextValue {
    companion object : ReferenceSnyggValueEncoder {
        private const val TextMaxLinesId = "textMaxLines"
        private const val NoneKey = "none"
        private const val NoneValue = Int.MAX_VALUE

        override val spec = ReferenceSnyggValueSpec {
            string(id = TextMaxLinesId, regex = """$NoneKey|[1-9][0-9]*""".toRegex())
        }

        override fun defaultValue() = ReferenceSnyggTextMaxLinesValue(NoneValue)

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggTextMaxLinesValue)
            require(v.maxLines >= 1)
            val map = referenceSnyggIdToValueMapOf(TextMaxLinesId to (
                if (v.maxLines == NoneValue) NoneKey else v.maxLines.toString()
                ))
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val clampValue = map.getString(TextMaxLinesId)
            val maxLines = if (clampValue == NoneKey) NoneValue else clampValue.toInt()
            return@runCatching ReferenceSnyggTextMaxLinesValue(maxLines)
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggTextOverflowValue(val textOverflow: TextOverflow) : ReferenceSnyggTextValue {
    companion object : ReferenceSnyggEnumLikeValueEncoder<TextOverflow>(
        serializationId = "textAlign",
        serializationMapping = mapOf(
            "clip" to TextOverflow.Clip,
            "ellipsis" to TextOverflow.Ellipsis,
            "visible" to TextOverflow.Visible,
        ),
        default = TextOverflow.Clip,
        construct = { ReferenceSnyggTextOverflowValue(it) },
        destruct = { (it as ReferenceSnyggTextOverflowValue).textOverflow },
    )
    override fun encoder() = Companion
}
