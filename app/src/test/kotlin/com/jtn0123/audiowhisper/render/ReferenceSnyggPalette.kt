package com.jtn0123.audiowhisper.render

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import org.florisboard.lib.color.ColorPalette
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.color.getColor
import org.florisboard.lib.snygg.ui.materialYouScheme

/**
 * lib/snygg's `snyggPalette`, deleted when Rust took the palette over (docs/phase3-compose-free-path.md), verbatim
 * but for returning a [ThemePalette]: the palette main's renderer, inline chips and theme editor got from
 * materialkolor, through the same `materialYouScheme` call Compose's ProvideSnyggTheme still makes.
 */
internal fun referenceSnyggPalette(primary: Color, flags: MaterialYouFlags): ThemePalette {
    val light = materialYouScheme(primary, false, flags)
    val dark = materialYouScheme(primary, true, flags)
    val names = ColorPalette.colorNames
    return ThemePalette(names, names.map { light.getColor(it).toArgb() } + names.map { dark.getColor(it).toArgb() })
}
