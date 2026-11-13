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

package dev.patrickgold.florisboard.ime.keyboard

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.lifecycle.MutableLiveData
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.ime.core.SubtypePreset
import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import dev.patrickgold.florisboard.ime.popup.PopupMappingComponent
import dev.patrickgold.florisboard.ime.text.composing.Composer
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class KeyboardManagerResources(context: Context, private val scope: CoroutineScope) {
    private val extensionManager by context.extensionManager()
    val composers = MutableLiveData<Map<ExtensionComponentName, Composer>>(emptyMap())
    val currencySets = MutableLiveData<Map<ExtensionComponentName, CurrencySet>>(emptyMap())
    val layouts = MutableLiveData<Map<LayoutType, Map<ExtensionComponentName, LayoutArrangementComponent>>>(emptyMap())
    val popupMappings = MutableLiveData<Map<ExtensionComponentName, PopupMappingComponent>>(emptyMap())
    val punctuationRules = MutableLiveData<Map<ExtensionComponentName, PunctuationRule>>(emptyMap())
    val subtypePresets = MutableLiveData<List<SubtypePreset>>(emptyList())

    private val anyChangedGuard = Mutex(locked = false)
    val anyChanged = MutableLiveData(Unit)

    init {
        scope.launch(Dispatchers.Main.immediate) {
            extensionManager.keyboardExtensions.observeForever { keyboardExtensions ->
                scope.launch {
                    anyChangedGuard.withLock {
                        parseKeyboardExtensions(keyboardExtensions)
                    }
                }
            }
        }
    }

    private fun parseKeyboardExtensions(keyboardExtensions: List<KeyboardExtension>) {
        val localComposers = mutableMapOf<ExtensionComponentName, Composer>()
        val localCurrencySets = mutableMapOf<ExtensionComponentName, CurrencySet>()
        val localLayouts = mutableMapOf<LayoutType, MutableMap<ExtensionComponentName, LayoutArrangementComponent>>()
        val localPopupMappings = mutableMapOf<ExtensionComponentName, PopupMappingComponent>()
        val localPunctuationRules = mutableMapOf<ExtensionComponentName, PunctuationRule>()
        val localSubtypePresets = mutableListOf<SubtypePreset>()
        for (layoutType in LayoutType.entries) {
            localLayouts[layoutType] = mutableMapOf()
        }
        for (keyboardExtension in keyboardExtensions) {
            keyboardExtension.composers.forEach { composer ->
                localComposers[ExtensionComponentName(keyboardExtension.meta.id, composer.id)] = composer
            }
            keyboardExtension.currencySets.forEach { currencySet ->
                localCurrencySets[ExtensionComponentName(keyboardExtension.meta.id, currencySet.id)] = currencySet
            }
            keyboardExtension.layouts.forEach { (type, layoutComponents) ->
                for (layoutComponent in layoutComponents) {
                    localLayouts[LayoutType.entries.first { it.id == type }]!![ExtensionComponentName(keyboardExtension.meta.id, layoutComponent.id)] = layoutComponent
                }
            }
            keyboardExtension.popupMappings.forEach { popupMapping ->
                localPopupMappings[ExtensionComponentName(keyboardExtension.meta.id, popupMapping.id)] = popupMapping
            }
            keyboardExtension.punctuationRules.forEach { punctuationRule ->
                localPunctuationRules[ExtensionComponentName(keyboardExtension.meta.id, punctuationRule.id)] = punctuationRule
            }
            localSubtypePresets.addAll(keyboardExtension.subtypePresets)
        }
        localSubtypePresets.sortBy { it.locale.displayName() }
        for (languageCode in listOf("en-CA", "en-AU", "en-UK", "en-US")) {
            val index: Int = localSubtypePresets.indexOfFirst { it.locale.languageTag() == languageCode }
            if (index > 0) {
                localSubtypePresets.add(0, localSubtypePresets.removeAt(index))
            }
        }
        subtypePresets.postValue(localSubtypePresets)
        composers.postValue(localComposers)
        currencySets.postValue(localCurrencySets)
        layouts.postValue(localLayouts)
        popupMappings.postValue(localPopupMappings)
        punctuationRules.postValue(localPunctuationRules)
        anyChanged.postValue(Unit)
    }
}
