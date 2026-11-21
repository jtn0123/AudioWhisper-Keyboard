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

package dev.patrickgold.florisboard.ime.core

import android.content.Context
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.settings.SettingsCodecs
import com.jtn0123.audiowhisper.settings.SettingsSubtypes
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.CurrencySet
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.FlorisLocale
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.florisboard.lib.kotlin.collectLatestIn

class SubtypeManager(context: Context) {
    private val prefs by FlorisPreferenceStore
    private val keyboardManager by context.keyboardManager()
    private val scope = CoroutineScope(Dispatchers.Default)
    private val _subtypesFlow = MutableStateFlow(listOf<Subtype>())
    val subtypesFlow = _subtypesFlow.asStateFlow()
    inline var subtypes
        get() = subtypesFlow.value
        private set(v) { _subtypesFlow.value = v }
    private val _activeSubtypeFlow = MutableStateFlow(Subtype.DEFAULT)
    val activeSubtypeFlow = _activeSubtypeFlow.asStateFlow()
    inline var activeSubtype
        get() = activeSubtypeFlow.value
        private set(v) { _activeSubtypeFlow.value = v }

    init {
        prefs.localization.subtypes.asFlow().collectLatestIn(scope) { raw ->
            val list = SettingsCodecs.subtypes(raw)
            val plan = SettingsSubtypes.plan("load", list, prefs.localization.activeSubtypeId.get())
            subtypes = plan.list
            evaluateActiveSubtype(plan.list)
            if (plan.changed) persistNewSubtypeList(plan.list)
        }
    }
    private fun persistNewSubtypeList(list: List<Subtype>) = scope.launch {
        prefs.localization.subtypes.set(SettingsCodecs.subtypes(list))
    }
    private fun evaluateActiveSubtype(list: List<Subtype>) = scope.launch {
        val plan = SettingsSubtypes.plan("evaluate", list, prefs.localization.activeSubtypeId.get())
        if (plan.writeActive) prefs.localization.activeSubtypeId.set(plan.active)
        activeSubtype = plan.selected
    }
    fun addSubtype(subtype: Subtype): Boolean {
        val plan = SettingsSubtypes.plan("add", subtypes, activeSubtype.id, subtype, System.currentTimeMillis())
        if (plan.changed) persistNewSubtypeList(plan.list)
        return plan.changed
    }
    fun getCurrencySet(subtypeToSearch: Subtype): CurrencySet =
        keyboardManager.resources.currencySets.value?.get(subtypeToSearch.currencySet) ?: CurrencySet.Fallback
    fun getSubtypeById(id: Long): Subtype? {
        val list = subtypes
        return list.getOrNull(SettingsRules.call(SettingsOp.SUBTYPE_INDEX, id.toString(), *list.map { it.id.toString() }.toTypedArray())[0].toInt())
    }
    fun getSubtypePresetForLocale(locale: FlorisLocale): SubtypePreset? {
        val presets = keyboardManager.resources.subtypePresets.value.orEmpty()
        val index = SettingsRules.preset(locale.base.toString(),locale.language,
            presets.map { it.locale.base.toString() to it.locale.language })
        return presets.getOrNull(index)
    }
    fun modifySubtypeWithSameId(subtypeToModify: Subtype) {
        val plan = SettingsSubtypes.plan("modify",subtypes,activeSubtype.id,subtypeToModify)
        if (plan.changed) persistNewSubtypeList(plan.list)
    }
    fun removeSubtype(subtypeToRemove: Subtype) {
        val plan = SettingsSubtypes.plan("remove",subtypes,activeSubtype.id,subtypeToRemove)
        if (plan.changed) { persistNewSubtypeList(plan.list); evaluateActiveSubtype(plan.list) }
    }
    fun reorderSubtypes(order: List<Int>) {
        val plan = SettingsSubtypes.plan("reorder",subtypes,activeSubtype.id,order = order)
        if (plan.changed) persistNewSubtypeList(plan.list)
    }
    fun switchToPrevSubtype() = switchSubtype("previous")
    fun switchToNextSubtype() = switchSubtype("next")
    private fun switchSubtype(direction: String) = scope.launch {
        val plan = SettingsSubtypes.plan(direction,subtypes,activeSubtype.id,selected = activeSubtype)
        prefs.localization.activeSubtypeId.set(plan.active)
        activeSubtype = plan.selected
    }
    fun switchToSubtypeById(id: Long) = scope.launch {
        val selected = getSubtypeById(id)
        if (selected != null) { activeSubtype = selected; prefs.localization.activeSubtypeId.set(id) }
    }
}
