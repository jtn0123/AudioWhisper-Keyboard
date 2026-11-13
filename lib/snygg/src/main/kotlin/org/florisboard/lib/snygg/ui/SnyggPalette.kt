package org.florisboard.lib.snygg.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.toArgb
import org.florisboard.lib.color.ColorPalette
import org.florisboard.lib.color.getColor

/** Platform palette materialization only; the native renderer selects colours from these names. */
data class SnyggPalette(val names: List<String>, val colors: List<Int>)

@Composable
fun rememberSnyggPalette(): SnyggPalette {
    val light = LocalSnyggDynamicLightColorScheme.current
    val dark = LocalSnyggDynamicDarkColorScheme.current
    return remember(light, dark) {
        val names = ColorPalette.colorNames
        SnyggPalette(names, names.map { light.getColor(it).toArgb() } + names.map { dark.getColor(it).toArgb() })
    }
}
