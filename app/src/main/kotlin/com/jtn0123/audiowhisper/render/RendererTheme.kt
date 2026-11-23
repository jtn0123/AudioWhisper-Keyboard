package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.ImeTheme
import com.jtn0123.audiowhisper.rust.ImeThemeAccent
import com.jtn0123.audiowhisper.rust.ImeThemeInline
import com.jtn0123.audiowhisper.rust.ImeThemeChip
import com.jtn0123.audiowhisper.rust.ImeThemeMaterialYou
import com.jtn0123.audiowhisper.rust.ImeThemeRule
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.ime.theme.ThemeManager
import org.florisboard.lib.color.MaterialYouFlags

/** A theme's palette as Rust makes it: the colour names, then each name's light colour, then each name's dark one. */
internal class ThemePalette(val names: List<String>, val colors: List<Int>)

/**
 * The palette a theme with Material You [flags] is drawn in, made in Rust (crates/palette) for the settings' [accent]
 * (ARGB, null when they name none) or else the [system]'s.
 */
internal fun themePalette(accent: Int?, system: Int, flags: MaterialYouFlags): ThemePalette {
    val answer = ImeTheme.rules(ImeThemeRule.PALETTE,
        *ImeThemeAccent.row(ImeThemeAccent.SPECIFIED to WireFlag.encode(accent != null),
            ImeThemeAccent.PREFERRED to "${accent ?: 0}", ImeThemeAccent.SYSTEM to "$system"),
        *ImeThemeMaterialYou.row(ImeThemeMaterialYou.STYLE to flags.paletteStyle.name,
            ImeThemeMaterialYou.CONTRAST to flags.contrastLevel.name, ImeThemeMaterialYou.SPEC to flags.specVersion.name))
    val names = answer.size / 3
    return ThemePalette(answer.take(names), answer.drop(names).map(String::toInt))
}

/** The inline autofill chips' background and foreground in [info] for the keyboard's [mode] and [shift], as Rust resolves them. */
internal fun inlineChipColors(info: ThemeManager.ThemeInfo, mode: String, shift: String, palette: ThemePalette): Pair<Int, Int> {
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
    info: ThemeManager.ThemeInfo, palette: ThemePalette, density: Float, fontScale: Float, multiplier: Float,
    mode: String = "characters", shift: String = "unshifted",
) = RendererTheme(info.stylesheetJson, palette.names, palette.colors, density, fontScale, multiplier, mode, shift)

/**
 * What the renderer draws in: a stylesheet, its palette, the display's scale and the keyboard's state, and the
 * stylesheet's `@font` [fonts], each `src` URI then its absolute path ([themeFontFiles]).
 */
data class RendererTheme(val json: String, val paletteNames: List<String>, val paletteColors: List<Int>, val density: Float, val fontScale: Float,
    val fontMultiplier: Float, val mode: String = "characters", val shift: String = "unshifted", val fonts: List<String> = emptyList())
