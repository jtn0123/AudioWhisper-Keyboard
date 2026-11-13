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

package org.florisboard.lib.snygg.value

import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight

sealed interface SnyggFontValue : SnyggValue {
    companion object {
        internal val FontNameRegex = NativeSnyggValues.patterns[5].toRegex()
    }
}

data class SnyggGenericFontFamilyValue(val fontFamily: FontFamily) : SnyggFontValue {
    companion object : SnyggEnumLikeValueEncoder<FontFamily>(
        serializationId = "genericFontFamily",
        nativeId = 10,
        serializationMapping = NativeSnyggValues.keywords(10).zip(listOf(
            FontFamily.Default,
            FontFamily.SansSerif,
            FontFamily.Serif,
            FontFamily.Monospace,
            FontFamily.Cursive,
        )).toMap(),
        default = FontFamily.Default,
        construct = { SnyggGenericFontFamilyValue(it) },
        destruct = { (it as SnyggGenericFontFamilyValue).fontFamily },
    )
    override fun encoder() = Companion
}

data class SnyggCustomFontFamilyValue(val fontName: String) : SnyggFontValue {
    companion object : SnyggNativeValueEncoder<SnyggCustomFontFamilyValue>(
        nativeId = 11, valueClass = SnyggCustomFontFamilyValue::class,
        construct = { SnyggCustomFontFamilyValue(it[0]) },
        destruct = { listOf(it.fontName) },
    )

    override fun encoder() = Companion
}

data class SnyggFontStyleValue(val fontStyle: FontStyle) : SnyggFontValue {
    companion object : SnyggEnumLikeValueEncoder<FontStyle>(
        serializationId = "textAlign",
        nativeId = 12,
        serializationMapping = NativeSnyggValues.keywords(12).zip(listOf(
            FontStyle.Normal,
            FontStyle.Italic,
        )).toMap(),
        default = FontStyle.Normal,
        construct = { SnyggFontStyleValue(it) },
        destruct = { (it as SnyggFontStyleValue).fontStyle },
    )
    override fun encoder() = Companion
}

data class SnyggFontWeightValue(val fontWeight: FontWeight) : SnyggFontValue {
    companion object : SnyggEnumLikeValueEncoder<FontWeight>(
        serializationId = "textAlign",
        nativeId = 13,
        serializationMapping = NativeSnyggValues.keywords(13).zip(listOf(
            FontWeight(100),
            FontWeight(200),
            FontWeight(300),
            FontWeight(400),
            FontWeight(500),
            FontWeight(600),
            FontWeight(700),
            FontWeight(800),
            FontWeight(900),
            FontWeight(100),
            FontWeight(200),
            FontWeight(300),
            FontWeight(400),
            FontWeight(500),
            FontWeight(600),
            FontWeight(700),
            FontWeight(800),
            FontWeight(900),
        )).toMap(),
        default = FontWeight.Normal,
        construct = { SnyggFontWeightValue(it) },
        destruct = { (it as SnyggFontWeightValue).fontWeight },
    )
    override fun encoder() = Companion
}
