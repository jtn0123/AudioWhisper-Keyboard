package com.jtn0123.audiowhisper.render

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.isSpecified
import androidx.compose.ui.graphics.toArgb
import com.jtn0123.audiowhisper.rust.ImeTheme
import com.jtn0123.audiowhisper.rust.ImeThemeAccent
import com.jtn0123.audiowhisper.rust.ImeThemeChip
import com.jtn0123.audiowhisper.rust.ImeThemeInline
import com.jtn0123.audiowhisper.rust.ImeThemeRule
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.ime.theme.ThemeManager
import org.florisboard.lib.color.MaterialYouFlags
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

/** What the renderer draws in: a stylesheet, its palette, the display's scale and the keyboard's state. */
data class RendererTheme(val json: String, val paletteNames: List<String>, val paletteColors: List<Int>,
    val density: Float, val fontScale: Float, val fontMultiplier: Float, val mode: String = "characters", val shift: String = "unshifted")
