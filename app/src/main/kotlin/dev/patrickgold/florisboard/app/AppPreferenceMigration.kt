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

package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.rust.SettingsRules
import dev.patrickgold.jetpref.datastore.model.PreferenceMigrationEntry
import dev.patrickgold.jetpref.datastore.model.PreferenceType

internal fun migrateAppPreference(entry: PreferenceMigrationEntry): PreferenceMigrationEntry {
    val plan = SettingsRules.call(12, entry.type.toString(), entry.key, entry.rawValue)
    return when (plan[0]) {
        "reset" -> entry.reset()
        "keep" -> entry.keepAsIs()
        else -> entry.transform(type = PreferenceType.from(plan[1]), key = plan[2], rawValue = plan[3])
    }
}
