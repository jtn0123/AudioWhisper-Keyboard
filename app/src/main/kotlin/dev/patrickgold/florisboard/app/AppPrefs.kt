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

import com.jtn0123.audiowhisper.settings.SettingsDefaults
import com.jtn0123.audiowhisper.settings.SettingsPreferenceModel
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalConfiguration
import dev.patrickgold.florisboard.app.settings.theme.ColorPreferenceSerializer
import dev.patrickgold.florisboard.app.settings.theme.DisplayKbdAfterDialogs
import dev.patrickgold.florisboard.app.settings.theme.SnyggLevel
import dev.patrickgold.florisboard.app.setup.NotificationPermissionState
import dev.patrickgold.florisboard.ime.clipboard.ClipboardSyncBehavior
import dev.patrickgold.florisboard.ime.core.DisplayLanguageNamesIn
import dev.patrickgold.florisboard.ime.input.CapitalizationBehavior
import dev.patrickgold.florisboard.ime.input.HapticVibrationMode
import dev.patrickgold.florisboard.ime.input.InputFeedbackActivationMode
import dev.patrickgold.florisboard.ime.keyboard.IncognitoMode
import dev.patrickgold.florisboard.ime.keyboard.SpaceBarMode
import dev.patrickgold.florisboard.ime.landscapeinput.LandscapeInputUiMode
import dev.patrickgold.florisboard.ime.media.emoji.EmojiHairStyle
import dev.patrickgold.florisboard.ime.media.emoji.EmojiHistory
import dev.patrickgold.florisboard.ime.media.emoji.EmojiSkinTone
import dev.patrickgold.florisboard.ime.media.emoji.EmojiSuggestionType
import dev.patrickgold.florisboard.ime.nlp.SpellingLanguageMode
import dev.patrickgold.florisboard.ime.onehanded.OneHandedMode
import dev.patrickgold.florisboard.ime.smartbar.CandidatesDisplayMode
import dev.patrickgold.florisboard.ime.smartbar.ExtendedActionsPlacement
import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode
import dev.patrickgold.florisboard.ime.smartbar.SmartbarLayout
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionArrangement
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import dev.patrickgold.florisboard.ime.text.key.UtilityKeyAction
import dev.patrickgold.florisboard.ime.theme.ThemeMode
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.jetpref.datastore.annotations.Preferences
import dev.patrickgold.jetpref.datastore.jetprefDataStoreOf
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import dev.patrickgold.jetpref.datastore.model.PreferenceMigrationEntry
import dev.patrickgold.jetpref.datastore.model.observeAsState
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import org.florisboard.lib.android.isOrientationPortrait

val FlorisPreferenceStore = jetprefDataStoreOf(FlorisPreferenceModel::class)

@Preferences
abstract class FlorisPreferenceModel : SettingsPreferenceModel() {
    companion object {
        const val NAME = "florisboard-app-prefs"
    }

    val clipboard = Clipboard()
    inner class Clipboard {
        val useInternalClipboard = nativeBoolean(key = "clipboard__use_internal_clipboard")
        val syncToFloris = nativeEnum<ClipboardSyncBehavior>(key = "clipboard__sync_to_floris")
        val syncToSystem = nativeEnum<ClipboardSyncBehavior>(key = "clipboard__sync_to_system")
        val suggestionEnabled = nativeBoolean(key = "clipboard__suggestion_enabled")
        val suggestionTimeout = nativeInt(key = "clipboard__suggestion_timeout")
        val historyEnabled = nativeBoolean(key = "clipboard__history_enabled")
        val historyNumGridColumnsPortrait = nativeInt(key = "clipboard__history_num_grid_columns_portrait")
        val historyNumGridColumnsLandscape = nativeInt(key = "clipboard__history_num_grid_columns_landscape")
        @Composable
        fun historyNumGridColumns(): PreferenceData<Int> {
            val configuration = LocalConfiguration.current
            return if (configuration.isOrientationPortrait()) {
                historyNumGridColumnsPortrait
            } else {
                historyNumGridColumnsLandscape
            }
        }
        val historyAutoCleanOldEnabled = nativeBoolean(key = "clipboard__history_auto_clean_old_enabled")
        val historyAutoCleanOldAfter = nativeInt(key = "clipboard__history_auto_clean_old_after")
        val historyAutoCleanSensitiveEnabled = nativeBoolean(key = "clipboard__history_auto_clean_sensitive_enabled")
        val historyAutoCleanSensitiveAfter = nativeInt(key = "clipboard__history_auto_clean_sensitive_after")
        val historySizeLimitEnabled = nativeBoolean(key = "clipboard__history_size_limit_enabled")
        val historySizeLimit = nativeInt(key = "clipboard__history_size_limit")
        val historyHideOnPaste = nativeBoolean(key = "clipboard__history_hide_on_paste")
        val historyHideOnNextTextField = nativeBoolean(key = "clipboard__history_hide_on_next_text_field")
        val clearPrimaryClipAffectsHistoryIfUnpinned = nativeBoolean(key = "clipboard__clear_primary_clip_affects_history_if_unpinned")
    }

    val correction = Correction()
    inner class Correction {
        val autoCapitalization = nativeBoolean(key = "correction__auto_capitalization")
        val autoSpacePunctuation = nativeBoolean(key = "correction__auto_space_punctuation")
        val doubleSpacePeriod = nativeBoolean(key = "correction__double_space_period")
        val rememberCapsLockState = nativeBoolean(key = "correction__remember_caps_lock_state")
    }

    val dictionary = Dictionary()
    inner class Dictionary {
        val enableSystemUserDictionary = nativeBoolean(key = "suggestion__enable_system_user_dictionary")
        val enableFlorisUserDictionary = nativeBoolean(key = "suggestion__enable_floris_user_dictionary")
    }

    val emoji = Emoji()
    inner class Emoji {
        val preferredSkinTone = nativeEnum<EmojiSkinTone>(key = "emoji__preferred_skin_tone")
        val preferredHairStyle = nativeEnum<EmojiHairStyle>(key = "emoji__preferred_hair_style")
        val historyEnabled = nativeBoolean(key = "emoji__history_enabled")
        val historyData = custom(key = "emoji__history_data", default = SettingsDefaults.history("emoji__history_data"), serializer = EmojiHistory.Serializer)
        val historyPinnedUpdateStrategy = nativeEnum<EmojiHistory.UpdateStrategy>(key = "emoji__history_pinned_update_strategy")
        val historyPinnedMaxSize = nativeInt(key = "emoji__history_pinned_max_size")
        val historyRecentUpdateStrategy = nativeEnum<EmojiHistory.UpdateStrategy>(key = "emoji__history_recent_update_strategy")
        val historyRecentMaxSize = nativeInt(key = "emoji__history_recent_max_size")
        val suggestionEnabled = nativeBoolean(key = "emoji__suggestion_enabled")
        val suggestionType = nativeEnum<EmojiSuggestionType>(key = "emoji__suggestion_type")
        val suggestionUpdateHistory = nativeBoolean(key = "emoji__suggestion_update_history")
        val suggestionCandidateShowName = nativeBoolean(key = "emoji__suggestion_candidate_show_name")
        val suggestionQueryMinLength = nativeInt(key = "emoji__suggestion_query_min_length")
        val suggestionCandidateMaxCount = nativeInt(key = "emoji__suggestion_candidate_max_count")
    }

    val gestures = Gestures()
    inner class Gestures {
        val swipeUp = nativeEnum<SwipeAction>(key = "gestures__swipe_up")
        val swipeDown = nativeEnum<SwipeAction>(key = "gestures__swipe_down")
        val swipeLeft = nativeEnum<SwipeAction>(key = "gestures__swipe_left")
        val swipeRight = nativeEnum<SwipeAction>(key = "gestures__swipe_right")
        val spaceBarSwipeUp = nativeEnum<SwipeAction>(key = "gestures__space_bar_swipe_up")
        val spaceBarSwipeLeft = nativeEnum<SwipeAction>(key = "gestures__space_bar_swipe_left")
        val spaceBarSwipeRight = nativeEnum<SwipeAction>(key = "gestures__space_bar_swipe_right")
        val spaceBarLongPress = nativeEnum<SwipeAction>(key = "gestures__space_bar_long_press")
        val deleteKeySwipeLeft = nativeEnum<SwipeAction>(key = "gestures__delete_key_swipe_left")
        val deleteKeyLongPress = nativeEnum<SwipeAction>(key = "gestures__delete_key_long_press")
        val swipeDistanceThreshold = nativeInt(key = "gestures__swipe_distance_threshold")
        val swipeVelocityThreshold = nativeInt(key = "gestures__swipe_velocity_threshold")
    }

    val glide = Glide()
    inner class Glide {
        val enabled = nativeBoolean(key = "glide__enabled")
        val showTrail = nativeBoolean(key = "glide__show_trail")
        val trailDuration = nativeInt(key = "glide__trail_fade_duration")
        val showPreview = nativeBoolean(key = "glide__show_preview")
        val previewRefreshDelay = nativeInt(key = "glide__preview_refresh_delay")
        val immediateBackspaceDeletesWord = nativeBoolean(key = "glide__immediate_backspace_deletes_word")
    }

    val inputFeedback = InputFeedback()
    inner class InputFeedback {
        val audioEnabled = nativeBoolean(key = "input_feedback__audio_enabled")
        val audioActivationMode = nativeEnum<InputFeedbackActivationMode>(key = "input_feedback__audio_activation_mode")
        val audioVolume = nativeInt(key = "input_feedback__audio_volume")
        val audioFeatKeyPress = nativeBoolean(key = "input_feedback__audio_feat_key_press")
        val audioFeatKeyLongPress = nativeBoolean(key = "input_feedback__audio_feat_key_long_press")
        val audioFeatKeyRepeatedAction = nativeBoolean(key = "input_feedback__audio_feat_key_repeated_action")
        val audioFeatGestureSwipe = nativeBoolean(key = "input_feedback__audio_feat_gesture_swipe")
        val audioFeatGestureMovingSwipe = nativeBoolean(key = "input_feedback__audio_feat_gesture_moving_swipe")

        val hapticEnabled = nativeBoolean(key = "input_feedback__haptic_enabled")
        val hapticActivationMode = nativeEnum<InputFeedbackActivationMode>(key = "input_feedback__haptic_activation_mode")
        val hapticVibrationMode = nativeEnum<HapticVibrationMode>(key = "input_feedback__haptic_vibration_mode")
        val hapticVibrationDuration = nativeInt(key = "input_feedback__haptic_vibration_duration")
        val hapticVibrationStrength = nativeInt(key = "input_feedback__haptic_vibration_strength")
        val hapticFeatKeyPress = nativeBoolean(key = "input_feedback__haptic_feat_key_press")
        val hapticFeatKeyLongPress = nativeBoolean(key = "input_feedback__haptic_feat_key_long_press")
        val hapticFeatKeyRepeatedAction = nativeBoolean(key = "input_feedback__haptic_feat_key_repeated_action")
        val hapticFeatGestureSwipe = nativeBoolean(key = "input_feedback__haptic_feat_gesture_swipe")
        val hapticFeatGestureMovingSwipe = nativeBoolean(key = "input_feedback__haptic_feat_gesture_moving_swipe")
    }

    val internal = Internal()
    inner class Internal {
        val homeIsBetaToolboxCollapsed = nativeBoolean(key = "internal__home_is_beta_toolbox_collapsed_040a01")
        val isImeSetUp = nativeBoolean(key = "internal__is_ime_set_up")
        val versionOnInstall = nativeString(key = "internal__version_on_install")
        val versionLastUse = nativeString(key = "internal__version_last_use")
        val versionLastChangelog = nativeString(key = "internal__version_last_changelog")
        val notificationPermissionState = nativeEnum<NotificationPermissionState>(key = "internal__notification_permission_state")
    }

    val keyboard = Keyboard()
    inner class Keyboard {
        val rustRendererPrototype = nativeBoolean(key = "keyboard__rust_renderer_prototype")
        val numberRow = nativeBoolean(key = "keyboard__number_row")
        val hintedNumberRowEnabled = nativeBoolean(key = "keyboard__hinted_number_row_enabled")
        val hintedNumberRowMode = nativeEnum<KeyHintMode>(key = "keyboard__hinted_number_row_mode")
        val hintedSymbolsEnabled = nativeBoolean(key = "keyboard__hinted_symbols_enabled")
        val hintedSymbolsMode = nativeEnum<KeyHintMode>(key = "keyboard__hinted_symbols_mode")
        // Off so the space bar is wider: emoji has its own button in the suggestion bar instead
        // (see Smartbar.kt). Turning it back on brings back the emoji/language key beside the space bar.
        val utilityKeyEnabled = nativeBoolean(key = "keyboard__utility_key_enabled")
        val utilityKeyAction = nativeEnum<UtilityKeyAction>(key = "keyboard__utility_key_action")
        val spaceBarMode = nativeEnum<SpaceBarMode>(key = "keyboard__space_bar_display_mode")
        val capitalizationBehavior = nativeEnum<CapitalizationBehavior>(key = "keyboard__capitalization_behavior")
        val fontSizeMultiplierPortrait = nativeInt(key = "keyboard__font_size_multiplier_portrait")
        val fontSizeMultiplierLandscape = nativeInt(key = "keyboard__font_size_multiplier_landscape")
        val oneHandedMode = nativeEnum<OneHandedMode>(key = "keyboard__one_handed_mode")
        val oneHandedModeEnabled = nativeBoolean(key = "keyboard__one_handed_mode_enabled")
        val oneHandedModeScaleFactor = nativeInt(key = "keyboard__one_handed_mode_scale_factor")
        val landscapeInputUiMode = nativeEnum<LandscapeInputUiMode>(key = "keyboard__landscape_input_ui_mode")
        val heightFactorPortrait = nativeInt(key = "keyboard__height_factor_portrait")
        val heightFactorLandscape = nativeInt(key = "keyboard__height_factor_landscape")
        val keySpacingVertical = nativeFloat(key = "keyboard__key_spacing_vertical")
        val keySpacingHorizontal = nativeFloat(key = "keyboard__key_spacing_horizontal")
        val bottomOffsetPortrait = nativeInt(key = "keyboard__bottom_offset_portrait")
        val bottomOffsetLandscape = nativeInt(key = "keyboard__bottom_offset_landscape")
        val popupEnabled = nativeBoolean(key = "keyboard__popup_enabled")
        val mergeHintPopupsEnabled = nativeBoolean(key = "keyboard__merge_hint_popups_enabled")
        val longPressDelay = nativeInt(key = "keyboard__long_press_delay")
        val spaceBarSwitchesToCharacters = nativeBoolean(key = "keyboard__space_bar_switches_to_characters")
        val incognitoDisplayMode = nativeEnum<IncognitoDisplayMode>(key = "keyboard__incognito_indicator")

        fun keyHintConfiguration(): KeyHintConfiguration {
            val modes = com.jtn0123.audiowhisper.rust.SettingsRules.call(29,
                hintedNumberRowEnabled.get().toString(), hintedNumberRowMode.get().name,
                hintedSymbolsEnabled.get().toString(), hintedSymbolsMode.get().name)
            return KeyHintConfiguration(KeyHintMode.valueOf(modes[0]), KeyHintMode.valueOf(modes[1]), mergeHintPopupsEnabled.get())
        }

        @Composable
        fun fontSizeMultiplier(): Float {
            val configuration = LocalConfiguration.current
            val oneHandedModeEnabled by oneHandedModeEnabled.observeAsState()
            val oneHandedModeFactor by oneHandedModeScaleFactor.observeAsState()
            val fontSizeMultiplierBase by if (configuration.isOrientationPortrait()) {
                fontSizeMultiplierPortrait
            } else {
                fontSizeMultiplierLandscape
            }.observeAsState()
            return com.jtn0123.audiowhisper.rust.ImeChromeRules.fontMultiplier(intArrayOf(
                fontSizeMultiplierBase, oneHandedModeFactor, if (oneHandedModeEnabled) 1 else 0,
                if (configuration.isOrientationPortrait()) 1 else 0))
        }
    }

    val localization = Localization()
    inner class Localization {
        val displayLanguageNamesIn = nativeEnum<DisplayLanguageNamesIn>(key = "localization__display_language_names_in")
        val displayKeyboardLabelsInSubtypeLanguage = nativeBoolean(key = "localization__display_keyboard_labels_in_subtype_language")
        val activeSubtypeId = nativeLong(key = "localization__active_subtype_id")
        val subtypes = nativeString(key = "localization__subtypes")
    }

    val other = Other()
    inner class Other {
        val rustSettingsPrototype = nativeBoolean(key = "other__rust_settings_prototype")

        val settingsTheme = nativeEnum<AppTheme>(key = "other__settings_theme")
        val accentColor = custom(key = "other__accent_color", default = SettingsDefaults.color("other__accent_color"), serializer = ColorPreferenceSerializer)
    }

    val physicalKeyboard = PhysicalKeyboard()
    inner class PhysicalKeyboard {
        val showOnScreenKeyboard = nativeBoolean(key = "physical_keyboard__show_on_screen_keyboard")
    }

    val smartbar = Smartbar()
    inner class Smartbar {
        val enabled = nativeBoolean(key = "smartbar__enabled")
        val layout = nativeEnum<SmartbarLayout>(key = "smartbar__layout")
        val actionArrangement = custom(key = "smartbar__action_arrangement", default = SettingsDefaults.arrangement("smartbar__action_arrangement"), serializer = QuickActionArrangement.Serializer)
        // Flipped so the pinned mic (the sticky action) sits at the left end of the suggestion bar and
        // the quick-actions arrow at the right.
        val flipToggles = nativeBoolean(key = "smartbar__flip_toggles")
        // AudioWhisper: the speaker beside the mic that reads the screen aloud.
        val showReadAloud = nativeBoolean(key = "smartbar__show_read_aloud")
        val sharedActionsExpanded = nativeBoolean(key = "smartbar__shared_actions_expanded")
        val sharedActionsExpandWithAnimation = nativeBoolean(key = "smartbar__shared_actions_expand_with_animation")
        val extendedActionsExpanded = nativeBoolean(key = "smartbar__extended_actions_expanded")
        val extendedActionsPlacement = nativeEnum<ExtendedActionsPlacement>(key = "smartbar__extended_actions_placement")
    }

    // AudioWhisper: how dictated phrases are tidied before they are typed, by the Mac app's rules.
    val dictation = DictationText()
    inner class DictationText {
        // On by default, as on the Mac.
        val removeFillers = nativeBoolean(key = "dictation__remove_fillers")
        // One entry per line, as on the Mac: a word to spell exactly, or "heard -> written".
        val vocabulary = nativeString(key = "dictation__vocabulary")
    }

    val spelling = Spelling()
    inner class Spelling {
        val languageMode = nativeEnum<SpellingLanguageMode>(key = "spelling__language_mode")
        val useContacts = nativeBoolean(key = "spelling__use_contacts")
        val useUdmEntries = nativeBoolean(key = "spelling__use_udm_entries")
    }

    val suggestion = Suggestion()
    inner class Suggestion {
        val api30InlineSuggestionsEnabled = nativeBoolean(key = "suggestion__api30_inline_suggestions_enabled")
        // Enabled by default with the bundled Latin dictionary.
        val enabled = nativeBoolean(key = "suggestion__enabled")
        // Three equal slots keep suggestions clear of the pinned emoji button.
        val displayMode = nativeEnum<CandidatesDisplayMode>(key = "suggestion__display_mode")
        val blockPossiblyOffensive = nativeBoolean(key = "suggestion__block_possibly_offensive")
        val incognitoMode = nativeEnum<IncognitoMode>(key = "suggestion__incognito_mode")
        // AudioWhisper: words and phrases learned from typing (never in incognito), see WordLearner.
        val learnFromTyping = nativeBoolean(key = "suggestion__learn_from_typing")
        // Internal pref
        val forceIncognitoModeFromDynamic = nativeBoolean(key = "suggestion__force_incognito_mode_from_dynamic")
    }

    val theme = Theme()
    inner class Theme {
        val mode = nativeEnum<ThemeMode>(key = "theme__mode")
        val dayThemeId = custom(key = "theme__day_theme_id", default = SettingsDefaults.component("theme__day_theme_id"), serializer = com.jtn0123.audiowhisper.settings.SettingsComponentSerializer)
        val nightThemeId = custom(key = "theme__night_theme_id", default = SettingsDefaults.component("theme__night_theme_id"), serializer = com.jtn0123.audiowhisper.settings.SettingsComponentSerializer)
        val accentColor = custom(key = "theme__accent_color", default = SettingsDefaults.color("theme__accent_color"), serializer = ColorPreferenceSerializer)
        val sunriseTime = custom(key = "theme__sunrise_time", default = SettingsDefaults.time("theme__sunrise_time"), serializer = com.jtn0123.audiowhisper.settings.SettingsTimeSerializer)
        val sunsetTime = custom(key = "theme__sunset_time", default = SettingsDefaults.time("theme__sunset_time"), serializer = com.jtn0123.audiowhisper.settings.SettingsTimeSerializer)
        val editorColorRepresentation = nativeEnum<ColorRepresentation>(key = "theme__editor_color_representation")
        val editorDisplayKbdAfterDialogs = nativeEnum<DisplayKbdAfterDialogs>(key = "theme__editor_display_kbd_after_dialogs")
        val editorLevel = nativeEnum<SnyggLevel>(key = "theme__editor_level")
    }

    override fun migrate(entry: PreferenceMigrationEntry) = migrateAppPreference(entry)
}
