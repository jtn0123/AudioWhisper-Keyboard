package com.jtn0123.audiowhisper.render

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.isSpecified
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import com.jtn0123.audiowhisper.rust.ImeTheme
import com.jtn0123.audiowhisper.rust.ImeThemeAccent
import com.jtn0123.audiowhisper.rust.ImeThemeChip
import com.jtn0123.audiowhisper.rust.ImeThemeInline
import com.jtn0123.audiowhisper.rust.ImeThemeRule
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.theme.ThemeManager
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.themeManager
import dev.patrickgold.jetpref.datastore.model.observeAsState
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.color.getSystemAccent
import org.florisboard.lib.snygg.ui.SnyggPalette
import org.florisboard.lib.snygg.ui.snyggPalette

/** The accent a palette is made from, as Rust chooses it: the settings' [accent] if they name one, else [system]'s. */
internal fun themePrimary(accent: Color, system: () -> Color): Color {
    val specified = accent.isSpecified
    val row = ImeThemeAccent.row(
        ImeThemeAccent.SPECIFIED to WireFlag.encode(specified),
        ImeThemeAccent.PREFERRED to (if (specified) accent.toArgb() else 0).toString(),
        ImeThemeAccent.SYSTEM to (if (specified) 0 else system().toArgb()).toString(),
    )
    return Color(ImeTheme.rules(ImeThemeRule.PRIMARY, *row).single().toInt())
}

/** The palette a theme with Material You [flags] is drawn in, for the settings' [accent] or else the [system]'s. */
internal fun themePalette(accent: Color, system: () -> Color, flags: MaterialYouFlags): SnyggPalette =
    snyggPalette(themePrimary(accent, system), flags)

/** The inline autofill chips' background and foreground in [info] for the keyboard's [mode] and [shift], as Rust resolves them. */
internal fun inlineChipColors(info: ThemeManager.ThemeInfo, mode: String, shift: String, palette: SnyggPalette): Pair<Int, Int> {
    val row = ImeThemeInline.row(
        ImeThemeInline.STYLESHEET to info.stylesheetJson,
        ImeThemeInline.MODE to mode,
        ImeThemeInline.SHIFT to shift,
    )
    val chip = ImeTheme.rules(ImeThemeRule.INLINE, *row, *palette.names.toTypedArray(), *palette.colors.map { it.toString() }.toTypedArray())
    return chip[ImeThemeChip.BACKGROUND].toInt() to chip[ImeThemeChip.FOREGROUND].toInt()
}

/** What the renderer draws [info] in: its stylesheet, the [palette], the display's scale and the keyboard's state. */
internal fun rendererTheme(
    info: ThemeManager.ThemeInfo, palette: SnyggPalette, density: Float, fontScale: Float, multiplier: Float,
    mode: String = "characters", shift: String = "unshifted",
) = RendererTheme(info.stylesheetJson, palette.names, palette.colors, density, fontScale, multiplier, mode, shift)

/** The palette the active theme [info] is drawn in, made again only when the accent or the theme's flags change. */
@Composable
internal fun rememberThemePalette(info: ThemeManager.ThemeInfo): SnyggPalette {
    val prefs by FlorisPreferenceStore
    val accent by prefs.theme.accentColor.observeAsState()
    val system = getSystemAccent()
    val flags = info.config.materialYouFlags
    return remember(accent, system, flags) { themePalette(accent, { system }, flags) }
}

@Composable
internal fun rendererTheme(): RendererTheme {
    val keyboard by LocalContext.current.keyboardManager()
    val state by keyboard.activeState.collectAsState()
    val mode = state.keyboardMode.toString()
    val shift = state.inputShiftState.toString()
    val manager by LocalContext.current.themeManager()
    val active by manager.activeThemeInfo.collectAsState()
    val palette = rememberThemePalette(active)
    val density = LocalDensity.current
    val prefs by FlorisPreferenceStore
    val multiplier = prefs.keyboard.fontSizeMultiplier()
    return remember(active, palette, density, multiplier, mode, shift) {
        rendererTheme(active, palette, density.density, density.fontScale, multiplier, mode, shift)
    }
}
