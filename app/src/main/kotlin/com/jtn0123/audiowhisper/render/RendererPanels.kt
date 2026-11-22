package com.jtn0123.audiowhisper.render

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import com.jtn0123.audiowhisper.rust.KeyGraph
import com.jtn0123.audiowhisper.rust.KeyGraphAction
import com.jtn0123.audiowhisper.rust.KeyGraphActionAnswer
import com.jtn0123.audiowhisper.rust.KeyGraphOp
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.keyIconVector
import dev.patrickgold.florisboard.ime.smartbar.quickaction.*
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.jetpref.datastore.model.observeAsState
import org.florisboard.lib.compose.stringRes

internal data class RendererPanels(val kind: Int, val actions: List<QuickAction>, val arrangement: QuickActionArrangement,
    val data: IntArray, val labels: Array<String>, val emoji: RendererEmoji, val clipboard: RendererClipboard, val gif: RendererGif)

/** Android resources and unchanged preference values; the key graph answers for each action's key. */
@Composable
internal fun rendererPanels(): RendererPanels {
    val context = LocalContext.current
    val manager by context.keyboardManager()
    val prefs by FlorisPreferenceStore
    val state by manager.activeState.collectAsState()
    val keyboard by manager.activeEvaluator.collectAsState()
    val arrangement by prefs.smartbar.actionArrangement.observeAsState()
    val actions = (listOfNotNull(arrangement.stickyAction) + arrangement.dynamicActions + arrangement.hiddenActions +
        listOf(QuickAction.InsertKey(TextKeyData.IME_UI_MODE_TEXT), QuickAction.InsertKey(TextKeyData.DELETE),
            QuickAction.InsertKey(TextKeyData(code = KeyCode.NOOP)))).distinct()
    val keys = ActionKeys(RustKeyboard.of(keyboard.keyboard).graph, actions.map { it.keyData() })
    val data = listOf(actions.indexOf(arrangement.stickyAction), arrangement.dynamicActions.size, arrangement.hiddenActions.size) +
        arrangement.dynamicActions.map { actions.indexOf(it) } + arrangement.hiddenActions.map { actions.indexOf(it) } +
        actions.flatMapIndexed { i, it -> listOf(if (it.keyData().code == KeyCode.NOOP) -1 else i, it.keyData().code,
            (if (keys.enabled(i)) 1 else 0) or (if (it is QuickAction.InsertText) 2 else 0)) }
    val titles = listOf(stringRes(R.string.quick_actions_overflow__customize_actions_button), stringRes(R.string.quick_actions_editor__header),
        stringRes(R.string.quick_actions_editor__subheader_sticky_action),
        stringRes(R.string.quick_actions_editor__subheader_dynamic_actions),
        stringRes(R.string.quick_actions_editor__subheader_hidden_actions), "Back")
    val gif = rendererGif()
    val labels = actions.flatMapIndexed { i, action ->
        val icon = if (action is QuickAction.InsertKey) keyIconVector(keys.icon(i), context) else null
        listOf(action.computeDisplayName(), when (action) {
            is QuickAction.InsertKey -> if (icon == null) keys.label(i) ?: "" else ""
            is QuickAction.InsertText -> action.data.firstOrNull().toString().ifBlank { "?" }
        }, icon?.let(::iconPath) ?: "")
    }
    return RendererPanels(if (state.isActionsEditorVisible) 3 else if (state.isActionsOverflowVisible) 2 else if (state.imeUiMode == dev.patrickgold.florisboard.ime.ImeUiMode.CLIPBOARD) 4 else if (state.imeUiMode == dev.patrickgold.florisboard.ime.ImeUiMode.MEDIA) { if(gif.showing) 5 else 1 } else 0,
        actions, arrangement, data.toIntArray(), (titles + labels).toTypedArray(), rendererEmoji(), rendererClipboard(), gif)
}

/**
 * What the key [graph] says of each of [keys], a smartbar action's key, with its last facts, as
 * the smartbar's evaluator did: whether it is enabled, its icon token and its label.
 */
internal class ActionKeys(graph: KeyGraph, keys: List<KeyData>) {
    private val rows = graph.call(KeyGraphOp.ACTIONS, *keys.flatMap { key ->
        KeyGraphAction.row(KeyGraphAction.TYPE to "${key.type.ordinal}", KeyGraphAction.CODE to "${key.code}",
            KeyGraphAction.LABEL to key.label).asList()
    }.toTypedArray())

    private fun field(i: Int, field: Int) = rows[i * KeyGraphActionAnswer.SIZE + field]

    fun enabled(i: Int): Boolean = WireFlag.decode(field(i, KeyGraphActionAnswer.ENABLED))

    /** Key [i]'s icon token, 0 for none. */
    fun icon(i: Int): Int = field(i, KeyGraphActionAnswer.ICON).toInt()

    /** Key [i]'s label, or null when it shows none. */
    fun label(i: Int): String? =
        field(i, KeyGraphActionAnswer.LABEL).takeIf { WireFlag.decode(field(i, KeyGraphActionAnswer.LABELED)) }
}
