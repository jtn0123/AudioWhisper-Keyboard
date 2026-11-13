package com.jtn0123.audiowhisper.render

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.viewinterop.AndroidView
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.subtypeManager

/** A window-width native modal, independent of compact keyboard placement underneath. */
@Composable
internal fun RustSubtypeSheet() {
    val context = LocalContext.current
    val keyboard by context.keyboardManager()
    val manager by context.subtypeManager()
    val data = rendererSubtypes()
    val theme = rendererTheme()
    val bottomInset = WindowInsets.safeDrawing.getBottom(LocalDensity.current).toFloat()
    AndroidView(modifier = Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Top + WindowInsetsSides.Horizontal)), factory = { RustChromeView(it) }, update = { view ->
        view.subtypeAction = { index ->
            index?.let { data.ids.getOrNull(it) }?.let(manager::switchToSubtypeById)
            keyboard.activeState.isSubtypeSelectionVisible = false
        }
        view.updateSubtypes(theme, data, bottomInset)
    })
}
