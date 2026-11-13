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

import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import org.florisboard.lib.snygg.reference.ReferenceSnyggFontValue.Companion.FontNameRegex

sealed interface ReferenceSnyggFontValue : ReferenceSnyggValue {
    companion object {
        internal val FontNameRegex = """`(?<fontName>[a-zA-Z0-9\s_-]+)`""".toRegex()
    }
}

data class ReferenceSnyggGenericFontFamilyValue(val fontFamily: FontFamily) : ReferenceSnyggFontValue {
    companion object : ReferenceSnyggEnumLikeValueEncoder<FontFamily>(
        serializationId = "genericFontFamily",
        serializationMapping = mapOf(
            "system" to FontFamily.Default,
            "sans-serif" to FontFamily.SansSerif,
            "serif" to FontFamily.Serif,
            "monospace" to FontFamily.Monospace,
            "cursive" to FontFamily.Cursive,
        ),
        default = FontFamily.Default,
        construct = { ReferenceSnyggGenericFontFamilyValue(it) },
        destruct = { (it as ReferenceSnyggGenericFontFamilyValue).fontFamily },
    )
    override fun encoder() = Companion
}

data class ReferenceSnyggCustomFontFamilyValue(val fontName: String) : ReferenceSnyggFontValue {
    companion object : ReferenceSnyggValueEncoder {
        private const val EnclosedFontNameId = "enclosedFontName"

        override val spec = ReferenceSnyggValueSpec {
            string(id = EnclosedFontNameId, regex = FontNameRegex)
        }

        override fun defaultValue() = ReferenceSnyggCustomFontFamilyValue("")

        override fun serialize(v: ReferenceSnyggValue) = runCatching<String> {
            require(v is ReferenceSnyggCustomFontFamilyValue)
            val map = referenceSnyggIdToValueMapOf(EnclosedFontNameId to "`${v.fontName}`")
            return@runCatching spec.pack(map)
        }

        override fun deserialize(v: String) = runCatching<ReferenceSnyggValue> {
            val map = referenceSnyggIdToValueMapOf()
            spec.parse(v, map)
            val enclosedFontName = map.getString(EnclosedFontNameId)
            val fontName = enclosedFontName.substring(1, enclosedFontName.length - 1)
            return@runCatching ReferenceSnyggCustomFontFamilyValue(fontName)
        }
    }

    override fun encoder() = Companion
}

data class ReferenceSnyggFontStyleValue(val fontStyle: FontStyle) : ReferenceSnyggFontValue {
    companion object : ReferenceSnyggEnumLikeValueEncoder<FontStyle>(
        serializationId = "textAlign",
        serializationMapping = mapOf(
            "normal" to FontStyle.Normal,
            "italic" to FontStyle.Italic,
        ),
        default = FontStyle.Normal,
        construct = { ReferenceSnyggFontStyleValue(it) },
        destruct = { (it as ReferenceSnyggFontStyleValue).fontStyle },
    )
    override fun encoder() = Companion
}

data class ReferenceSnyggFontWeightValue(val fontWeight: FontWeight) : ReferenceSnyggFontValue {
    companion object : ReferenceSnyggEnumLikeValueEncoder<FontWeight>(
        serializationId = "textAlign",
        serializationMapping = mapOf(
            "thin" to FontWeight(100),
            "extra-light" to FontWeight(200),
            "light" to FontWeight(300),
            "normal" to FontWeight(400),
            "medium" to FontWeight(500),
            "semi-bold" to FontWeight(600),
            "bold" to FontWeight(700),
            "extra-bold" to FontWeight(800),
            "black" to FontWeight(900),
            "100" to FontWeight(100),
            "200" to FontWeight(200),
            "300" to FontWeight(300),
            "400" to FontWeight(400),
            "500" to FontWeight(500),
            "600" to FontWeight(600),
            "700" to FontWeight(700),
            "800" to FontWeight(800),
            "900" to FontWeight(900),
        ),
        default = FontWeight.Normal,
        construct = { ReferenceSnyggFontWeightValue(it) },
        destruct = { (it as ReferenceSnyggFontWeightValue).fontWeight },
    )
    override fun encoder() = Companion
}
