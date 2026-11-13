package com.jtn0123.audiowhisper.render

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.themeManager
import org.florisboard.lib.snygg.ui.rememberSnyggPalette

@Composable
internal fun rendererTheme(): RendererTheme {
    val manager by LocalContext.current.themeManager()
    val active by manager.activeThemeInfo.collectAsState()
    val palette = rememberSnyggPalette()
    val density = LocalDensity.current
    val prefs by FlorisPreferenceStore
    val multiplier = prefs.keyboard.fontSizeMultiplier()
    return remember(active, palette, density, multiplier) {
        RendererTheme(active.stylesheetJson, palette.names, palette.colors, density.density, density.fontScale, multiplier)
    }
}
