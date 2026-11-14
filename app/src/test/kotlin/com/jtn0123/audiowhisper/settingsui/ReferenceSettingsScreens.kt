package com.jtn0123.audiowhisper.settingsui

/** Frozen KeyboardScreen / HomeScreen decisions at 04186d1d; no production Rust calls. */
internal object ReferenceSettingsScreens {
    data class Row(val key: String, val kind: String, val switch: String = "", val second: String = "")
    val home = listOf("settings/localization", "settings/theme", "settings/keyboard", "settings/smartbar", "settings/dictation",
        "settings/typing", "settings/gestures", "settings/clipboard", "settings/media", "ext", "settings/other", "settings/about")
    val rows = listOf(
        Row("keyboard__rust_renderer_prototype", "toggle"), Row("keyboard__number_row", "toggle"),
        Row("keyboard__hinted_number_row_mode", "choice", "keyboard__hinted_number_row_enabled"),
        Row("keyboard__hinted_symbols_mode", "choice", "keyboard__hinted_symbols_enabled"),
        Row("keyboard__utility_key_enabled", "toggle"), Row("keyboard__utility_key_action", "choice"),
        Row("keyboard__space_bar_display_mode", "choice"), Row("keyboard__capitalization_behavior", "choice"),
        Row("keyboard__font_size_multiplier_portrait", "slider", second = "keyboard__font_size_multiplier_landscape"),
        Row("keyboard__incognito_indicator", "choice"), Row("pref__keyboard__group_layout__label", "heading"),
        Row("keyboard__one_handed_mode", "choice", "keyboard__one_handed_mode_enabled"),
        Row("keyboard__one_handed_mode_scale_factor", "slider"), Row("keyboard__landscape_input_ui_mode", "choice"),
        Row("keyboard__height_factor_portrait", "slider", second = "keyboard__height_factor_landscape"),
        Row("keyboard__key_spacing_vertical", "slider", second = "keyboard__key_spacing_horizontal"),
        Row("keyboard__bottom_offset_portrait", "slider", second = "keyboard__bottom_offset_landscape"),
        Row("pref__keyboard__group_keypress__label", "heading"), Row("settings__input_feedback__title", "route"),
        Row("keyboard__popup_enabled", "toggle"), Row("keyboard__merge_hint_popups_enabled", "toggle"),
        Row("keyboard__long_press_delay", "slider"), Row("keyboard__space_bar_switches_to_characters", "toggle"),
    )
    fun visible(i: Int, p: Map<String,String>) = rows[i].key != "keyboard__utility_key_action" || p["keyboard__utility_key_enabled"] == "true"
    fun enabled(i: Int, p: Map<String,String>) = when (rows[i].key) {
        "keyboard__hinted_number_row_mode" -> p["keyboard__number_row"] != "true"
        "keyboard__one_handed_mode_scale_factor" -> p["keyboard__one_handed_mode_enabled"] == "true"
        else -> true
    }
    fun choices(i: Int) = when (i) {
        2, 3 -> listOf("ACCENT_PRIORITY", "HINT_PRIORITY", "SMART_PRIORITY")
        5 -> listOf("SWITCH_TO_EMOJIS", "SWITCH_LANGUAGE", "SWITCH_KEYBOARD_APP", "DYNAMIC_SWITCH_LANGUAGE_EMOJIS")
        6 -> listOf("NOTHING", "CURRENT_LANGUAGE", "SPACE_BAR_KEY")
        7 -> listOf("CAPSLOCK_BY_DOUBLE_TAP", "CAPSLOCK_BY_CYCLE")
        9 -> listOf("REPLACE_SHARED_ACTIONS_TOGGLE", "DISPLAY_BEHIND_KEYBOARD")
        11 -> listOf("START", "END")
        13 -> listOf("NEVER_SHOW", "ALWAYS_SHOW", "DYNAMICALLY_SHOW")
        else -> error("not a list preference")
    }
    // Frozen DialogSliderPreference parameters before their stage-2 range lookup was introduced.
    val bounds = mapOf(8 to Triple(50f,150f,5f),12 to Triple(70f,90f,1f),14 to Triple(50f,150f,5f),
        15 to Triple(0f,10f,.5f),16 to Triple(0f,60f,1f),21 to Triple(100f,700f,10f))
}
