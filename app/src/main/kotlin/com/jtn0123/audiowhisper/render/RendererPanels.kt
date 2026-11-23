package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyGraph
import com.jtn0123.audiowhisper.rust.KeyGraphAction
import com.jtn0123.audiowhisper.rust.KeyGraphActionAnswer
import com.jtn0123.audiowhisper.rust.KeyGraphOp
import com.jtn0123.audiowhisper.rust.WireFlag
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.smartbar.quickaction.*
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData

internal data class RendererPanels(val kind: Int, val actions: List<QuickAction>, val arrangement: QuickActionArrangement,
    val data: IntArray, val labels: Array<String>, val emoji: RendererEmoji, val clipboard: RendererClipboard, val gif: RendererGif)

/** Android resources and unchanged preference values for the panel Rust planned, [kind]; the key graph answers for each action's key. */
internal fun rendererPanels(r: RootReads, kind: Int, emoji: RendererEmoji, clipboard: RendererClipboard, gif: RendererGif): RendererPanels {
    val arrangement = r.prefs.smartbar.actionArrangement.get()
    val actions = (listOfNotNull(arrangement.stickyAction) + arrangement.dynamicActions + arrangement.hiddenActions +
        listOf(QuickAction.InsertKey(TextKeyData.IME_UI_MODE_TEXT), QuickAction.InsertKey(TextKeyData.DELETE),
            QuickAction.InsertKey(TextKeyData(code = KeyCode.NOOP)))).distinct()
    val keys = ActionKeys(RustKeyboard.of(r.keyboard.activeEvaluator.value.keyboard).graph, actions.map { it.keyData() })
    val data = listOf(actions.indexOf(arrangement.stickyAction), arrangement.dynamicActions.size, arrangement.hiddenActions.size) +
        arrangement.dynamicActions.map { actions.indexOf(it) } + arrangement.hiddenActions.map { actions.indexOf(it) } +
        actions.flatMapIndexed { i, it -> listOf(if (it.keyData().code == KeyCode.NOOP) -1 else i, it.keyData().code,
            (if (keys.enabled(i)) 1 else 0) or (if (it is QuickAction.InsertText) 2 else 0)) }
    val titles = listOf(R.string.quick_actions_overflow__customize_actions_button, R.string.quick_actions_editor__header,
        R.string.quick_actions_editor__subheader_sticky_action, R.string.quick_actions_editor__subheader_dynamic_actions,
        R.string.quick_actions_editor__subheader_hidden_actions).map { r.string(it) } + "Back"
    val labels = actions.flatMapIndexed { i, action ->
        val icon = if (action is QuickAction.InsertKey) keys.icon(i) else 0
        listOf(r.name(action), when (action) {
            is QuickAction.InsertKey -> if (icon == 0) keys.label(i) ?: "" else ""
            is QuickAction.InsertText -> action.data.firstOrNull().toString().ifBlank { "?" }
        }, if (icon == 0) "" else "$icon")
    }
    return RendererPanels(kind, actions, arrangement, data.toIntArray(), (titles + labels).toTypedArray(), emoji, clipboard, gif)
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
