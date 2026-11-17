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

import androidx.compose.runtime.getValue
import dev.patrickgold.florisboard.ime.clipboard.ClipboardSyncBehavior
import dev.patrickgold.florisboard.ime.media.emoji.EmojiHistory
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionArrangement
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionJsonConfig
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.jetpref.datastore.model.PreferenceMigrationEntry
import dev.patrickgold.jetpref.datastore.model.PreferenceType
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import kotlinx.serialization.json.Json

internal fun referenceMigrateAppPreference(entry: PreferenceMigrationEntry): PreferenceMigrationEntry {
    if (entry.key.startsWith("devtools__")) return entry.reset()
    return when (entry.key) {

        // Migrate media prefs to emoji prefs
        // Keep migration rule until: 0.6 dev cycle
        "media__emoji_recently_used" -> {
            val emojiValues = entry.rawValue.split(";")
            val recent = emojiValues.map {
                dev.patrickgold.florisboard.ime.media.emoji.Emoji(it, "", emptyList())
            }
            val data = EmojiHistory(emptyList(), recent)
            entry.transform(key = "emoji__history_data", rawValue = Json.encodeToString(data))
        }
        "media__emoji_recently_used_max_size" -> {
            entry.transform(key = "emoji__history_recent_max_size")
        }

        // Migrate advanced prefs to other prefs
        // Keep migration rules until: 0.7 dev cycle
        "advanced__settings_theme" -> {
            entry.transform(key = "other__settings_theme")
        }
        "advanced__accent_color" -> {
            entry.transform(key = "other__accent_color")
        }
        // AudioWhisper: the settings are English only, so there is no language to pick.
        "advanced__settings_language", "other__settings_language" -> {
            entry.reset()
        }
        // AudioWhisper: Android always shows the app icon, so there is nothing to choose.
        "advanced__show_app_icon", "other__show_app_icon" -> {
            entry.reset()
        }
        // AudioWhisper: Rust always draws settings, so the switch has nothing to choose.
        "other__rust_settings_prototype" -> {
            entry.reset()
        }
        "advanced__incognito_mode" -> {
            entry.transform(key = "suggestion__incognito_mode")
        }
        "advanced__force_incognito_mode_from_dynamic" -> {
            entry.transform(key = "suggestion__force_incognito_mode_from_dynamic")
        }
        // Migrate clipboard suggestion prefs to clipboard
        // Keep migration rules until: 0.7 dev cycle
        "suggestion__clipboard_content_enabled" -> {
            entry.transform(key = "clipboard__suggestion_enabled")
        }
        "suggestion__clipboard_content_timeout" -> {
            entry.transform(key = "clipboard__suggestion_timeout")
        }

        //Migrate one hand mode prefs keep until: 0.7 dev cycle
        "keyboard__one_handed_mode" -> {
            if (entry.rawValue == "OFF") {
                entry.reset()
            } else {
                entry.keepAsIs()
            }
        }
        "smartbar__action_arrangement" -> {
            fun migrateAction(action: QuickAction): QuickAction {
                return if (action is QuickAction.InsertKey && action.data.code == KeyCode.COMPACT_LAYOUT_TO_RIGHT) {
                    action.copy(TextKeyData.TOGGLE_COMPACT_LAYOUT)
                } else {
                    action
                }
            }

            val arrangement = QuickActionJsonConfig.decodeFromString<QuickActionArrangement>(entry.rawValue)
            var newArrangement = arrangement.copy(
                stickyAction = arrangement.stickyAction?.let{ migrateAction(it) },
                dynamicActions = arrangement.dynamicActions.map { migrateAction(it) },
                hiddenActions = arrangement.hiddenActions.map { migrateAction(it) },
            )
            if (QuickAction.InsertKey(TextKeyData.LANGUAGE_SWITCH) !in newArrangement) {
                newArrangement = newArrangement.copy(
                    dynamicActions = newArrangement.dynamicActions.plus(QuickAction.InsertKey(TextKeyData.LANGUAGE_SWITCH))
                )
            }
            if (QuickAction.InsertKey(TextKeyData.FORWARD_DELETE) !in newArrangement) {
                newArrangement = newArrangement.copy(
                    dynamicActions = newArrangement.dynamicActions.plus(QuickAction.InsertKey(TextKeyData.FORWARD_DELETE))
                )
            }
            if (QuickAction.InsertKey(TextKeyData.IME_HIDE_UI) !in newArrangement) {
                newArrangement = newArrangement.copy(
                    dynamicActions = newArrangement.dynamicActions.plus(QuickAction.InsertKey(TextKeyData.IME_HIDE_UI))
                )
            }
            val json = QuickActionJsonConfig.encodeToString(newArrangement.distinct())
            entry.transform(rawValue = json)
        }

        // Migrate theme editor fine-tuning
        // Keep migration rule until: 0.6 dev cycle
        "theme__editor_display_colors_as" -> {
            val colorRepresentation = when (entry.rawValue) {
                "RGBA" -> ColorRepresentation.RGB
                else -> ColorRepresentation.HEX
            }
            entry.transform(
                key = "theme__editor_color_representation",
                rawValue = colorRepresentation.name,
            )
        }

        // Migrate clipboard history pref names
        // Keep migration rules until: 0.7 dev cycle
        "clipboard__sync_to_floris", "clipboard__sync_to_system" -> {
            entry.transform(
                type = PreferenceType.string(),
                rawValue = when (entry.rawValue) {
                    "true" -> ClipboardSyncBehavior.ALL_EVENTS.name
                    "false" -> ClipboardSyncBehavior.NO_EVENTS.name
                    else -> entry.rawValue
                },
            )
        }
        "clipboard__num_history_grid_columns_portrait" -> {
            entry.transform(key = "clipboard__history_num_grid_columns_portrait")
        }
        "clipboard__num_history_grid_columns_landscape" -> {
            entry.transform(key = "clipboard__history_num_grid_columns_landscape")
        }
        "clipboard__clean_up_old" -> {
            entry.transform(key = "clipboard__history_auto_clean_old_enabled")
        }
        "clipboard__clean_up_after" -> {
            entry.transform(key = "clipboard__history_auto_clean_old_after")
        }
        "clipboard__auto_clean_sensitive" -> {
            entry.transform(key = "clipboard__history_auto_clean_sensitive_enabled")
        }
        "clipboard__auto_clean_sensitive_after" -> {
            entry.transform(key = "clipboard__history_auto_clean_sensitive_after")
        }
        "clipboard__limit_history_size" -> {
            entry.transform(key = "clipboard__history_size_limit_enabled")
        }
        "clipboard__max_history_size" -> {
            entry.transform(key = "clipboard__history_size_limit")
        }
        "clipboard__clear_primary_clip_deletes_last_item" -> {
            entry.transform(key = "clipboard__clear_primary_clip_affects_history_if_unpinned")
        }

        // Default: keep entry
        else -> entry.keepAsIs()
    }
}
