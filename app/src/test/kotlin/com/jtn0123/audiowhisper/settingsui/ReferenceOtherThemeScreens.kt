package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.graphics.Color
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import dev.patrickgold.jetpref.material.ui.JetPrefColorPickerState
import dev.patrickgold.jetpref.material.ui.parseOrNull

/**
 * Frozen OtherScreen decisions, transcribed from its Compose source at b0d4a9a4f: the row order,
 * each list's entries (`enumDisplayEntriesOf`), each summary, and where each row goes. The colour
 * row is Jetpref's ColorPickerPreference with `enableAdvancedLayout`, whose dialog the match test
 * holds against Jetpref and Compose themselves. No production code.
 */
internal object ReferenceOtherThemeScreens {
    /** What the screens read: the stored preferences, as their serializers write them. */
    class Facts(val p: Map<String, String>)

    /**
     * A row in source order. `kind` is choice, color, heading or route. `labels` are a list's
     * entry labels in order; `summary` is a fixed summary's string key.
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
    )
    data class Screen(val route: String, val preview: Boolean, val rows: List<Row>, val inputs: Map<String, List<String>>)

    /** `Color.Unspecified`, the stored form of "follow the system accent". */
    const val UNSPECIFIED = "0000000000000010"
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

    val screens = listOf(other)

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

    /** The row's summary as Compose shows it: a list's selected entry label, or the fixed summary. */
    fun summary(row: Row, f: Facts, s: (String) -> String): String = when (row.kind) {
        "choice" -> s(row.labels[row.choices.indexOf(f.p.getValue(row.key))])
        else -> if (row.summary.isEmpty()) "" else s(row.summary)
    }
}
