package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import com.jtn0123.audiowhisper.rust.*
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import com.materialkolor.dynamicColorScheme
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentEditor
import org.florisboard.lib.color.ColorPalette
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.color.getColor
import kotlin.io.path.createTempDirectory
import kotlin.test.*

/**
 * The theme editor's dynamic colour swatches: the report carries the palette ProvideSnyggTheme gives the keyboard for
 * the component, the accent's Material You scheme under the component's own flags, as materialkolor makes it; and the
 * accent is the keyboard's, the system's when the user chose none.
 */
class SettingsThemeEditorPaletteMatchTest {
    private val accent = Color(0xFF00897B)

    /** The colours ProvideSnyggTheme's `dynamicColorScheme` call gives a theme with [flags], as unsigned ARGB. */
    private fun keyboard(flags: MaterialYouFlags, isDark: Boolean): List<String> {
        val scheme = dynamicColorScheme(primary = accent, isDark = isDark, style = flags.paletteStyle,
            contrastLevel = flags.contrastLevel.value, specVersion = flags.specVersion)
        return ColorPalette.colorNames.map { scheme.getColor(it).toArgb().toUInt().toString() }
    }
    private fun report(flags: MaterialYouFlags): Array<String> {
        val theme = ThemeExtensionComponentEditor(id = "sunny", label = "Sunny", materialYouFlags = flags)
        val dir = createTempDirectory("theme-editor-palette").toFile()
        try {
            return themeEditorReport(dir, "local.my.themes", theme, emptyList(), ThemeEditorNatives.UUID, "0", accent)
        } finally {
            dir.deleteRecursively()
        }
    }

    @Test fun theReportCarriesTheKeyboardsPaletteUnderTheComponentsFlags() {
        val vibrant = MaterialYouFlags(PaletteStyle.Vibrant, Contrast.entries.last())
        for (flags in listOf(MaterialYouFlags(), vibrant)) {
            val r = report(flags)
            assertEquals(ColorPalette.colorNames, r[SettingsUiThemeEditor.COLOR_NAMES].split("/"))
            assertEquals(keyboard(flags, isDark = false), r[SettingsUiThemeEditor.LIGHT_COLORS].split("/"), "$flags light")
            assertEquals(keyboard(flags, isDark = true), r[SettingsUiThemeEditor.DARK_COLORS].split("/"), "$flags dark")
        }
        val light = SettingsUiThemeEditor.LIGHT_COLORS
        assertNotEquals(report(MaterialYouFlags())[light], report(vibrant)[light], "the component's flags, not materialkolor's defaults")
        assertNotEquals(report(vibrant)[light], report(vibrant)[SettingsUiThemeEditor.DARK_COLORS])
    }

    @Test fun theAccentIsTheChosenOneOrTheSystemsWhenNoneIs() {
        assertEquals(accent, keyboardAccent(accent) { Color.Red })
        assertEquals(Color.Red, keyboardAccent(Color.Unspecified) { Color.Red })
    }
}
