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

package org.florisboard.lib.snygg

import androidx.compose.material3.ColorScheme
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import org.florisboard.lib.snygg.value.SnyggAssetResolver
import org.florisboard.lib.snygg.value.SnyggDefaultAssetResolver
import org.florisboard.lib.snygg.value.SnyggFontStyleValue
import org.florisboard.lib.snygg.value.SnyggFontWeightValue
import org.florisboard.lib.snygg.value.SnyggUriValue
import java.io.File

/**
 * Pre-compiled style data for a SnyggTheme.
 *
 * The map must fulfill the following contract:
 * 1. The map key represents the element name.
 * 2. All rules mapped to a key are guaranteed to have the same element
 *    name. This is not checked again during style querying.
 * 3. All rules per key are sorted in ascending order based on the comparison
 *    logic defined by SnyggRule.
 * 4. All SnyggPropertySet values are pre-computed and must not contain
 *    variable references.
 * 5. All annotation elements have been pre-processed and stripped from the data.
 */
internal typealias CompiledStyleData = Map<String, List<Pair<SnyggElementRule, SnyggSinglePropertySet>>>

typealias SnyggQueryAttributes = Map<String, Any>

internal typealias CompiledFontFamilyData = Map<String, FontFamily>

/**
 * Represents the runtime style data, which is used for styling UI elements.
 */
data class SnyggTheme internal constructor(
    internal val style: CompiledStyleData,
    internal val fontFamilies: CompiledFontFamilyData,
) {
    internal fun query(
        elementName: String,
        attributes: SnyggQueryAttributes,
        selector: SnyggSelector,
        parentStyle: SnyggSinglePropertySet,
        dynamicLightColorScheme: ColorScheme,
        dynamicDarkColorScheme: ColorScheme,
        fontSizeMultiplier: Float,
    ): SnyggSinglePropertySet {
        return SnyggCompilerAdapter.query(style, elementName, attributes, selector, parentStyle,
            dynamicLightColorScheme, dynamicDarkColorScheme, fontSizeMultiplier)
    }

    companion object {
        internal fun compileFrom(
            stylesheet: SnyggStylesheet,
            assetResolver: SnyggAssetResolver = SnyggDefaultAssetResolver,
        ): SnyggTheme {
            val compiled = SnyggCompilerAdapter.compile(stylesheet)
            val fonts = mutableMapOf<String, MutableList<Font>>()
            compiled.fonts.forEach { (name, sets) ->
                val fontList = fonts.getOrDefault(name, mutableListOf())
                sets.forEach { fontSet ->
                    val src = fontSet.src as? SnyggUriValue ?: return@forEach
                    val path = assetResolver.resolveAbsolutePath(src.uri).getOrNull() ?: return@forEach
                    fontList.add(Font(file = File(path),
                        weight = (fontSet.fontWeight as? SnyggFontWeightValue)?.fontWeight ?: FontWeight.Normal,
                        style = (fontSet.fontStyle as? SnyggFontStyleValue)?.fontStyle ?: FontStyle.Normal))
                }
                if (fontList.isNotEmpty()) fonts[name] = fontList
            }
            val fontFamilies = fonts.mapValues { (_, fonts) ->
                FontFamily(fonts)
            }
            return SnyggTheme(compiled.style, fontFamilies)
        }
    }
}
