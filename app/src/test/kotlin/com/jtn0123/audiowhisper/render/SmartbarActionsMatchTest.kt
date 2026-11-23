package com.jtn0123.audiowhisper.render

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.reader.ReadAloud
import com.jtn0123.audiowhisper.rust.KeyGraph
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import com.jtn0123.audiowhisper.rust.SmartbarIcon
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode
import dev.patrickgold.florisboard.ime.smartbar.SmartbarLayout
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.keyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Seeded smartbars, each shown by main's choices (frozen in [referenceSmartbarActions]) and by the
 * Rust plan through [SmartbarShown], must agree on every action's code, flags, icon and label and
 * on what the arrow toggles. What a key action's key says (enabled, icon, label) comes from a
 * computed Rust key graph on the branch, and from main's smartbar evaluator, frozen as
 * [ReferenceEvaluator], for the same facts. Two labels differ on purpose, each counted: an empty
 * text shows "?" where main showed "null", and a text starting with a surrogate pair shows the
 * whole character where main showed half of it. The overflow and editor panels ask the same graph
 * about every action's key ([ActionKeys]), which must answer as main's evaluator did. See
 * docs/phase3-typing-fan-out.md and docs/phase3-key-graph.md.
 */
class SmartbarActionsMatchTest {
    private val seen = mutableMapOf<String, Int>()
    private val tokenIcons = HashMap<Int, ImageVector>()
    private val inputs = TraceInputs(Random(SEED + 1))
    /** A bundled preset's letters, which every computation of the facts completes on. */
    private val build = KeyGraphAssets.fixtures.presets.first().let { preset ->
        TraceBuild(
            KeyboardMode.CHARACTERS, false, false, preset.locale, preset.layouts, preset.popups, true, null, null, null,
        )
    }
    private val keyboard =
        RustKeyboard(KeyGraph.build(*keyGraphBuild(build.mode, build.numberRow, build.numberHints, build.files())),
            Subtype.DEFAULT)
    private fun saw(what: String) { seen[what] = (seen[what] ?: 0) + 1 }

    @Test fun mainsChoicesAndTheRustPlanShowTheSameSmartbar() {
        val random = Random(SEED)
        repeat(CASES) { case -> check(case, random) }
        println("Smartbar actions: $CASES seeded smartbars identical but for the two label fixes; $seen")
        for (icon in ICON_NAMES.values) assertTrue((seen["icon $icon"] ?: 0) > 0, "never showed $icon")
        for (flag in FLAG_NAMES.values) assertTrue((seen["flag $flag"] ?: 0) > 0, "never set $flag")
        for (what in listOf("empty text", "surrogate pair", "blank text", "key label", "expanded", "toggles extended",
            "preview of shared actions", "panel disabled", "panel icon", "panel unlabeled")) {
            assertTrue((seen[what] ?: 0) > 0, "never saw $what")
        }
    }

    private fun check(case: Int, random: Random) {
        val facts = facts(random)
        val actions = List(5 + random.nextInt(7)) { action(random) }
        val keyFacts = inputs.facts(build)
        keyboard.compute(keyFacts.row(), traceAnswer(build.locale))
        val main = ReferenceEvaluator(
            KeyboardMode.SMARTBAR_QUICK_ACTIONS, keyFacts, DEFAULT_LOCALE, keyFacts.defaultLanguage, TRACE_PLATFORM,
        )
        val icon = { data: KeyData -> main.icon(data).takeIf { token -> token != 0 }?.let(::tokenIcon) }
        val evaluator = FakeEvaluator(main::evaluateEnabled, icon, main::label)
        val reference = referenceSmartbarActions(facts, actions, evaluator, INCOGNITO)
        val shown = with(facts) {
            val bar = smartbarBar(incognitoMode, incognito, dictation, level, warming)
            SmartbarShown(config(), bar, actions, keyboard.graph)
        }
        assertEquals(reference.expanded, shown.expanded, "case $case: expanded differs")
        assertEquals(reference.togglesExtended, shown.togglesExtended, "case $case: the toggle's target differs")
        assertContentEquals(reference.codes, shown.codes, "case $case: codes or flags differ")
        for (i in actions.indices) {
            val icon = if (shown.icons[i] == SmartbarIcon.KEY) {
                tokenIcon(shown.keyIcon(i))
            } else {
                shown.icon(i, INCOGNITO, null)
            }
            assertEquals(reference.labels[i].first, icon, "case $case action $i: icon differs")
            val label = intended(actions[i], reference.labels[i].second)
            assertEquals(label, shown.label(i), "case $case action $i: label differs")
        }
        count(facts, shown, actions)
        panel(case, actions.filterNotNull(), main)
    }

    /** The overflow and editor panels ask the graph about every action's key, as main asked its evaluator. */
    private fun panel(case: Int, actions: List<QuickAction>, main: ReferenceEvaluator) {
        val keys = ActionKeys(keyboard.graph, actions.map { it.keyData() })
        for ((i, action) in actions.withIndex()) {
            val data = action.keyData()
            assertEquals(main.evaluateEnabled(data), keys.enabled(i), "case $case panel action $i: enabled differs")
            assertEquals(main.icon(data), keys.icon(i), "case $case panel action $i: icon differs")
            assertEquals(main.label(data), keys.label(i), "case $case panel action $i: label differs")
            saw("panel action")
            if (!keys.enabled(i)) saw("panel disabled")
            if (keys.icon(i) != 0) saw("panel icon")
            if (keys.label(i) == null) saw("panel unlabeled")
        }
    }

    /** A stand-in for the icon an icon token draws, the same one each time. */
    private fun tokenIcon(token: Int): ImageVector = tokenIcons.getOrPut(token) {
        ImageVector.Builder("token $token", 1.dp, 1.dp, 1f, 1f).build()
    }

    /** Main's label, but for the two fixes. */
    private fun intended(action: QuickAction?, main: String): String {
        val text = (action as? QuickAction.InsertText)?.data ?: return main.also {
            if (action is QuickAction.InsertKey && it.isNotEmpty()) saw("key label")
        }
        if (text.isEmpty()) return "?".also { assertEquals("null", main); saw("empty text") }
        if (text.length > 1 && text[0].isHighSurrogate() && text[1].isLowSurrogate()) {
            return text.substring(0, 2).also { assertEquals(text.substring(0, 1), main); saw("surrogate pair") }
        }
        if (main == "?" && text[0] != '?') saw("blank text")
        return main
    }

    private fun count(facts: SmartbarFacts, shown: SmartbarShown, actions: List<QuickAction?>) {
        if (shown.expanded) saw("expanded")
        if (shown.togglesExtended) saw("toggles extended")
        for (i in actions.indices) {
            saw("icon ${ICON_NAMES[shown.icons[i]] ?: "none"}")
            val flags = shown.codes[i * BAR_ACTION_STRIDE + BAR_FLAGS_OFFSET]
            for ((flag, name) in FLAG_NAMES) if (flags and flag != 0) saw("flag $name")
        }
        if (facts.level.isNaN()) saw("level NaN")
        if (facts.preview && facts.shared) saw("preview of shared actions")
    }

    private fun facts(random: Random) = SmartbarFacts(
        SmartbarLayout.entries.random(random), random.nextBoolean(), random.nextBoolean(), random.nextBoolean(),
        IncognitoDisplayMode.entries.random(random), random.nextBoolean(), Dictation.Status.entries.random(random),
        if (random.nextBoolean()) LEVELS.random(random) else random.nextFloat() * 1.4f - 0.2f,
        random.nextBoolean(), ReadAloud.Status.entries.random(random), random.nextInt(4) == 0,
    )

    private fun action(random: Random): QuickAction? = when (random.nextInt(5)) {
        0 -> null
        1 -> QuickAction.InsertText(TEXTS.random(random))
        else -> {
            val type = if (random.nextInt(3) == 0) KeyType.entries.random(random) else KeyType.CHARACTER
            val code = CODES.random(random)
            val label = if (random.nextBoolean()) "" else KEY_LABELS.random(random)
            QuickAction.InsertKey(TextKeyData(type, code, label))
        }
    }

    private companion object {
        const val SEED = 20261122L
        const val CASES = 20_000
        val INCOGNITO = Icons.Default.VisibilityOff
        val CODES = listOf(KeyCode.VOICE_INPUT, KeyCode.READ_ALOUD, KeyCode.TOGGLE_ACTIONS_OVERFLOW, KeyCode.UNDO,
            KeyCode.REDO, KeyCode.SETTINGS, KeyCode.CLIPBOARD_COPY, KeyCode.ARROW_LEFT, KeyCode.TOGGLE_INCOGNITO_MODE,
            KeyCode.UNSPECIFIED, 'a'.code, 'é'.code, KeyCode.LANGUAGE_SWITCH, KeyCode.VIEW_SYMBOLS,
            KeyCode.CLIPBOARD_PASTE, KeyCode.CLIPBOARD_CUT, KeyCode.CLIPBOARD_SELECT_ALL,
            KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP, KeyCode.TOGGLE_AUTOCORRECT, KeyCode.IME_UI_MODE_CLIPBOARD,
            KeyCode.PHONE_PAUSE, KeyCode.SPACE, KeyCode.SHIFT, KeyCode.ENTER, KeyCode.DELETE)
        val TEXTS = listOf("", "a", "ab", "?", "null", " ", " x", "\t", " ", "\u0085", "\u001c", " ", "é",
            "😀", "😀x", "\uD83D", "\uD83Dx", "\uDE00")
        val KEY_LABELS = listOf("a", "Ab", "?", "😀", "\u0301")
        val LEVELS = listOf(Float.NaN, -0.5f, -0f, 0f, 1e-5f, 0.25f, 0.5f, 0.999f, 1f, 1.5f,
            Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY)
        val ICON_NAMES = mapOf(SmartbarIcon.KEY to "key", SmartbarIcon.ARROW_LEFT to "arrow left",
            SmartbarIcon.ARROW_RIGHT to "arrow right", SmartbarIcon.UNFOLD_LESS to "unfold less",
            SmartbarIcon.UNFOLD_MORE to "unfold more", SmartbarIcon.INCOGNITO to "incognito",
            SmartbarIcon.STOP to "stop")
        val FLAG_NAMES = mapOf(BAR_ENABLED to "enabled", BAR_DICTATING to "dictating", BAR_READING to "reading",
            BAR_WARMING to "warming", BAR_FINISHING to "finishing", (BAR_LEVEL_MASK shl BAR_LEVEL_SHIFT) to "level")
    }
}
