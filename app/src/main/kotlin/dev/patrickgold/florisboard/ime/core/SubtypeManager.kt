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
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.CurrencySet
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.florisboard.lib.kotlin.collectLatestIn

/** Where the subtype settings live: the stored list as it changes, and the active subtype's id. */
internal interface SubtypeStore {
    val lists: StateFlow<String>
    fun activeId(): Long
    suspend fun setActiveId(id: Long)
    suspend fun setList(raw: String)
}

/**
 * The keyboard's subtypes. Rust ([SubtypeSession], crates/imesubtype) holds the list and the
 * active subtype and decides every change; this class reads the settings it decides from, writes
 * the ones it asks for, and publishes the result to the flows the Compose keyboard and the other
 * managers collect.
 */
class SubtypeManager internal constructor(
    private val store: SubtypeStore,
    private val currencySets: () -> Map<ExtensionComponentName, CurrencySet>?,
    private val scope: CoroutineScope,
    private val session: SubtypeSession = SubtypeSession(),
) {
    constructor(context: Context) : this(PreferenceSubtypeStore(), context.keyboardManager().let { manager ->
        { manager.value.resources.currencySets.value }
    }, CoroutineScope(Dispatchers.Default))

    private val _subtypesFlow = MutableStateFlow(listOf<Subtype>())
    val subtypesFlow = _subtypesFlow.asStateFlow()
    inline var subtypes
        get() = subtypesFlow.value
        private set(v) { _subtypesFlow.value = v }

    /** The active subtype and what Rust says it names, set together. */
    @Volatile
    internal var active: SubtypeEntry = session.active()
        private set(value) {
            field = value
            _activeSubtypeFlow.value = value.subtype
        }
    private val _activeSubtypeFlow = MutableStateFlow(active.subtype)
    val activeSubtypeFlow = _activeSubtypeFlow.asStateFlow()
    val activeSubtype get() = activeSubtypeFlow.value

    init {
        store.lists.collectLatestIn(scope) { raw ->
            val loaded = session.load(raw)
            subtypes = loaded.list.map { it.subtype }
            evaluateActiveSubtype()
            loaded.repaired?.let { scope.launch { store.setList(it) } }
        }
    }

    private fun evaluateActiveSubtype() = scope.launch {
        val switched = session.evaluate(store.activeId())
        if (switched.write) store.setActiveId(switched.active.subtype.id)
        active = switched.active
    }

    fun getCurrencySet(subtypeToSearch: Subtype): CurrencySet =
        session.currencySet(currencySets(), subtypeToSearch.currencySet) ?: CurrencySet.Fallback

    fun switchToPrevSubtype() = switchSubtype(session::previous)
    fun switchToNextSubtype() = switchSubtype(session::next)

    private fun switchSubtype(direction: () -> SubtypeSession.Switched) = scope.launch {
        val switched = direction()
        store.setActiveId(switched.active.subtype.id)
        active = switched.active
    }

    fun switchToSubtypeById(id: Long) = scope.launch {
        val switched = session.select(id) ?: return@launch
        active = switched.active
        store.setActiveId(id)
    }
}

/** The subtype settings in Jetpref's store. */
private class PreferenceSubtypeStore : SubtypeStore {
    private val prefs by FlorisPreferenceStore
    override val lists get() = prefs.localization.subtypes.asFlow()
    override fun activeId() = prefs.localization.activeSubtypeId.get()
    override suspend fun setActiveId(id: Long) { prefs.localization.activeSubtypeId.set(id) }
    override suspend fun setList(raw: String) { prefs.localization.subtypes.set(raw) }
}
