package com.jtn0123.audiowhisper.render

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Stop
import androidx.compose.material.icons.filled.UnfoldLess
import androidx.compose.material.icons.filled.UnfoldMore
import androidx.compose.ui.graphics.vector.ImageVector
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.reader.ReadAloud
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode
import dev.patrickgold.florisboard.ime.smartbar.SmartbarLayout
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.keyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode

/**
 * What rendererSmartbar reads, for the frozen reference and the branch alike. [preview] folds the
 * shared actions in the layout, but main's choices read the setting itself, so it must not matter.
 */
internal data class SmartbarFacts(
    val layout: SmartbarLayout, val flip: Boolean, val shared: Boolean, val extended: Boolean,
    val incognitoMode: IncognitoDisplayMode, val incognito: Boolean, val dictation: Dictation.Status,
    val level: Float, val warming: Boolean, val reading: ReadAloud.Status, val preview: Boolean,
) {
    /** The config smartbarConfig packs for these facts, with three dynamic actions. */
    fun config(): FloatArray {
        fun bit(value: Boolean) = if (value) 1f else 0f
        return floatArrayOf(1f, layout.ordinal.toFloat(), bit(flip), bit(shared), bit(extended), 0f, 1f, 1f, 3f, 0f,
            2.75f, 132f, 0f, bit(preview), 1f, bit(dictation != Dictation.Status.IDLE), bit(reading == ReadAloud.Status.READING))
    }
}

/** What the key evaluator says of a key, faked for the trace. */
internal class FakeEvaluator(
    val enabled: (KeyData) -> Boolean,
    val keyIcon: (KeyData) -> ImageVector?,
    val keyLabel: (KeyData) -> String?,
)

/** What main's choices showed. */
internal class ReferenceShown(val expanded: Boolean, val togglesExtended: Boolean, val codes: IntArray,
                              val labels: List<Pair<ImageVector?, String>>)

/**
 * Main's choice of each Smartbar action's icon, label and state, frozen from rendererSmartbar in
 * RendererSmartbar.kt at f1fdf3f34 (lines 98 and 102 to 126) and never called by production. The
 * evaluator and the incognito drawable are parameters, each label pair keeps its icon's vector
 * rather than its path data, and the toggle's target is whether the layout is the extended one.
 */
internal fun referenceSmartbarActions(facts: SmartbarFacts, actions: List<QuickAction?>, evaluator: FakeEvaluator,
                                      incognitoIcon: ImageVector): ReferenceShown = with(facts) {
    val expanded = if (layout == SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED) extended else shared
    val arrow = if (flip xor (expanded && incognitoMode == IncognitoDisplayMode.DISPLAY_BEHIND_KEYBOARD)) {
        Icons.AutoMirrored.Default.KeyboardArrowLeft
    } else Icons.AutoMirrored.Default.KeyboardArrowRight
    val toggleIcon = if (layout == SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED) {
        if (extended) Icons.Default.UnfoldLess else Icons.Default.UnfoldMore
    } else if (incognito && incognitoMode == IncognitoDisplayMode.REPLACE_SHARED_ACTIONS_TOGGLE) {
        incognitoIcon
    } else arrow
    val codes = actions.flatMap { action -> listOf(action?.keyData()?.code ?: 0,
        (if (action == null || evaluator.enabled(action.keyData())) BAR_ENABLED else 0) or
            (if (action?.keyData()?.code == KeyCode.VOICE_INPUT && dictation != Dictation.Status.IDLE) BAR_DICTATING else 0) or
            (if (action?.keyData()?.code == KeyCode.READ_ALOUD && reading == ReadAloud.Status.READING) BAR_READING else 0) or
            (if (action?.keyData()?.code == KeyCode.VOICE_INPUT) {
                (if (warming) BAR_WARMING else 0) or (if (dictation != Dictation.Status.LISTENING) BAR_FINISHING else 0) or
                    ((level.coerceIn(0f,1f)*BAR_LEVEL_MASK).toInt() shl BAR_LEVEL_SHIFT)
            } else 0)) }.toIntArray()
    val labels = actions.map { action ->
        val icon = if (action?.keyData()?.code == KeyCode.READ_ALOUD && reading == ReadAloud.Status.READING) Icons.Default.Stop else if (action == null) toggleIcon else if (action is QuickAction.InsertKey) evaluator.keyIcon(action.data) else null
        icon to when (action) {
            is QuickAction.InsertText -> action.data.firstOrNull().toString().ifBlank { "?" }
            is QuickAction.InsertKey -> if (icon == null) evaluator.keyLabel(action.data) ?: "" else ""
            null -> ""
        }
    }
    ReferenceShown(expanded, layout == SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED, codes, labels)
}
