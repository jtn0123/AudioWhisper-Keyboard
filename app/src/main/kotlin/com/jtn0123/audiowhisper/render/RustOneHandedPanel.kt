package com.jtn0123.audiowhisper.render

import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.ZoomOutMap
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.input.LocalInputFeedbackController
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.onehanded.OneHandedMode
import kotlinx.coroutines.launch
import org.florisboard.lib.compose.stringRes

@Composable
internal fun RowScope.RustOneHandedPanel(side: OneHandedMode, weight: Float) {
    val theme = rendererTheme()
    val prefs by FlorisPreferenceStore
    val scope = rememberCoroutineScope()
    val feedback = LocalInputFeedbackController.current
    val labels = arrayOf(stringRes(R.string.one_handed__close_btn_content_description),
        stringRes(if (side == OneHandedMode.START) R.string.one_handed__move_start_btn_content_description else R.string.one_handed__move_end_btn_content_description))
    val icons = arrayOf(iconPath(Icons.Default.ZoomOutMap), iconPath(if (side == OneHandedMode.START)
        Icons.AutoMirrored.Filled.KeyboardArrowLeft else Icons.AutoMirrored.Filled.KeyboardArrowRight))
    AndroidView(modifier = Modifier.weight(weight).height(FlorisImeSizing.imeUiHeight()),
        factory = { RustChromeView(it) }, update = { view ->
            view.update(theme, 0, labels, icons)
            view.action = { op -> scope.launch {
                feedback.keyPress()
                if (op == 0) prefs.keyboard.oneHandedModeEnabled.set(false) else prefs.keyboard.oneHandedMode.set(side)
            } }
        })
}
