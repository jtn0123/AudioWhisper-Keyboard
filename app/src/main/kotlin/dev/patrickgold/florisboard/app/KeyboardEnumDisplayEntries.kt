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
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.settings.theme.SnyggLevel
import dev.patrickgold.florisboard.ime.input.InputFeedbackActivationMode
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.keyboard.SpaceBarMode
import dev.patrickgold.florisboard.ime.landscapeinput.LandscapeInputUiMode
import dev.patrickgold.florisboard.ime.nlp.SpellingLanguageMode
import dev.patrickgold.florisboard.ime.onehanded.OneHandedMode
import dev.patrickgold.florisboard.ime.smartbar.SmartbarLayout
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.UtilityKeyAction
import dev.patrickgold.florisboard.ime.theme.ThemeMode
import dev.patrickgold.jetpref.datastore.ui.ListPreferenceEntry
import dev.patrickgold.jetpref.datastore.ui.listPrefEntries
import org.florisboard.lib.compose.stringRes
import kotlin.reflect.KClass

private const val DEFAULT = ""

/** The input, keyboard, Smartbar, gesture, theme and utility-key enums of [enumDisplayEntriesOf]. */
internal val KEYBOARD_ENUM_DISPLAY_ENTRIES = mapOf<Pair<KClass<*>, String>, @Composable () -> List<ListPreferenceEntry<*>>>(
    InputFeedbackActivationMode::class to "audio" to {
        listPrefEntries {
            entry(
                key = InputFeedbackActivationMode.RESPECT_SYSTEM_SETTINGS,
                label = stringRes(R.string.enum__input_feedback_activation_mode__audio_respect_system_settings),
            )
            entry(
                key = InputFeedbackActivationMode.IGNORE_SYSTEM_SETTINGS,
                label = stringRes(R.string.enum__input_feedback_activation_mode__audio_ignore_system_settings),
            )
        }
    },
    InputFeedbackActivationMode::class to "haptic" to {
        listPrefEntries {
            entry(
                key = InputFeedbackActivationMode.RESPECT_SYSTEM_SETTINGS,
                label = stringRes(R.string.enum__input_feedback_activation_mode__haptic_respect_system_settings),
            )
            entry(
                key = InputFeedbackActivationMode.IGNORE_SYSTEM_SETTINGS,
                label = stringRes(R.string.enum__input_feedback_activation_mode__haptic_ignore_system_settings),
            )
        }
    },
    InputShiftState::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = InputShiftState.UNSHIFTED,
                label = stringRes(R.string.enum__input_shift_state__unshifted),
            )
            entry(
                key = InputShiftState.SHIFTED_MANUAL,
                label = stringRes(R.string.enum__input_shift_state__shifted_manual),
            )
            entry(
                key = InputShiftState.SHIFTED_AUTOMATIC,
                label = stringRes(R.string.enum__input_shift_state__shifted_automatic),
            )
            entry(
                key = InputShiftState.CAPS_LOCK,
                label = stringRes(R.string.enum__input_shift_state__caps_lock),
            )
        }
    },
    KeyboardMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = KeyboardMode.CHARACTERS,
                label = stringRes(R.string.enum__keyboard_mode__characters),
            )
            entry(
                key = KeyboardMode.SYMBOLS,
                label = stringRes(R.string.enum__keyboard_mode__symbols),
            )
            entry(
                key = KeyboardMode.SYMBOLS2,
                label = stringRes(R.string.enum__keyboard_mode__symbols2),
            )
            entry(
                key = KeyboardMode.NUMERIC,
                label = stringRes(R.string.enum__keyboard_mode__numeric),
            )
            entry(
                key = KeyboardMode.NUMERIC_ADVANCED,
                label = stringRes(R.string.enum__keyboard_mode__numeric_advanced),
            )
            entry(
                key = KeyboardMode.PHONE,
                label = stringRes(R.string.enum__keyboard_mode__phone),
            )
            entry(
                key = KeyboardMode.PHONE2,
                label = stringRes(R.string.enum__keyboard_mode__phone2),
            )
        }
    },
    LandscapeInputUiMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = LandscapeInputUiMode.NEVER_SHOW,
                label = stringRes(R.string.enum__landscape_input_ui_mode__never_show),
            )
            entry(
                key = LandscapeInputUiMode.ALWAYS_SHOW,
                label = stringRes(R.string.enum__landscape_input_ui_mode__always_show),
            )
            entry(
                key = LandscapeInputUiMode.DYNAMICALLY_SHOW,
                label = stringRes(R.string.enum__landscape_input_ui_mode__dynamically_show),
            )
        }
    },
    OneHandedMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = OneHandedMode.START,
                label = stringRes(R.string.enum__one_handed_mode__start),
            )
            entry(
                key = OneHandedMode.END,
                label = stringRes(R.string.enum__one_handed_mode__end),
            )
        }
    },
    SmartbarLayout::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = SmartbarLayout.SUGGESTIONS_ONLY,
                label = stringRes(R.string.enum__smartbar_layout__suggestions_only),
                description = stringRes(R.string.enum__smartbar_layout__suggestions_only__description),
            )
            entry(
                key = SmartbarLayout.ACTIONS_ONLY,
                label = stringRes(R.string.enum__smartbar_layout__actions_only),
                description = stringRes(R.string.enum__smartbar_layout__actions_only__description),
            )
            entry(
                key = SmartbarLayout.SUGGESTIONS_ACTIONS_SHARED,
                label = stringRes(R.string.enum__smartbar_layout__suggestions_action_shared),
                description = stringRes(R.string.enum__smartbar_layout__suggestions_action_shared__description),
            )
            entry(
                key = SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED,
                label = stringRes(R.string.enum__smartbar_layout__suggestions_actions_extended),
                description = stringRes(R.string.enum__smartbar_layout__suggestions_actions_extended__description),
            )
        }
    },
    SnyggLevel::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = SnyggLevel.BASIC,
                label = stringRes(R.string.enum__snygg_level__basic),
                description = stringRes(R.string.enum__snygg_level__basic__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = SnyggLevel.ADVANCED,
                label = stringRes(R.string.enum__snygg_level__advanced),
                description = stringRes(R.string.enum__snygg_level__advanced__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = SnyggLevel.DEVELOPER,
                label = stringRes(R.string.enum__snygg_level__developer),
                description = stringRes(R.string.enum__snygg_level__developer__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    SpaceBarMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = SpaceBarMode.NOTHING,
                label = stringRes(R.string.enum__space_bar_mode__nothing),
            )
            entry(
                key = SpaceBarMode.CURRENT_LANGUAGE,
                label = stringRes(R.string.enum__space_bar_mode__current_language),
            )
            entry(
                key = SpaceBarMode.SPACE_BAR_KEY,
                label = stringRes(R.string.enum__space_bar_mode__space_bar_key),
            )
        }
    },
    SpellingLanguageMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = SpellingLanguageMode.USE_SYSTEM_LANGUAGES,
                label = stringRes(R.string.enum__spelling_language_mode__use_system_languages),
            )
            entry(
                key = SpellingLanguageMode.USE_KEYBOARD_SUBTYPES,
                label = stringRes(R.string.enum__spelling_language_mode__use_keyboard_subtypes),
            )
        }
    },
    SwipeAction::class to "general" to {
        listPrefEntries {
            entry(
                key = SwipeAction.NO_ACTION,
                label = stringRes(R.string.enum__swipe_action__no_action),
            )
            entry(
                key = SwipeAction.CYCLE_TO_PREVIOUS_KEYBOARD_MODE,
                label = stringRes(R.string.enum__swipe_action__cycle_to_previous_keyboard_mode),
            )
            entry(
                key = SwipeAction.CYCLE_TO_NEXT_KEYBOARD_MODE,
                label = stringRes(R.string.enum__swipe_action__cycle_to_next_keyboard_mode),
            )
            entry(
                key = SwipeAction.DELETE_WORD,
                label = stringRes(R.string.enum__swipe_action__delete_word),
            )
            entry(
                key = SwipeAction.HIDE_KEYBOARD,
                label = stringRes(R.string.enum__swipe_action__hide_keyboard),
            )
            entry(
                key = SwipeAction.INSERT_SPACE,
                label = stringRes(R.string.enum__swipe_action__insert_space),
            )
            entry(
                key = SwipeAction.MOVE_CURSOR_UP,
                label = stringRes(R.string.enum__swipe_action__move_cursor_up),
            )
            entry(
                key = SwipeAction.MOVE_CURSOR_DOWN,
                label = stringRes(R.string.enum__swipe_action__move_cursor_down),
            )
            entry(
                key = SwipeAction.MOVE_CURSOR_LEFT,
                label = stringRes(R.string.enum__swipe_action__move_cursor_left),
            )
            entry(
                key = SwipeAction.MOVE_CURSOR_RIGHT,
                label = stringRes(R.string.enum__swipe_action__move_cursor_right),
            )
            entry(
                key = SwipeAction.MOVE_CURSOR_START_OF_LINE,
                label = stringRes(R.string.enum__swipe_action__move_cursor_start_of_line),
            )
            entry(
                key = SwipeAction.MOVE_CURSOR_END_OF_LINE,
                label = stringRes(R.string.enum__swipe_action__move_cursor_end_of_line),
            )
            entry(
                key = SwipeAction.MOVE_CURSOR_START_OF_PAGE,
                label = stringRes(R.string.enum__swipe_action__move_cursor_start_of_page),
            )
            entry(
                key = SwipeAction.MOVE_CURSOR_END_OF_PAGE,
                label = stringRes(R.string.enum__swipe_action__move_cursor_end_of_page),
            )
            entry(
                key = SwipeAction.SHIFT,
                label = stringRes(R.string.enum__swipe_action__shift),
            )
            entry(
                key = SwipeAction.REDO,
                label = stringRes(R.string.enum__swipe_action__redo),
            )
            entry(
                key = SwipeAction.UNDO,
                label = stringRes(R.string.enum__swipe_action__undo),
            )
            entry(
                key = SwipeAction.SWITCH_TO_CLIPBOARD_CONTEXT,
                label = stringRes(R.string.enum__swipe_action__switch_to_clipboard_context),
            )
            entry(
                key = SwipeAction.SWITCH_TO_MEDIA_CONTEXT,
                label = stringRes(R.string.enum__swipe_action__switch_to_media_context)
            )
            entry(
                key = SwipeAction.SHOW_INPUT_METHOD_PICKER,
                label = stringRes(R.string.enum__swipe_action__show_input_method_picker),
            )
            entry(
                key = SwipeAction.SHOW_SUBTYPE_PICKER,
                label = stringRes(R.string.enum__swipe_action__show_subtype_picker),
            )
            entry(
                key = SwipeAction.SWITCH_TO_PREV_SUBTYPE,
                label = stringRes(R.string.enum__swipe_action__switch_to_prev_subtype),
            )
            entry(
                key = SwipeAction.SWITCH_TO_NEXT_SUBTYPE,
                label = stringRes(R.string.enum__swipe_action__switch_to_next_subtype),
            )
            entry(
                key = SwipeAction.SWITCH_TO_PREV_KEYBOARD,
                label = stringRes(R.string.enum__swipe_action__switch_to_prev_keyboard),
            )
            entry(
                key = SwipeAction.TOGGLE_SMARTBAR_VISIBILITY,
                label = stringRes(R.string.enum__swipe_action__toggle_smartbar_visibility),
            )
        }
    },
    SwipeAction::class to "deleteSwipe" to {
        listPrefEntries {
            entry(
                key = SwipeAction.NO_ACTION,
                label = stringRes(R.string.enum__swipe_action__no_action),
            )
            entry(
                key = SwipeAction.DELETE_CHARACTERS_PRECISELY,
                label = stringRes(R.string.enum__swipe_action__delete_characters_precisely),
            )
            entry(
                key = SwipeAction.DELETE_WORD,
                label = stringRes(R.string.enum__swipe_action__delete_word),
            )
            entry(
                key = SwipeAction.DELETE_WORDS_PRECISELY,
                label = stringRes(R.string.enum__swipe_action__delete_words_precisely),
            )
            entry(
                key = SwipeAction.SELECT_CHARACTERS_PRECISELY,
                label = stringRes(R.string.enum__swipe_action__select_characters_precisely),
            )
            entry(
                key = SwipeAction.SELECT_WORDS_PRECISELY,
                label = stringRes(R.string.enum__swipe_action__select_words_precisely),
            )
        }
    },
    SwipeAction::class to "deleteLongPress" to {
        listPrefEntries {
            entry(
                key = SwipeAction.DELETE_CHARACTER,
                label = stringRes(R.string.enum__swipe_action__delete_character),
            )
            entry(
                key = SwipeAction.DELETE_WORD,
                label = stringRes(R.string.enum__swipe_action__delete_word),
            )
        }
    },
    ThemeMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = ThemeMode.ALWAYS_DAY,
                label = stringRes(R.string.enum__theme_mode__always_day),
            )
            entry(
                key = ThemeMode.ALWAYS_NIGHT,
                label = stringRes(R.string.enum__theme_mode__always_night),
            )
            entry(
                key = ThemeMode.FOLLOW_SYSTEM,
                label = stringRes(R.string.enum__theme_mode__follow_system),
            )
            entry(
                key = ThemeMode.FOLLOW_TIME,
                label = stringRes(R.string.enum__theme_mode__follow_time),
            )
        }
    },
    UtilityKeyAction::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = UtilityKeyAction.SWITCH_TO_EMOJIS,
                label = stringRes(R.string.enum__utility_key_action__switch_to_emojis),
            )
            entry(
                key = UtilityKeyAction.SWITCH_LANGUAGE,
                label = stringRes(R.string.enum__utility_key_action__switch_language),
            )
            entry(
                key = UtilityKeyAction.SWITCH_KEYBOARD_APP,
                label = stringRes(R.string.enum__utility_key_action__switch_keyboard_app),
            )
            entry(
                key = UtilityKeyAction.DYNAMIC_SWITCH_LANGUAGE_EMOJIS,
                label = stringRes(R.string.enum__utility_key_action__dynamic_switch_language_emojis),
            )
        }
    },
)
