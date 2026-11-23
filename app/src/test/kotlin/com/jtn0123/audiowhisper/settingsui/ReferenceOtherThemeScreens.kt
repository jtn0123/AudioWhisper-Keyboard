package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.graphics.Color
import dev.patrickgold.florisboard.app.FlorisPreferenceModelImpl
import dev.patrickgold.jetpref.datastore.model.LocalTime
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import dev.patrickgold.jetpref.material.ui.JetPrefColorPickerState
import dev.patrickgold.jetpref.material.ui.parseOrNull

/**
 * Frozen OtherScreen and ThemeScreen decisions, transcribed from their Compose sources at
 * b0d4a9a4f: the row order, each list's entries (`enumDisplayEntriesOf`), each summary, each
 * `enabledIf`, and where each row goes. The colour rows are Jetpref's ColorPickerPreference with
 * `enableAdvancedLayout`, whose dialog the match test holds against Jetpref and Compose themselves;
 * the time rows are Jetpref's LocalTimePickerPreference, whose summary is LocalTime's own
 * `stringRepresentation`. No production code.
 */
internal object ReferenceOtherThemeScreens {
    /**
     * What the screens read: the stored preferences, as their serializers write them; the theme
     * index's labels by id (`indexedThemeConfigs`); the system's 24-hour setting.
     */
    class Facts(val p: Map<String, String>, val labels: Map<String, String> = emptyMap(), val is24: Boolean = false)

    /**
     * A row in source order. `kind` is choice, color, heading, route, theme (a Preference opening
     * the theme manager), time or addons (AddonManagementReferenceBox, whose row node is its note).
     * `labels` are a list's entry labels in order; `summary` is a fixed summary's string key;
     * `extType` is an addon box's extension type title.
     */
    data class Row(
        val key: String,
        val kind: String,
        val choices: List<String> = emptyList(),
        val labels: List<String> = emptyList(),
        val summary: String = "",
        val route: String = "",
        val enabled: (Facts) -> Boolean = { true },
        /** The title's string key, when it is not the key itself. */
        val title: String = key,
        val extType: String = "",
    )
    /** `inputs` are stored values to combine; `labelSets` and `clocks` the facts to cross them with. */
    data class Screen(
        val route: String, val preview: Boolean, val rows: List<Row>, val inputs: Map<String, List<String>>,
        val labelSets: List<Map<String, String>> = listOf(emptyMap()), val clocks: List<Boolean> = listOf(false),
    )

    /** `Color.Unspecified`, the stored form of "follow the system accent". */
    const val UNSPECIFIED = "0000000000000010"
    /** FlorisBoard's `ColorMappings.colors` (Material's 500 shades), the Presets page in order. */
    val presets = listOf(
        0xFF4CAF50, 0xFFF44336, 0xFFE91E63, 0xFFFF2C93, 0xFF9C27B0, 0xFF673AB7, 0xFF3F51B5, 0xFF2196F3, 0xFF03A9F4,
        0xFF00BCD4, 0xFF009688, 0xFF8BC34A, 0xFFCDDC39, 0xFFFFEB3B, 0xFFFFC107, 0xFFFF9800, 0xFF795548, 0xFF607D8B,
        0xFF9E9E9E,
    ).map { Color(it) }
    /** The colours a stored accent can hold: none, a preset, a colour of the user's own. */
    val accents = listOf(UNSPECIFIED, "ff673ab700000000", "ff12345600000000")

    val other = Screen("settings/other", false, listOf(
        // EnumDisplayEntries: AppTheme has labels and no descriptions.
        Row("other__settings_theme", "choice", listOf("AUTO", "AUTO_AMOLED", "LIGHT", "DARK", "AMOLED_DARK"), listOf(
            "settings__system_default", "pref__other__settings_theme__auto_amoled", "pref__other__settings_theme__light",
            "pref__other__settings_theme__dark", "pref__other__settings_theme__amoled_dark"),
            title = "pref__other__settings_theme__label"),
        // ColorPickerPreference: no summary; the trailing swatch is drawn, not a node.
        Row("other__accent_color", "color", title = "pref__other__settings_accent_color__label"),
        Row("physical_keyboard__title", "route", route = "settings/other/physical-keyboard"),
        Row("backup_and_restore__title", "heading"),
        Row("backup_and_restore__back_up__title", "route", summary = "backup_and_restore__back_up__summary",
            route = "settings/other/backup"),
        Row("backup_and_restore__restore__title", "route", summary = "backup_and_restore__restore__summary",
            route = "settings/other/restore"),
    ), mapOf(
        "other__settings_theme" to listOf("AUTO", "AUTO_AMOLED", "LIGHT", "DARK", "AMOLED_DARK"),
        "other__accent_color" to accents,
    ))

    const val DAY = "org.florisboard.themes:floris_day"
    const val NIGHT = "org.florisboard.themes:floris_night"
    private fun mode(f: Facts) = f.p.getValue("theme__mode")

    val theme = Screen("settings/theme", true, listOf(
        Row("theme__mode", "choice", listOf("ALWAYS_DAY", "ALWAYS_NIGHT", "FOLLOW_SYSTEM", "FOLLOW_TIME"), listOf(
            "enum__theme_mode__always_day", "enum__theme_mode__always_night", "enum__theme_mode__follow_system",
            "enum__theme_mode__follow_time"), title = "pref__theme__mode__label"),
        Row("theme__day_theme_id", "theme", route = "settings/theme/manage/SELECT_DAY",
            enabled = { mode(it) != "ALWAYS_NIGHT" }, title = "pref__theme__day"),
        Row("theme__night_theme_id", "theme", route = "settings/theme/manage/SELECT_NIGHT",
            enabled = { mode(it) != "ALWAYS_DAY" }, title = "pref__theme__night"),
        Row("theme__sunrise_time", "time", enabled = { mode(it) == "FOLLOW_TIME" }, title = "pref__theme__sunrise_time__label"),
        Row("theme__sunset_time", "time", enabled = { mode(it) == "FOLLOW_TIME" }, title = "pref__theme__sunset_time__label"),
        Row("theme__accent_color", "color", title = "pref__theme__theme_accent_color__label"),
        Row("ext_theme", "addons", route = "ext/list/EXT_THEME?showUpdate=true",
            title = "ext__addon_management_box__addon_manager_info", extType = "ext__list__ext_theme"),
    ), mapOf(
        "theme__mode" to listOf("ALWAYS_DAY", "ALWAYS_NIGHT", "FOLLOW_SYSTEM", "FOLLOW_TIME"),
        // The defaults, a theme in the index, one that is not, and texts that are no id at all.
        "theme__day_theme_id" to listOf(DAY, "my.ext:sunny", "my.ext:gone", "no-colon"),
        "theme__night_theme_id" to listOf(NIGHT, "my.ext:sunny", "a:b:c"),
        "theme__sunrise_time" to listOf("06:00:00.000", "00:00:00.000", "12:30:59.999", "7:00"),
        "theme__sunset_time" to listOf("18:00:00.000", "23:59:00.000"),
    ), labelSets = listOf(emptyMap(), mapOf(DAY to "Floris Day", NIGHT to "Floris Night", "my.ext:sunny" to "Sunny")),
        clocks = listOf(false, true))

    val screens = listOf(other, theme)

    /**
     * The numbers Material's ClockFace reads out with a plural suffix: a 24-hour clock's 0 to 23
     * and the minutes, 0 to 55. A 12-hour clock's "o'clock" is a plain string.
     */
    val dialPlurals = setOf("settings__time_picker__hour_24h_suffix|0|23", "settings__time_picker__minute_suffix|0|55")

    private val model = FlorisPreferenceModelImpl()
    /** The preference as Jetpref reads it: its serializer's value, or the default when that fails. */
    private fun read(key: String, f: Facts): Any {
        val pref = model.declaredPreferenceEntries.values.first { it.key == key }
        return pref.serializer.deserialize(f.p.getValue(key)) ?: pref.default
    }

    /** AddonManagementReferenceBox's title and button text, `curlyFormat` filled in. */
    fun addonTitle(row: Row, s: (String) -> String) =
        s("ext__addon_management_box__managing_placeholder").replace("{extensions}", s(row.extType).lowercase())
    fun addonButton(row: Row, s: (String) -> String) =
        s("ext__addon_management_box__go_to_page").replace("{ext_home_title}", s(row.extType))

    /** JetPrefColorPicker's state; its `setColor` and `color` are Jetpref's and Compose's own maths. */
    class PickerState(c: Color) : JetPrefColorPickerState {
        override var hue = 0f
        override var saturation = 1f
        override var value = 1f
        override var alpha = 1f
        override var isEditorMode = false
        init { setColor(c) }
    }

    /**
     * ColorPickerPreference's dialog (`enableAdvancedLayout = true`, no alpha slider) as its
     * composable keeps it: `dialogValue`, `selectedPreset`, `advanced`, the picker state (created
     * once per opening, outside `key(advanced)`), and the field's representation (inside it, so a
     * page switch starts it over as HEX). `defaultColor` is the system accent.
     */
    class ColorDialog(stored: Color?, private val accent: Color, private val presets: List<Color>) {
        var value: Color = stored ?: accent
        var preset = presets.indexOf(value.copy(alpha = 1f))
        var advanced = !presets.contains(value.copy(alpha = 1f)) && value.copy(alpha = 1f) != accent
        val state = PickerState(value)
        var representation = ColorRepresentation.HEX
        val defaultTicked get() = value.copy(alpha = 1f) == accent
        val neutralLabel get() = if (advanced) "Presets" else "Custom"
        fun neutral() {
            advanced = !advanced
            representation = ColorRepresentation.HEX
            if (advanced) preset = -1
        }
        fun pick(j: Int) {
            preset = j
            value = presets[j].copy(alpha = value.alpha)
            state.setColor(value)
        }
        fun pickDefault() {
            preset = -1
            value = accent
            state.setColor(accent)
        }
        /** What the read-only field shows, and what the editor starts with. */
        fun shown() = representation.formatColor(state.color(), false)
        /** Apply changes: false (and the error) when the text is no colour. */
        fun apply(text: String): Boolean {
            val (color, representation) = Color.parseOrNull(text, false) ?: return false
            state.setColor(color)
            value = color
            this.representation = representation
            state.isEditorMode = false
            return true
        }
        /** OK: `colorOverride` stores the system accent itself as no colour. */
        fun stored(serialize: (Color) -> String) = if (value == accent) UNSPECIFIED else serialize(value)
    }

    /**
     * The row's summary as Compose shows it: a list's selected entry label, `getThemeLabel` (the
     * indexed label, else the id), LocalTime's `stringRepresentation`, or the fixed summary.
     */
    fun summary(row: Row, f: Facts, s: (String) -> String): String = when (row.kind) {
        "choice" -> s(row.labels[row.choices.indexOf(f.p.getValue(row.key))])
        "theme" -> read(row.key, f).toString().let { f.labels[it] ?: it }
        "time" -> (read(row.key, f) as LocalTime).stringRepresentation(f.is24)
        else -> if (row.summary.isEmpty()) "" else s(row.summary)
    }
}
