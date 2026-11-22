package com.jtn0123.audiowhisper.render

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.VisibilityOff
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.reader.ReadAloud
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import com.jtn0123.audiowhisper.rust.SmartbarIcon
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.smartbar.IncognitoDisplayMode
import dev.patrickgold.florisboard.ime.smartbar.SmartbarLayout
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlin.math.absoluteValue
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Seeded smartbars, each shown by main's choices (frozen in [referenceSmartbarActions]) and by the
 * Rust plan through [SmartbarShown], must agree on every action's code, flags, icon and label and
 * on what the arrow toggles. Two labels differ on purpose, each counted: an empty text shows "?"
 * where main showed "null", and a text starting with a surrogate pair shows the whole character
 * where main showed half of it. See docs/phase3-typing-fan-out.md.
 */
class SmartbarActionsMatchTest {
    private val seen = mutableMapOf<String, Int>()
    private fun saw(what: String) { seen[what] = (seen[what] ?: 0) + 1 }

    @Test fun mainsChoicesAndTheRustPlanShowTheSameSmartbar() {
        val random = Random(SEED)
        repeat(CASES) { case -> check(case, random) }
        println("Smartbar actions: $CASES seeded smartbars identical but for the two label fixes; $seen")
        for (icon in ICON_NAMES.keys) assertTrue((seen["icon ${ICON_NAMES[icon]}"] ?: 0) > 0, "never showed ${ICON_NAMES[icon]}")
        for (flag in FLAG_NAMES.keys) assertTrue((seen["flag ${FLAG_NAMES[flag]}"] ?: 0) > 0, "never set ${FLAG_NAMES[flag]}")
        for (what in listOf("empty text", "surrogate pair", "blank text", "key label", "expanded", "toggles extended",
            "preview of shared actions")) {
            assertTrue((seen[what] ?: 0) > 0, "never saw $what")
        }
    }

    private fun check(case: Int, random: Random) {
        val facts = facts(random)
        val actions = List(5 + random.nextInt(7)) { action(random) }
        val disabled = CODES.filter { random.nextInt(4) == 0 }.toSet()
        val plain = CODES.filter { random.nextInt(3) == 0 }.toSet()
        val evaluator = FakeEvaluator({ it.code !in disabled }, { if (it.code in plain) null else KEY_ICONS[it.code.absoluteValue % KEY_ICONS.size] },
            { if (it.code % 5 == 0) null else "key ${it.code}" })
        val reference = referenceSmartbarActions(facts, actions, evaluator, INCOGNITO)
        val shown = with(facts) {
            SmartbarShown(config(), smartbarBar(incognitoMode, incognito, dictation, level, warming), actions, evaluator.enabled,
                evaluator.keyIcon)
        }
        assertEquals(reference.expanded, shown.expanded, "case $case: expanded differs")
        assertEquals(reference.togglesExtended, shown.togglesExtended, "case $case: the toggle's target differs")
        assertContentEquals(reference.codes, shown.codes, "case $case: codes or flags differ")
        for (i in actions.indices) {
            assertEquals(reference.labels[i].first, shown.icon(i, INCOGNITO), "case $case action $i: icon differs")
            assertEquals(intended(actions[i], reference.labels[i].second), shown.label(i, evaluator.keyLabel), "case $case action $i: label differs")
        }
        count(facts, shown, actions)
    }

    /** Main's label, but for the two fixes. */
    private fun intended(action: QuickAction?, main: String): String {
        val text = (action as? QuickAction.InsertText)?.data ?: return main.also { if (action is QuickAction.InsertKey && it.startsWith("key")) saw("key label") }
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
        else -> QuickAction.InsertKey(TextKeyData(code = CODES.random(random)))
    }

    private companion object {
        const val SEED = 20261122L
        const val CASES = 20_000
        val INCOGNITO = Icons.Default.VisibilityOff
        val KEY_ICONS = listOf(Icons.Default.ContentCopy, Icons.Default.Settings, Icons.Default.Mic)
        val CODES = listOf(KeyCode.VOICE_INPUT, KeyCode.READ_ALOUD, KeyCode.TOGGLE_ACTIONS_OVERFLOW, KeyCode.UNDO,
            KeyCode.REDO, KeyCode.SETTINGS, KeyCode.CLIPBOARD_COPY, KeyCode.ARROW_LEFT, KeyCode.TOGGLE_INCOGNITO_MODE,
            KeyCode.UNSPECIFIED, 'a'.code, 'é'.code)
        val TEXTS = listOf("", "a", "ab", "?", "null", " ", " x", "\t", " ", "\u0085", "\u001c", " ", "é",
            "😀", "😀x", "\uD83D", "\uD83Dx", "\uDE00")
        val LEVELS = listOf(Float.NaN, -0.5f, -0f, 0f, 1e-5f, 0.25f, 0.5f, 0.999f, 1f, 1.5f,
            Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY)
        val ICON_NAMES = mapOf(SmartbarIcon.KEY to "key", SmartbarIcon.ARROW_LEFT to "arrow left",
            SmartbarIcon.ARROW_RIGHT to "arrow right", SmartbarIcon.UNFOLD_LESS to "unfold less",
            SmartbarIcon.UNFOLD_MORE to "unfold more", SmartbarIcon.INCOGNITO to "incognito", SmartbarIcon.STOP to "stop")
        val FLAG_NAMES = mapOf(BAR_ENABLED to "enabled", BAR_DICTATING to "dictating", BAR_READING to "reading",
            BAR_WARMING to "warming", BAR_FINISHING to "finishing", (BAR_LEVEL_MASK shl BAR_LEVEL_SHIFT) to "level")
    }
}
