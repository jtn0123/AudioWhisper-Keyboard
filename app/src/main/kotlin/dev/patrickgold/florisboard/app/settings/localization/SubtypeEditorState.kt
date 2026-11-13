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

package dev.patrickgold.florisboard.app.settings.localization

import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.setValue
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.core.SubtypeJsonConfig
import dev.patrickgold.florisboard.ime.core.SubtypeLayoutMap
import dev.patrickgold.florisboard.ime.core.SubtypeNlpProviderMap
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName

internal val SelectComponentName = ExtensionComponentName("00", "00")
internal val SelectNlpProviderId = SelectComponentName.toString()
internal val SelectNlpProviders = SubtypeNlpProviderMap(
    spelling = SelectNlpProviderId,
)
internal val SelectLayoutMap = SubtypeLayoutMap(
    characters = SelectComponentName,
    symbols = SelectComponentName,
    symbols2 = SelectComponentName,
    numeric = SelectComponentName,
    numericAdvanced = SelectComponentName,
    numericRow = SelectComponentName,
    phone = SelectComponentName,
    phone2 = SelectComponentName,
)
internal val SelectLocale = FlorisLocale.from("00", "00")
internal val SelectListKeys = listOf(SelectComponentName)

internal class SubtypeEditorState(init: Subtype?) {
    companion object {
        val Saver = Saver<SubtypeEditorState, String>(
            save = { editor ->
                val subtype = Subtype(
                    id = editor.id.value,
                    primaryLocale = editor.primaryLocale.value,
                    secondaryLocales = editor.secondaryLocales.value,
                    nlpProviders = editor.nlpProviders.value,
                    composer = editor.composer.value,
                    currencySet = editor.currencySet.value,
                    punctuationRule = editor.punctuationRule.value,
                    popupMapping = editor.popupMapping.value,
                    layoutMap = editor.layoutMap.value,
                )
                SubtypeJsonConfig.encodeToString(subtype)
            },
            restore = { str ->
                val subtype = SubtypeJsonConfig.decodeFromString<Subtype>(str)
                SubtypeEditorState(subtype)
            },
        )
    }

    val id: MutableState<Long> = mutableLongStateOf(init?.id ?: -1)
    val primaryLocale: MutableState<FlorisLocale> = mutableStateOf(init?.primaryLocale ?: SelectLocale)
    val secondaryLocales: MutableState<List<FlorisLocale>> = mutableStateOf(init?.secondaryLocales ?: listOf())
    val nlpProviders: MutableState<SubtypeNlpProviderMap> = mutableStateOf(init?.nlpProviders ?: Subtype.DEFAULT.nlpProviders)
    val composer: MutableState<ExtensionComponentName> = mutableStateOf(init?.composer ?: SelectComponentName)
    val currencySet: MutableState<ExtensionComponentName> = mutableStateOf(init?.currencySet ?: SelectComponentName)
    val punctuationRule: MutableState<ExtensionComponentName> = mutableStateOf(init?.punctuationRule ?: Subtype.DEFAULT.punctuationRule)
    val popupMapping: MutableState<ExtensionComponentName> = mutableStateOf(init?.popupMapping ?: SelectComponentName)
    val layoutMap: MutableState<SubtypeLayoutMap> = mutableStateOf(init?.layoutMap ?: SelectLayoutMap)

    fun applySubtype(subtype: Subtype) {
        id.value = subtype.id
        primaryLocale.value = subtype.primaryLocale
        secondaryLocales.value = subtype.secondaryLocales
        composer.value = subtype.composer
        nlpProviders.value = subtype.nlpProviders
        currencySet.value = subtype.currencySet
        punctuationRule.value = subtype.punctuationRule
        popupMapping.value = subtype.popupMapping
        layoutMap.value = subtype.layoutMap
    }

    fun toSubtype() = runCatching<Subtype> {
        check(primaryLocale.value != SelectLocale)
        check(nlpProviders.value.spelling != SelectNlpProviderId)
        check(nlpProviders.value.suggestion != SelectNlpProviderId)
        check(composer.value != SelectComponentName)
        check(currencySet.value != SelectComponentName)
        check(punctuationRule.value != SelectComponentName)
        check(popupMapping.value != SelectComponentName)
        check(layoutMap.value.characters != SelectComponentName)
        check(layoutMap.value.symbols != SelectComponentName)
        check(layoutMap.value.symbols2 != SelectComponentName)
        check(layoutMap.value.numeric != SelectComponentName)
        check(layoutMap.value.numericAdvanced != SelectComponentName)
        check(layoutMap.value.numericRow != SelectComponentName)
        check(layoutMap.value.phone != SelectComponentName)
        check(layoutMap.value.phone2 != SelectComponentName)
        Subtype(
            id.value, primaryLocale.value, secondaryLocales.value, nlpProviders.value, composer.value,
            currencySet.value, punctuationRule.value, popupMapping.value, layoutMap.value,
        )
    }
}
