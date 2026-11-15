package com.jtn0123.audiowhisper.settingsui

/**
 * Frozen InputFeedbackScreen, SmartbarScreen, GesturesScreen, ClipboardScreen and MediaScreen
 * decisions, transcribed from their Compose sources and EnumDisplayEntries.kt at 33f903a2 (the
 * row order, each enabledIf/visibleIf lambda, each list's entries and each preview callback).
 * No production Rust calls.
 */
internal object ReferenceGroupOneScreens {
    /** What an enabledIf/visibleIf lambda reads: stored preferences and the system vibrator. */
    class Facts(val p: Map<String, String>, val vibrator: Boolean, val amplitude: Boolean) {
        fun on(key: String) = p[key] == "true"
        fun off(key: String) = p[key] == "false"
        fun eq(key: String, value: String) = p[key] == value
    }
    data class Row(
        val key: String,
        val kind: String,
        val switch: String = "",
        val second: String = "",
        val choices: List<String> = emptyList(),
        /** onPreviewSelectedValue vibrates (InputFeedbackScreen's haptic sliders). */
        val preview: Boolean = false,
        /** DeleteEmojiHistoryConfirmDialog's `pinned` flag, as the effect names it. */
        val confirm: String = "",
        val enabled: (Facts) -> Boolean = { true },
        val visible: (Facts) -> Boolean = { true },
    )
    /** A screen and the preferences its lambdas read, with the values worth enumerating. */
    data class Screen(val route: String, val rows: List<Row>, val inputs: Map<String, List<String>>)

    private val bool = listOf("false", "true")
    private val activation = listOf("RESPECT_SYSTEM_SETTINGS", "IGNORE_SYSTEM_SETTINGS")
    private val swipe = listOf("NO_ACTION", "CYCLE_TO_PREVIOUS_KEYBOARD_MODE", "CYCLE_TO_NEXT_KEYBOARD_MODE", "DELETE_WORD",
        "HIDE_KEYBOARD", "INSERT_SPACE", "MOVE_CURSOR_UP", "MOVE_CURSOR_DOWN", "MOVE_CURSOR_LEFT", "MOVE_CURSOR_RIGHT",
        "MOVE_CURSOR_START_OF_LINE", "MOVE_CURSOR_END_OF_LINE", "MOVE_CURSOR_START_OF_PAGE", "MOVE_CURSOR_END_OF_PAGE", "SHIFT",
        "REDO", "UNDO", "SWITCH_TO_CLIPBOARD_CONTEXT", "SWITCH_TO_MEDIA_CONTEXT", "SHOW_INPUT_METHOD_PICKER",
        "SHOW_SUBTYPE_PICKER", "SWITCH_TO_PREV_SUBTYPE", "SWITCH_TO_NEXT_SUBTYPE", "SWITCH_TO_PREV_KEYBOARD",
        "TOGGLE_SMARTBAR_VISIBILITY")
    private val sync = listOf("NO_EVENTS", "ONLY_CLEAR_EVENTS", "ONLY_SET_EVENTS", "ALL_EVENTS")
    private val strategy = listOf("AUTO_SORT_PREPEND", "AUTO_SORT_APPEND", "MANUAL_SORT_PREPEND", "MANUAL_SORT_APPEND")
    private val layouts = listOf("SUGGESTIONS_ONLY", "ACTIONS_ONLY", "SUGGESTIONS_ACTIONS_SHARED", "SUGGESTIONS_ACTIONS_EXTENDED")

    private fun feats(kind: String) = listOf("key_press", "key_long_press", "key_repeated_action", "gesture_swipe", "gesture_moving_swipe")
        .map { Row("input_feedback__${kind}_feat_$it", "toggle", enabled = { f -> f.on("input_feedback__${kind}_enabled") }) }
    private fun hapticDirect(f: Facts) = f.on("input_feedback__haptic_enabled") &&
        f.eq("input_feedback__haptic_vibration_mode", "USE_VIBRATOR_DIRECTLY") && f.vibrator

    val inputFeedback = Screen("settings/keyboard/input-feedback", listOf(
        Row("pref__input_feedback__group_audio__label", "heading"),
        Row("input_feedback__audio_activation_mode", "choice", "input_feedback__audio_enabled", choices = activation),
        Row("input_feedback__audio_volume", "slider", enabled = { it.on("input_feedback__audio_enabled") }),
    ) + feats("audio") + listOf(
        Row("pref__input_feedback__group_haptic__label", "heading"),
        Row("input_feedback__haptic_activation_mode", "choice", "input_feedback__haptic_enabled", choices = activation),
        Row("input_feedback__haptic_vibration_mode", "choice", choices = listOf("USE_VIBRATOR_DIRECTLY", "USE_HAPTIC_FEEDBACK_INTERFACE"),
            enabled = { it.on("input_feedback__haptic_enabled") }),
        Row("input_feedback__haptic_vibration_duration", "slider", preview = true, enabled = ::hapticDirect),
        Row("input_feedback__haptic_vibration_strength", "slider", preview = true, enabled = { hapticDirect(it) && it.amplitude }),
    ) + feats("haptic"), mapOf("input_feedback__audio_enabled" to bool, "input_feedback__haptic_enabled" to bool,
        "input_feedback__haptic_vibration_mode" to listOf("USE_VIBRATOR_DIRECTLY", "USE_HAPTIC_FEEDBACK_INTERFACE")))

    private fun smartbarOn(f: Facts) = f.on("smartbar__enabled")
    private fun actions(f: Facts) = f.eq("smartbar__layout", "SUGGESTIONS_ACTIONS_SHARED") || f.eq("smartbar__layout", "SUGGESTIONS_ACTIONS_EXTENDED")
    val smartbar = Screen("settings/smartbar", listOf(
        Row("smartbar__enabled", "toggle"),
        Row("smartbar__layout", "choice", choices = layouts, enabled = ::smartbarOn),
        Row("pref__smartbar__group_layout_specific__label", "heading"),
        Row("suggestion__display_mode", "choice", choices = listOf("CLASSIC", "DYNAMIC", "DYNAMIC_SCROLLABLE"), enabled = ::smartbarOn,
            visible = { !it.eq("smartbar__layout", "ACTIONS_ONLY") }),
        Row("smartbar__show_read_aloud", "toggle", enabled = ::smartbarOn, visible = ::actions),
        Row("smartbar__flip_toggles", "toggle", enabled = ::smartbarOn, visible = ::actions),
        Row("pref__smartbar__shared_actions_auto_expand_collapse__label", "fixed", enabled = { false },
            visible = { it.eq("smartbar__layout", "SUGGESTIONS_ACTIONS_SHARED") }),
        Row("smartbar__extended_actions_placement", "choice", choices = listOf("ABOVE_CANDIDATES", "BELOW_CANDIDATES", "OVERLAY_APP_UI"),
            enabled = ::smartbarOn, visible = { it.eq("smartbar__layout", "SUGGESTIONS_ACTIONS_EXTENDED") }),
    ), mapOf("smartbar__enabled" to bool, "smartbar__layout" to layouts))

    private fun glide(f: Facts) = f.on("glide__enabled")
    val gestures = Screen("settings/gestures", listOf(
        Row("pref__glide__title", "heading"),
        Row("glide__enabled", "toggle"),
        Row("glide__show_trail", "toggle", enabled = ::glide),
        Row("glide__trail_fade_duration", "slider", enabled = { glide(it) && it.on("glide__show_trail") }),
        Row("glide__show_preview", "toggle", enabled = ::glide),
        Row("glide__preview_refresh_delay", "slider", enabled = { glide(it) && it.on("glide__show_preview") }),
        Row("glide__immediate_backspace_deletes_word", "toggle", enabled = ::glide),
        Row("pref__gestures__general_title", "heading"),
    ) + listOf("up", "down", "left", "right").map { Row("gestures__swipe_$it", "choice", choices = swipe, enabled = { f -> f.off("glide__enabled") }) } +
        listOf(Row("pref__gestures__space_bar_title", "heading")) +
        listOf("swipe_up", "swipe_left", "swipe_right", "long_press").map { Row("gestures__space_bar_$it", "choice", choices = swipe) } + listOf(
        Row("pref__gestures__other_title", "heading"),
        Row("gestures__delete_key_swipe_left", "choice", choices = listOf("NO_ACTION", "DELETE_CHARACTERS_PRECISELY", "DELETE_WORD",
            "DELETE_WORDS_PRECISELY", "SELECT_CHARACTERS_PRECISELY", "SELECT_WORDS_PRECISELY")),
        Row("gestures__delete_key_long_press", "choice", choices = listOf("DELETE_CHARACTER", "DELETE_WORD")),
        Row("gestures__swipe_velocity_threshold", "slider"),
        Row("gestures__swipe_distance_threshold", "slider"),
    ), mapOf("glide__enabled" to bool, "glide__show_trail" to bool, "glide__show_preview" to bool))

    private fun history(f: Facts) = f.on("clipboard__history_enabled")
    val clipboard = Screen("settings/clipboard", listOf(
        Row("clipboard__use_internal_clipboard", "toggle"),
        Row("clipboard__sync_to_floris", "choice", choices = sync, enabled = { it.on("clipboard__use_internal_clipboard") }),
        Row("clipboard__sync_to_system", "choice", choices = sync, enabled = { it.on("clipboard__use_internal_clipboard") }),
        Row("pref__clipboard__group_clipboard_suggestion__label", "heading"),
        Row("clipboard__suggestion_enabled", "toggle"),
        Row("clipboard__suggestion_timeout", "slider", enabled = { it.on("clipboard__suggestion_enabled") }),
        Row("pref__clipboard__group_clipboard_history__label", "heading"),
        Row("clipboard__history_enabled", "toggle"),
        Row("clipboard__history_num_grid_columns_portrait", "slider", second = "clipboard__history_num_grid_columns_landscape", enabled = ::history),
        Row("clipboard__history_auto_clean_old_enabled", "toggle", enabled = ::history),
        Row("clipboard__history_auto_clean_old_after", "slider", enabled = { history(it) && it.on("clipboard__history_auto_clean_old_enabled") }),
        Row("clipboard__history_auto_clean_sensitive_enabled", "toggle", enabled = ::history),
        Row("clipboard__history_auto_clean_sensitive_after", "slider",
            enabled = { history(it) && it.on("clipboard__history_auto_clean_sensitive_enabled") }),
        Row("clipboard__history_size_limit_enabled", "toggle", enabled = ::history),
        Row("clipboard__history_size_limit", "slider", enabled = { history(it) && it.on("clipboard__history_size_limit_enabled") }),
        Row("clipboard__history_hide_on_paste", "toggle", enabled = ::history),
        Row("clipboard__history_hide_on_next_text_field", "toggle", enabled = ::history),
        Row("clipboard__clear_primary_clip_affects_history_if_unpinned", "toggle", enabled = ::history),
    ), listOf("use_internal_clipboard", "suggestion_enabled", "history_enabled", "history_auto_clean_old_enabled",
        "history_auto_clean_sensitive_enabled", "history_size_limit_enabled").associate { "clipboard__$it" to bool })

    private fun emojiHistory(f: Facts) = f.on("emoji__history_enabled")
    private fun suggestion(f: Facts) = f.on("emoji__suggestion_enabled")
    val media = Screen("settings/media", listOf(
        Row("emoji__preferred_skin_tone", "choice", choices = listOf("DEFAULT", "LIGHT_SKIN_TONE", "MEDIUM_LIGHT_SKIN_TONE",
            "MEDIUM_SKIN_TONE", "MEDIUM_DARK_SKIN_TONE", "DARK_SKIN_TONE")),
        Row("prefs__media__emoji_history__title", "heading"),
        Row("emoji__history_enabled", "toggle"),
        Row("emoji__history_pinned_update_strategy", "choice", choices = strategy, enabled = ::emojiHistory),
        Row("emoji__history_recent_update_strategy", "choice", choices = strategy, enabled = ::emojiHistory),
        Row("emoji__history_pinned_max_size", "slider", second = "emoji__history_recent_max_size", enabled = ::emojiHistory),
        Row("prefs__media__emoji_history_pinned_reset", "confirm", confirm = "pinned", enabled = ::emojiHistory),
        Row("prefs__media__emoji_history_reset", "confirm", confirm = "recent", enabled = ::emojiHistory),
        Row("prefs__media__emoji_suggestion__title", "heading"),
        Row("emoji__suggestion_enabled", "toggle"),
        Row("emoji__suggestion_type", "choice", choices = listOf("LEADING_COLON", "INLINE_TEXT"), enabled = ::suggestion),
        Row("emoji__suggestion_update_history", "toggle", enabled = { suggestion(it) && emojiHistory(it) }),
        Row("emoji__suggestion_candidate_show_name", "toggle", enabled = ::suggestion),
        Row("emoji__suggestion_query_min_length", "slider", enabled = ::suggestion),
        Row("emoji__suggestion_candidate_max_count", "slider", enabled = ::suggestion),
    ), mapOf("emoji__history_enabled" to bool, "emoji__suggestion_enabled" to bool))

    val screens = listOf(inputFeedback, smartbar, gestures, clipboard, media)
    /** The plurals each DialogSliderPreference's valueLabel uses (ClipboardScreen, MediaScreen). */
    val plurals = mapOf(
        "clipboard__history_auto_clean_old_after" to "unit__minutes__written",
        "clipboard__history_auto_clean_sensitive_after" to "unit__seconds__written",
        "clipboard__history_size_limit" to "unit__items__written",
        "emoji__history_pinned_max_size" to "unit__items__written",
        "emoji__history_recent_max_size" to "unit__items__written",
        "emoji__suggestion_query_min_length" to "unit__characters__written",
        "emoji__suggestion_candidate_max_count" to "unit__candidates__written",
    )
}
