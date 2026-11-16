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

package dev.patrickgold.florisboard.app

import androidx.compose.runtime.Composable
import com.jtn0123.audiowhisper.suggest.AutoCorrectStrength
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.settings.theme.DisplayKbdAfterDialogs
import dev.patrickgold.florisboard.ime.clipboard.ClipboardSyncBehavior
import dev.patrickgold.florisboard.ime.core.DisplayLanguageNamesIn
import dev.patrickgold.florisboard.ime.input.CapitalizationBehavior
import dev.patrickgold.florisboard.ime.input.HapticVibrationMode
import dev.patrickgold.florisboard.ime.keyboard.IncognitoMode
import dev.patrickgold.florisboard.ime.media.emoji.EmojiHistory
import dev.patrickgold.florisboard.ime.media.emoji.EmojiSkinTone
import dev.patrickgold.florisboard.ime.media.emoji.EmojiSuggestionType
import dev.patrickgold.florisboard.ime.smartbar.CandidatesDisplayMode
import dev.patrickgold.florisboard.ime.smartbar.ExtendedActionsPlacement
import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import dev.patrickgold.jetpref.datastore.ui.ListPreferenceEntry
import dev.patrickgold.jetpref.datastore.ui.listPrefEntries
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import org.florisboard.lib.compose.stringRes
import org.florisboard.lib.kotlin.curlyFormat
import kotlin.reflect.KClass

private const val DEFAULT = ""

private val ENUM_DISPLAY_ENTRIES = mapOf<Pair<KClass<*>, String>, @Composable () -> List<ListPreferenceEntry<*>>>(
    AppTheme::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = AppTheme.AUTO,
                label = stringRes(R.string.settings__system_default),
            )
            entry(
                key = AppTheme.AUTO_AMOLED,
                label = stringRes(R.string.pref__other__settings_theme__auto_amoled),
            )
            entry(
                key = AppTheme.LIGHT,
                label = stringRes(R.string.pref__other__settings_theme__light),
            )
            entry(
                key = AppTheme.DARK,
                label = stringRes(R.string.pref__other__settings_theme__dark),
            )
            entry(
                key = AppTheme.AMOLED_DARK,
                label = stringRes(R.string.pref__other__settings_theme__amoled_dark),
            )
        }
    },
    CandidatesDisplayMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = CandidatesDisplayMode.CLASSIC,
                label = stringRes(R.string.enum__candidates_display_mode__classic),
            )
            entry(
                key = CandidatesDisplayMode.DYNAMIC,
                label = stringRes(R.string.enum__candidates_display_mode__dynamic),
            )
            entry(
                key = CandidatesDisplayMode.DYNAMIC_SCROLLABLE,
                label = stringRes(R.string.enum__candidates_display_mode__dynamic_scrollable),
            )
        }
    },
    CapitalizationBehavior::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = CapitalizationBehavior.CAPSLOCK_BY_DOUBLE_TAP,
                label = stringRes(R.string.enum__capitalization_behavior__capslock_by_double_tap),
            )
            entry(
                key = CapitalizationBehavior.CAPSLOCK_BY_CYCLE,
                label = stringRes(R.string.enum__capitalization_behavior__capslock_by_cycle),
            )
        }
    },
    ClipboardSyncBehavior::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = ClipboardSyncBehavior.NO_EVENTS,
                label = stringRes(R.string.enum__clipboard_sync_behavior__no_events),
                description = stringRes(R.string.enum__clipboard_sync_behavior__no_events__description),
            )
            entry(
                key = ClipboardSyncBehavior.ONLY_CLEAR_EVENTS,
                label = stringRes(R.string.enum__clipboard_sync_behavior__only_clear_events),
                description = stringRes(R.string.enum__clipboard_sync_behavior__only_clear_events__description),
            )
            entry(
                key = ClipboardSyncBehavior.ONLY_SET_EVENTS,
                label = stringRes(R.string.enum__clipboard_sync_behavior__only_set_events),
                description = stringRes(R.string.enum__clipboard_sync_behavior__only_set_events__description),
            )
            entry(
                key = ClipboardSyncBehavior.ALL_EVENTS,
                label = stringRes(R.string.enum__clipboard_sync_behavior__all_events),
                description = stringRes(R.string.enum__clipboard_sync_behavior__all_events__description),
            )
        }
    },
    ColorRepresentation::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = ColorRepresentation.HEX,
                label = stringRes(R.string.enum__color_representation__hex),
                description = stringRes(R.string.general__example_given).curlyFormat("example" to "#4caf50ff"),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = ColorRepresentation.RGB,
                label = stringRes(R.string.enum__color_representation__rgb),
                description = stringRes(R.string.general__example_given).curlyFormat("example" to "rgba(76, 175, 80, 1.0)"),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = ColorRepresentation.HSV,
                label = stringRes(R.string.enum__color_representation__hsv),
                description = stringRes(R.string.general__example_given).curlyFormat("example" to "hsva(122, 56, 68, 1.0)"),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    DisplayKbdAfterDialogs::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = DisplayKbdAfterDialogs.ALWAYS,
                label = stringRes(R.string.enum__display_kbd_after_dialogs__always),
                description = stringRes(R.string.enum__display_kbd_after_dialogs__always__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = DisplayKbdAfterDialogs.NEVER,
                label = stringRes(R.string.enum__display_kbd_after_dialogs__never),
                description = stringRes(R.string.enum__display_kbd_after_dialogs__never__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = DisplayKbdAfterDialogs.REMEMBER,
                label = stringRes(R.string.enum__display_kbd_after_dialogs__remember),
                description = stringRes(R.string.enum__display_kbd_after_dialogs__remember__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    DisplayLanguageNamesIn::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = DisplayLanguageNamesIn.SYSTEM_LOCALE,
                label = stringRes(R.string.enum__display_language_names_in__system_locale),
                description = stringRes(R.string.enum__display_language_names_in__system_locale__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = DisplayLanguageNamesIn.NATIVE_LOCALE,
                label = stringRes(R.string.enum__display_language_names_in__native_locale),
                description = stringRes(R.string.enum__display_language_names_in__native_locale__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    EmojiHistory.UpdateStrategy::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = EmojiHistory.UpdateStrategy.AUTO_SORT_PREPEND,
                label = stringRes(R.string.enum__emoji_history_update_strategy__auto_sort_prepend),
                description = stringRes(R.string.enum__emoji_history_update_strategy__auto_sort_prepend__description),
            )
            entry(
                key = EmojiHistory.UpdateStrategy.AUTO_SORT_APPEND,
                label = stringRes(R.string.enum__emoji_history_update_strategy__auto_sort_append),
                description = stringRes(R.string.enum__emoji_history_update_strategy__auto_sort_append__description),
            )
            entry(
                key = EmojiHistory.UpdateStrategy.MANUAL_SORT_PREPEND,
                label = stringRes(R.string.enum__emoji_history_update_strategy__manual_sort_prepend),
                description = stringRes(R.string.enum__emoji_history_update_strategy__manual_sort_prepend__description),
            )
            entry(
                key = EmojiHistory.UpdateStrategy.MANUAL_SORT_APPEND,
                label = stringRes(R.string.enum__emoji_history_update_strategy__manual_sort_append),
                description = stringRes(R.string.enum__emoji_history_update_strategy__manual_sort_append__description),
            )
        }
    },
    EmojiSkinTone::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = EmojiSkinTone.DEFAULT,
                label = stringRes(
                    R.string.enum__emoji_skin_tone__default,
                    "emoji" to "\uD83D\uDC4B" // 👋
                ),
            )
            entry(
                key = EmojiSkinTone.LIGHT_SKIN_TONE,
                label = stringRes(
                    R.string.enum__emoji_skin_tone__light_skin_tone,
                    "emoji" to "\uD83D\uDC4B\uD83C\uDFFB" // 👋🏻
                ),
            )
            entry(
                key = EmojiSkinTone.MEDIUM_LIGHT_SKIN_TONE,
                label = stringRes(
                    R.string.enum__emoji_skin_tone__medium_light_skin_tone,
                    "emoji" to "\uD83D\uDC4B\uD83C\uDFFC" // 👋🏼
                ),
            )
            entry(
                key = EmojiSkinTone.MEDIUM_SKIN_TONE,
                label = stringRes(
                    R.string.enum__emoji_skin_tone__medium_skin_tone,
                    "emoji" to "\uD83D\uDC4B\uD83C\uDFFD" // 👋🏽
                ),
            )
            entry(
                key = EmojiSkinTone.MEDIUM_DARK_SKIN_TONE,
                label = stringRes(
                    R.string.enum__emoji_skin_tone__medium_dark_skin_tone,
                    "emoji" to "\uD83D\uDC4B\uD83C\uDFFE" // 👋🏾
                ),
            )
            entry(
                key = EmojiSkinTone.DARK_SKIN_TONE,
                label = stringRes(
                    R.string.enum__emoji_skin_tone__dark_skin_tone,
                    "emoji" to "\uD83D\uDC4B\uD83C\uDFFF" // 👋🏿
                ),
            )
        }
    },
    EmojiSuggestionType::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = EmojiSuggestionType.LEADING_COLON,
                label = stringRes(R.string.enum__emoji_suggestion_type__leading_colon),
                description = stringRes(R.string.enum__emoji_suggestion_type__leading_colon__description),
            )
            entry(
                key = EmojiSuggestionType.INLINE_TEXT,
                label = stringRes(R.string.enum__emoji_suggestion_type__inline_text),
                description = stringRes(R.string.enum__emoji_suggestion_type__inline_text__description),
            )
        }
    },
    ExtendedActionsPlacement::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = ExtendedActionsPlacement.ABOVE_CANDIDATES,
                label = stringRes(R.string.enum__extended_actions_placement__above_candidates),
                description = stringRes(R.string.enum__extended_actions_placement__above_candidates__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = ExtendedActionsPlacement.BELOW_CANDIDATES,
                label = stringRes(R.string.enum__extended_actions_placement__below_candidates),
                description = stringRes(R.string.enum__extended_actions_placement__below_candidates__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = ExtendedActionsPlacement.OVERLAY_APP_UI,
                label = stringRes(R.string.enum__extended_actions_placement__overlay_app_ui),
                description = stringRes(R.string.enum__extended_actions_placement__overlay_app_ui__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    HapticVibrationMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = HapticVibrationMode.USE_VIBRATOR_DIRECTLY,
                label = stringRes(R.string.enum__haptic_vibration_mode__use_vibrator_directly),
                description = stringRes(R.string.enum__haptic_vibration_mode__use_vibrator_directly__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = HapticVibrationMode.USE_HAPTIC_FEEDBACK_INTERFACE,
                label = stringRes(R.string.enum__haptic_vibration_mode__use_haptic_feedback_interface),
                description = stringRes(R.string.enum__haptic_vibration_mode__use_haptic_feedback_interface__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    KeyHintMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = KeyHintMode.ACCENT_PRIORITY,
                label = stringRes(R.string.enum__key_hint_mode__accent_priority),
                description = stringRes(R.string.enum__key_hint_mode__accent_priority__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = KeyHintMode.HINT_PRIORITY,
                label = stringRes(R.string.enum__key_hint_mode__hint_priority),
                description = stringRes(R.string.enum__key_hint_mode__hint_priority__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = KeyHintMode.SMART_PRIORITY,
                label = stringRes(R.string.enum__key_hint_mode__smart_priority),
                description = stringRes(R.string.enum__key_hint_mode__smart_priority__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    IncognitoDisplayMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = IncognitoDisplayMode.REPLACE_SHARED_ACTIONS_TOGGLE,
                label = stringRes(id = R.string.enum__incognito_display_mode__replace_shared_actions_toggle),
            )
            entry(
                key = IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD,
                label = stringRes(id = R.string.enum__incognito_display_mode__display_behind_keyboard),
            )
        }
    },
    AutoCorrectStrength::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = AutoCorrectStrength.OFF,
                label = stringRes(R.string.enum__auto_correct_strength__off),
                description = stringRes(R.string.enum__auto_correct_strength__off__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = AutoCorrectStrength.MODEST,
                label = stringRes(R.string.enum__auto_correct_strength__modest),
                description = stringRes(R.string.enum__auto_correct_strength__modest__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = AutoCorrectStrength.NORMAL,
                label = stringRes(R.string.enum__auto_correct_strength__normal),
                description = stringRes(R.string.enum__auto_correct_strength__normal__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = AutoCorrectStrength.AGGRESSIVE,
                label = stringRes(R.string.enum__auto_correct_strength__aggressive),
                description = stringRes(R.string.enum__auto_correct_strength__aggressive__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    IncognitoMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = IncognitoMode.FORCE_OFF,
                label = stringRes(R.string.enum__incognito_mode__force_off),
                description = stringRes(R.string.enum__incognito_mode__force_off__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = IncognitoMode.DYNAMIC_ON_OFF,
                label = stringRes(R.string.enum__incognito_mode__dynamic_on_off),
                description = stringRes(R.string.enum__incognito_mode__dynamic_on_off__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = IncognitoMode.FORCE_ON,
                label = stringRes(R.string.enum__incognito_mode__force_on),
                description = stringRes(R.string.enum__incognito_mode__force_on__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
) + KEYBOARD_ENUM_DISPLAY_ENTRIES

@Composable
fun <V : Any> enumDisplayEntriesOf(
    enumClass: KClass<V>,
    variant: String = DEFAULT,
): List<ListPreferenceEntry<V>> {
    @Suppress("UNCHECKED_CAST")
    return ENUM_DISPLAY_ENTRIES[enumClass to variant]?.invoke()
        as List<ListPreferenceEntry<V>>
}
