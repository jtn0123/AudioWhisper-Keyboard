package com.jtn0123.audiowhisper.render

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode
import dev.patrickgold.jetpref.datastore.model.observeAsState

@Composable
internal fun rendererWatermark(evaluator: ComputingEvaluator): Pair<String, IntArray> {
    val prefs by FlorisPreferenceStore
    val display by prefs.keyboard.incognitoDisplayMode.observeAsState()
    return iconPath(ImageVector.vectorResource(R.drawable.ic_incognito)) to intArrayOf(
        if (evaluator.state.isIncognitoMode) 1 else 0,
        if (display == IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD) 1 else 0)
}
