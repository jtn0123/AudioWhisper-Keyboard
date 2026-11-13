package com.jtn0123.audiowhisper.render

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Stop
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.reader.ReadAloud
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import androidx.compose.material.icons.filled.UnfoldLess
import androidx.compose.material.icons.filled.UnfoldMore
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.vectorResource
import com.jtn0123.audiowhisper.rust.SmartbarRules
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.computeImageVector
import dev.patrickgold.florisboard.ime.keyboard.computeLabel
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode
import dev.patrickgold.florisboard.ime.smartbar.SmartbarLayout
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.keyData
import dev.patrickgold.florisboard.ime.smartbar.quickaction.computeTooltip
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.jetpref.datastore.model.observeAsState
import kotlinx.coroutines.launch

internal data class RendererSmartbar(
    val config: FloatArray,
    val actions: List<QuickAction?>,
    val codes: IntArray,
    val labels: Array<String>,
    val tooltips: List<String>,
    val animating: Boolean,
    val checkedCodes: IntArray,
    val toggle: () -> Unit,
) {
    fun measure(width: Float) = SmartbarRules.layout(config, width, floatArrayOf())
}

/** Platform settings, evaluated action icons and handlers; all placement decisions are native. */
@Composable
internal fun rendererSmartbar(preview: Boolean): RendererSmartbar = with(LocalDensity.current) {
    val context = LocalContext.current
    val prefs by FlorisPreferenceStore
    val manager by context.keyboardManager()
    val dictation by Dictation.status.collectAsState()
    val level by Dictation.level.collectAsState()
    val warming by Dictation.warming.collectAsState()
    val reading by ReadAloud.status.collectAsState()
    val evaluator by manager.activeSmartbarEvaluator.collectAsState()
    val enabled by prefs.smartbar.enabled.observeAsState()
    val layout by prefs.smartbar.layout.observeAsState()
    val flip by prefs.smartbar.flipToggles.observeAsState()
    val shared by prefs.smartbar.sharedActionsExpanded.observeAsState()
    val extended by prefs.smartbar.extendedActionsExpanded.observeAsState()
    val placement by prefs.smartbar.extendedActionsPlacement.observeAsState()
    val speaker by prefs.smartbar.showReadAloud.observeAsState()
    val arrangement by prefs.smartbar.actionArrangement.observeAsState()
    val display by prefs.suggestion.displayMode.observeAsState()
    val incognitoMode by prefs.keyboard.incognitoDisplayMode.observeAsState()
    val scope = rememberCoroutineScope()
    val expanded = if (layout == SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED) extended else shared
    val actions = listOf(arrangement.stickyAction, QuickAction.InsertKey(TextKeyData.READ_ALOUD),
        QuickAction.InsertKey(TextKeyData.IME_UI_MODE_MEDIA), QuickAction.InsertKey(TextKeyData.TOGGLE_ACTIONS_OVERFLOW),
        null) + arrangement.dynamicActions
    val arrow = if (flip xor (expanded && incognitoMode == IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD)) {
        Icons.AutoMirrored.Default.KeyboardArrowLeft
    } else Icons.AutoMirrored.Default.KeyboardArrowRight
    val toggleIcon = if (layout == SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED) {
        if (extended) Icons.Default.UnfoldLess else Icons.Default.UnfoldMore
    } else if (evaluator.state.isIncognitoMode && incognitoMode == IncognitoDisplayMode.REPLACE_SHARED_ACTIONS_TOGGLE) {
        ImageVector.vectorResource(R.drawable.ic_incognito)
    } else arrow
    val codes = actions.flatMap { action -> listOf(action?.keyData()?.code ?: 0,
        (if (action == null || evaluator.evaluateEnabled(action.keyData())) BAR_ENABLED else 0) or
            (if (action?.keyData()?.code == KeyCode.VOICE_INPUT && dictation != Dictation.Status.IDLE) BAR_DICTATING else 0) or
            (if (action?.keyData()?.code == KeyCode.READ_ALOUD && reading == ReadAloud.Status.READING) BAR_READING else 0) or
            (if (action?.keyData()?.code == KeyCode.VOICE_INPUT) {
                (if (warming) BAR_WARMING else 0) or (if (dictation != Dictation.Status.LISTENING) BAR_FINISHING else 0) or
                    ((level.coerceIn(0f,1f)*BAR_LEVEL_MASK).toInt() shl BAR_LEVEL_SHIFT)
            } else 0)) }.toIntArray()
    val tooltips = actions.map { it?.computeTooltip(evaluator) ?: "" }
    val labels = actions.flatMap { action ->
        val icon = if (action?.keyData()?.code == KeyCode.READ_ALOUD && reading == ReadAloud.Status.READING) Icons.Default.Stop else if (action == null) toggleIcon else if (action is QuickAction.InsertKey) evaluator.computeImageVector(action.data) else null
        listOf(icon?.let(::iconPath) ?: "", when (action) {
            is QuickAction.InsertText -> action.data.firstOrNull().toString().ifBlank { "?" }
            is QuickAction.InsertKey -> if (icon == null) evaluator.computeLabel(action.data) ?: "" else ""
            null -> ""
        })
    }.toTypedArray()
    RendererSmartbar(floatArrayOf(if (enabled) 1f else 0f, layout.ordinal.toFloat(), if (flip) 1f else 0f,
        if (shared && !preview) 1f else 0f, if (extended) 1f else 0f, placement.ordinal.toFloat(), if (speaker) 1f else 0f,
        if (arrangement.stickyAction != null) 1f else 0f, arrangement.dynamicActions.size.toFloat(),
        display.ordinal.toFloat(), density, FlorisImeSizing.smartbarHeight.toPx()), actions, codes, labels, tooltips, warming,
        listOfNotNull(KeyCode.TOGGLE_INCOGNITO_MODE.takeIf { evaluator.state.isIncognitoMode },
            KeyCode.TOGGLE_COMPACT_LAYOUT.takeIf { prefs.keyboard.oneHandedModeEnabled.get() }).toIntArray()) {
        if (expanded) manager.activeState.isActionsOverflowVisible = false
        scope.launch {
            if (layout == SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED) prefs.smartbar.extendedActionsExpanded.set(!extended)
            else prefs.smartbar.sharedActionsExpanded.set(!shared)
        }
    }
}
