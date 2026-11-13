package com.jtn0123.audiowhisper.render

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.computeImageVector
import dev.patrickgold.florisboard.ime.keyboard.computeLabel
import dev.patrickgold.florisboard.ime.smartbar.quickaction.*
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.jetpref.datastore.model.observeAsState
import org.florisboard.lib.compose.stringRes

internal data class RendererPanels(val kind: Int, val actions: List<QuickAction>, val arrangement: QuickActionArrangement,
    val data: IntArray, val labels: Array<String>, val emoji: RendererEmoji)

/** Android resources/evaluator and unchanged preference values; no grid or drag decisions. */
@Composable
internal fun rendererPanels(): RendererPanels {
    val context = LocalContext.current
    val manager by context.keyboardManager()
    val prefs by FlorisPreferenceStore
    val state by manager.activeState.collectAsState()
    val evaluator by manager.activeSmartbarEvaluator.collectAsState()
    val arrangement by prefs.smartbar.actionArrangement.observeAsState()
    val actions = (listOfNotNull(arrangement.stickyAction) + arrangement.dynamicActions + arrangement.hiddenActions +
        listOf(QuickAction.InsertKey(dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData.IME_UI_MODE_TEXT),
            QuickAction.InsertKey(dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData.DELETE),
            QuickAction.InsertKey(dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData(code = dev.patrickgold.florisboard.ime.text.key.KeyCode.NOOP)))).distinct()
    val data = listOf(actions.indexOf(arrangement.stickyAction), arrangement.dynamicActions.size, arrangement.hiddenActions.size) +
        arrangement.dynamicActions.map { actions.indexOf(it) } + arrangement.hiddenActions.map { actions.indexOf(it) } +
        actions.flatMap { listOf(if (it.keyData().code == dev.patrickgold.florisboard.ime.text.key.KeyCode.NOOP) -1 else actions.indexOf(it), it.keyData().code, (if (evaluator.evaluateEnabled(it.keyData())) 1 else 0) or (if (it is QuickAction.InsertText) 2 else 0)) }
    val titles = listOf(stringRes(R.string.quick_actions_overflow__customize_actions_button), stringRes(R.string.quick_actions_editor__header),
        stringRes(R.string.quick_actions_editor__subheader_sticky_action),
        stringRes(R.string.quick_actions_editor__subheader_dynamic_actions),
        stringRes(R.string.quick_actions_editor__subheader_hidden_actions), "Back")
    val labels = actions.flatMap { action ->
        val icon = if (action is QuickAction.InsertKey) evaluator.computeImageVector(action.data) else null
        listOf(action.computeDisplayName(evaluator), when (action) {
            is QuickAction.InsertKey -> if (icon == null) evaluator.computeLabel(action.data) ?: "" else ""
            is QuickAction.InsertText -> action.data.firstOrNull().toString().ifBlank { "?" }
        }, icon?.let(::iconPath) ?: "")
    }
    return RendererPanels(if (state.isActionsEditorVisible) 3 else if (state.isActionsOverflowVisible) 2 else if (state.imeUiMode == dev.patrickgold.florisboard.ime.ImeUiMode.MEDIA) 1 else 0,
        actions, arrangement, data.toIntArray(), (titles + labels).toTypedArray(), rendererEmoji())
}
