package com.jtn0123.audiowhisper.render

import android.content.Context
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import com.jtn0123.audiowhisper.rust.KeyGraph
import com.jtn0123.audiowhisper.rust.SmartbarAction
import com.jtn0123.audiowhisper.rust.SmartbarActionKind
import com.jtn0123.audiowhisper.rust.SmartbarActionPlan
import com.jtn0123.audiowhisper.rust.SmartbarBar
import com.jtn0123.audiowhisper.rust.SmartbarIcon
import com.jtn0123.audiowhisper.rust.SmartbarPlan
import com.jtn0123.audiowhisper.rust.SmartbarRules
import com.jtn0123.audiowhisper.rust.WireFlag
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.keyboard.keyIconVector
import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.keyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.nlpManager
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

/**
 * The packed facts Rust places the Smartbar from (see `smartbar::Config`). The Rust renderer and
 * Compose's Smartbar read the same array, so both ask Rust the same question.
 */
@Composable
internal fun smartbarConfig(preview: Boolean, inline: Boolean): FloatArray = with(LocalDensity.current) {
    val prefs by FlorisPreferenceStore
    val nlp by LocalContext.current.nlpManager()
    val dictation by Dictation.status.collectAsState()
    val reading by ReadAloud.status.collectAsState()
    val words by nlp.activeCandidatesWordsFlow.collectAsState()
    val enabled by prefs.smartbar.enabled.observeAsState()
    val layout by prefs.smartbar.layout.observeAsState()
    val flip by prefs.smartbar.flipToggles.observeAsState()
    val shared by prefs.smartbar.sharedActionsExpanded.observeAsState()
    val extended by prefs.smartbar.extendedActionsExpanded.observeAsState()
    val placement by prefs.smartbar.extendedActionsPlacement.observeAsState()
    val speaker by prefs.smartbar.showReadAloud.observeAsState()
    val arrangement by prefs.smartbar.actionArrangement.observeAsState()
    val display by prefs.suggestion.displayMode.observeAsState()
    fun bit(value: Boolean) = if (value) 1f else 0f
    floatArrayOf(bit(enabled), layout.ordinal.toFloat(), bit(flip), bit(shared), bit(extended),
        placement.ordinal.toFloat(), bit(speaker), bit(arrangement.stickyAction != null),
        arrangement.dynamicActions.size.toFloat(), display.ordinal.toFloat(), density,
        FlorisImeSizing.smartbarHeight.toPx(), bit(inline), bit(preview), bit(words),
        bit(dictation != Dictation.Status.IDLE), bit(reading == ReadAloud.Status.READING))
}

/** [smartbarConfig]'s facts read without Compose, the density and row height left for Rust's plan. */
internal fun smartbarFacts(r: RootReads, preview: Boolean, inline: Boolean): FloatArray = with(r.prefs.smartbar) {
    fun bit(value: Boolean) = if (value) 1f else 0f
    val arrangement = actionArrangement.get()
    floatArrayOf(bit(enabled.get()), layout.get().ordinal.toFloat(), bit(flipToggles.get()), bit(sharedActionsExpanded.get()),
        bit(extendedActionsExpanded.get()), extendedActionsPlacement.get().ordinal.toFloat(), bit(showReadAloud.get()),
        bit(arrangement.stickyAction != null), arrangement.dynamicActions.size.toFloat(),
        r.prefs.suggestion.displayMode.get().ordinal.toFloat(), 0f, 0f, bit(inline), bit(preview),
        bit(r.nlp.activeCandidatesWordsFlow.value), bit(Dictation.status.value != Dictation.Status.IDLE),
        bit(ReadAloud.status.value == ReadAloud.Status.READING))
}

/** Platform settings, evaluated action icons and handlers for the Smartbar [config]; all placement decisions are native. */
internal fun rendererSmartbar(r: RootReads, config: FloatArray): RendererSmartbar {
    val manager = r.keyboard
    val evaluator = manager.activeSmartbarEvaluator.value
    val arrangement = r.prefs.smartbar.actionArrangement.get()
    // Slot 2 was the pinned emoji, now the bottom row's emoji key; Rust never places it.
    val actions = listOf(arrangement.stickyAction, QuickAction.InsertKey(TextKeyData.READ_ALOUD),
        null, QuickAction.InsertKey(TextKeyData.TOGGLE_ACTIONS_OVERFLOW), null) + arrangement.dynamicActions
    val bar = smartbarBar(r.prefs.keyboard.incognitoDisplayMode.get(), evaluator.state.isIncognitoMode,
        Dictation.status.value, Dictation.level.value, Dictation.warming.value)
    val shown = SmartbarShown(config, bar, actions, RustKeyboard.of(manager.activeEvaluator.value.keyboard).graph)
    val incognito = if (SmartbarIcon.INCOGNITO in shown.icons) r.incognito else null
    val labels = actions.indices.flatMap {
        listOf(shown.icon(it, incognito, r.service)?.let(::iconPath) ?: "", shown.label(it))
    }.toTypedArray()
    return RendererSmartbar(config, actions, shown.codes, labels, actions.map { it?.let(r::tooltip) ?: "" }, Dictation.warming.value,
        listOfNotNull(KeyCode.TOGGLE_INCOGNITO_MODE.takeIf { evaluator.state.isIncognitoMode },
            KeyCode.TOGGLE_COMPACT_LAYOUT.takeIf { r.prefs.keyboard.oneHandedModeEnabled.get() }).toIntArray()) {
        if (shown.expanded) manager.activeState.isActionsOverflowVisible = false
        r.scope.launch {
            if (shown.togglesExtended) r.prefs.smartbar.extendedActionsExpanded.set(!shown.expanded)
            else r.prefs.smartbar.sharedActionsExpanded.set(!shown.expanded)
        }
    }
}

/** A `SmartbarBar` row: what every action's plan reads that [smartbarConfig] does not carry. */
internal fun smartbarBar(incognitoMode: IncognitoDisplayMode, incognito: Boolean, dictation: Dictation.Status, level: Float,
                         warming: Boolean) = with(SmartbarBar) {
    Array(SIZE) { "" }.also { row ->
        row[INCOGNITO_DISPLAY] = "${incognitoMode.ordinal}"; row[LEVEL] = "$level"
        for ((field, on) in listOf(INCOGNITO to incognito, LISTENING to (dictation == Dictation.Status.LISTENING), WARMING to warming)) {
            row[field] = WireFlag.encode(on)
        }
    }
}

/**
 * What each action shows, as Rust plans it (`smartbar_actions`) from the [config], the [bar] row
 * and each action's key data, which the key [graph] answers for: whether it is enabled, and the
 * icon and label of its key.
 */
internal class SmartbarShown(config: FloatArray, bar: Array<String>, private val actions: List<QuickAction?>, graph: KeyGraph) {
    private val plan = SmartbarRules.actions(config, graph, bar + actions.flatMap(::row))
    private fun field(i: Int, field: Int) = plan[SmartbarPlan.SIZE + i * SmartbarActionPlan.SIZE + field]
    private fun row(action: QuickAction?) = with(SmartbarAction) {
        val data = action?.keyData()
        Array(SIZE) { "" }.also { row ->
            row[KIND] = "${when (action) { null -> SmartbarActionKind.NONE; is QuickAction.InsertKey -> SmartbarActionKind.KEY
                is QuickAction.InsertText -> SmartbarActionKind.TEXT }}"
            row[CODE] = "${data?.code ?: 0}"; row[TYPE] = "${data?.type?.ordinal ?: 0}"; row[LABEL] = data?.label ?: ""
            row[TEXT] = (action as? QuickAction.InsertText)?.data ?: ""
        }.asList()
    }

    val expanded = WireFlag.decode(plan[SmartbarPlan.EXPANDED])
    val togglesExtended = WireFlag.decode(plan[SmartbarPlan.TOGGLES_EXTENDED])
    /** Each action's code and flags, as the renderer's smartbar packet carries them. */
    val codes = IntArray(actions.size * BAR_ACTION_STRIDE) {
        field(it / BAR_ACTION_STRIDE, if (it % BAR_ACTION_STRIDE == BAR_FLAGS_OFFSET) SmartbarActionPlan.FLAGS else SmartbarActionPlan.CODE).toInt()
    }
    /** Each action's `SmartbarIcon`. */
    val icons = IntArray(actions.size) { field(it, SmartbarActionPlan.ICON).toInt() }

    /** Action [i]'s icon, with [incognito] for `SmartbarIcon.INCOGNITO` and its key's drawn from [context]. */
    fun icon(i: Int, incognito: ImageVector?, context: Context?): ImageVector? = when (icons[i]) {
        SmartbarIcon.KEY -> keyIconVector(keyIcon(i), context)
        SmartbarIcon.ARROW_LEFT -> Icons.AutoMirrored.Default.KeyboardArrowLeft
        SmartbarIcon.ARROW_RIGHT -> Icons.AutoMirrored.Default.KeyboardArrowRight
        SmartbarIcon.UNFOLD_LESS -> Icons.Default.UnfoldLess
        SmartbarIcon.UNFOLD_MORE -> Icons.Default.UnfoldMore
        SmartbarIcon.INCOGNITO -> incognito
        SmartbarIcon.STOP -> Icons.Default.Stop
        else -> null
    }

    /** The icon token the graph picked for action [i]'s key, 0 for none. */
    fun keyIcon(i: Int): Int = field(i, SmartbarActionPlan.KEY_ICON).toInt()

    /** Action [i]'s label: its own, or its key's. */
    fun label(i: Int): String = field(i, SmartbarActionPlan.LABEL)
}
