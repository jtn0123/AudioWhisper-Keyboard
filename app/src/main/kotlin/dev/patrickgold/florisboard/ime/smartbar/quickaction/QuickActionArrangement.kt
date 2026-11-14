/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.smartbar.quickaction

import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer
import kotlinx.serialization.Serializable

@Serializable
data class QuickActionArrangement(
    val stickyAction: QuickAction?,
    val dynamicActions: List<QuickAction>,
    val hiddenActions: List<QuickAction>,
) {
    operator fun contains(action: QuickAction): Boolean {
        return stickyAction == action || dynamicActions.contains(action) || hiddenActions.contains(action)
    }

    fun distinct(): QuickActionArrangement {
        val pool = (listOfNotNull(stickyAction) + dynamicActions + hiddenActions).distinct()
        val result = com.jtn0123.audiowhisper.rust.PanelRules.arrange(
            intArrayOf(pool.indexOf(stickyAction), 0, 0, 0),
            dynamicActions.map { pool.indexOf(it) }.toIntArray(), hiddenActions.map { pool.indexOf(it) }.toIntArray(),
        )
        return QuickActionArrangement(pool.getOrNull(result[0][0]), result[1].map { pool[it] }, result[2].map { pool[it] })
    }

    companion object {
        val Default by lazy { com.jtn0123.audiowhisper.settings.SettingsDefaults.arrangement("smartbar__action_arrangement") }
    }

    object Serializer : PreferenceSerializer<QuickActionArrangement> {
        override fun serialize(value: QuickActionArrangement): String {
            return com.jtn0123.audiowhisper.settings.SettingsCodecs.arrangement(value)
        }

        override fun deserialize(value: String): QuickActionArrangement {
            return com.jtn0123.audiowhisper.settings.SettingsCodecs.arrangement(value)
        }
    }
}
