package org.florisboard.lib.snygg.ui

import androidx.compose.material3.ColorScheme
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import com.materialkolor.dynamicColorScheme
import org.florisboard.lib.color.ColorPalette
import org.florisboard.lib.color.MaterialYouFlags
import org.florisboard.lib.color.getColor

/** Platform palette materialization only; the native renderer selects colours from these names. */
data class SnyggPalette(val names: List<String>, val colors: List<Int>)

/** The Material You scheme ProvideSnyggTheme gives a theme with [flags] for accent [primary]. */
fun materialYouScheme(primary: Color, isDark: Boolean, flags: MaterialYouFlags): ColorScheme = dynamicColorScheme(
    primary = primary, isDark = isDark, style = flags.paletteStyle, contrastLevel = flags.contrastLevel.value,
    specVersion = flags.specVersion,
)

/** The palette the keyboard draws a theme with [flags] in for accent [primary], made outside composition: the Rust
 * renderer's, the inline chips' and the theme editor's dynamic colour swatches. */
fun snyggPalette(primary: Color, flags: MaterialYouFlags) =
    snyggPalette(materialYouScheme(primary, false, flags), materialYouScheme(primary, true, flags))

private fun snyggPalette(light: ColorScheme, dark: ColorScheme): SnyggPalette {
    val names = ColorPalette.colorNames
    return SnyggPalette(names, names.map { light.getColor(it).toArgb() } + names.map { dark.getColor(it).toArgb() })
}
