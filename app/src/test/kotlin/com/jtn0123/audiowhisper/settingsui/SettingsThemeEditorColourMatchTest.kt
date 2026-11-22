package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.graphics.Color
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import dev.patrickgold.jetpref.material.ui.parseOrNull
import kotlinx.serialization.json.*
import org.florisboard.lib.snygg.SnyggAnnotationRule
import org.florisboard.lib.snygg.SnyggSinglePropertySetEditor
import org.florisboard.lib.snygg.value.SnyggStaticColorValue
import kotlin.random.Random
import kotlin.test.*

/**
 * The theme editor's static colours through the real natives against Jetpref and Compose: the rules list, the property
 * editor's value row and the colour picker's field show a colour in each of fine-tune's representations as
 * `ColorRepresentation.formatColor(color, withAlpha = true)` writes it, and the picker takes the texts
 * `Color.parseOrNull(text, withAlpha = true)` reads, as the colours it reads.
 */
class SettingsThemeEditorColourMatchTest {
    private val base = """{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","@defines":{},"key":{}}"""
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    private fun node(s: JsonObject, id: Int) = nodes(s).firstOrNull { it.getValue("id").jsonPrimitive.int == id }
    private fun represent(n: KeyboardRenderer, rep: ColorRepresentation) =
        n.settings(SettingsUiOp.PREFERENCES, arrayOf("theme__editor_color_representation", rep.name))

    /** Every alpha byte on a random colour, and the corners of the cube. */
    private fun colours(random: Random) = (0..255).map { Color(random.nextInt(256), random.nextInt(256), random.nextInt(256), it) } +
        listOf(Color.Black, Color.White, Color.Red, Color.Green, Color.Blue, Color(0xff808080), Color.Transparent, Color(0x80ff00ff))

    @Test fun theRulesListShowsStaticColoursAsFormatColorWithAlpha() {
        for (chunk in colours(Random(20261122)).chunked(66)) {
            val sheet = ThemeEditorNatives.composeWrites(base) {
                val defines = rules.getValue(SnyggAnnotationRule.Defines) as SnyggSinglePropertySetEditor
                chunk.forEachIndexed { i, c -> defines.properties["--c$i"] = SnyggStaticColorValue(c) }
            }
            for (rep in ColorRepresentation.entries) KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
                ThemeEditorNatives.open(n)
                represent(n, rep)
                val shown = nodes(ThemeEditorNatives.loaded(n, sheet)).associate { it.text("label") to it.text("state") }
                chunk.forEachIndexed { i, c -> assertEquals(rep.formatColor(c, true), shown["--c$i"], "$rep $c") }
            }
        }
    }

    /** Texts for the theme picker's field: every form Jetpref reads with an alpha, and near misses of each. */
    private fun texts(random: Random): List<String> {
        val alphas = listOf("0", "1", "1.0", "0.5", ".5", "0.25", "0.333", ".05", "0.999", "0.004", "0.", "1.5", "2", "-0.5",
            "00.5", "1.00", "0.5f", "1e-1", " 0.5 ", "")
        val out = mutableListOf("#ff000080", "#FF000080", "#ff0000", "#00000000", " #12345678\n", "#ff00008", "#ff0000800",
            "#ff0000 80", "rgba(1, 2, 3, 0.5)", "rgb(1, 2, 3, 0.5)", "rgba(1, 2, 3)", "rgba(1, 2, 3, 0.5, 1)", "hsva(120, 50, 50, .9)",
            "hsv(0, 0, 0, 1)", "HSVA(359, 100, 100, 0)", "rgba(1,2,3,0.123456789)", "rgba (1, 2, 3, 0.5)", "red", "")
        repeat(60) { out += "#%08x".format(random.nextInt()).let { if (random.nextBoolean()) it.uppercase() else it } }
        repeat(60) { out += "rgba(${random.nextInt(256)}, ${random.nextInt(256)}, ${random.nextInt(256)}, ${alphas.random(random)})" }
        repeat(60) { out += "hsva(${random.nextInt(361)}, ${random.nextInt(101)}, ${random.nextInt(101)}, ${alphas.random(random)})" }
        return out
    }

    @Test fun thePickerTakesWhatParseOrNullWithAlphaTakesAndTheValueRowShowsIt() {
        val reps = ColorRepresentation.entries
        val key = ThemeEditorNatives.composeEditor(base).rules.keys.indexOfFirst { it !is SnyggAnnotationRule }
        KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
            ThemeEditorNatives.open(n)
            ThemeEditorNatives.loaded(n, base)
            ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, (500_000 + 4096 * key + 4).toString())
            ThemeEditorNatives.choose(n, 2, "snygg__property_name__background")
            ThemeEditorNatives.choose(n, 3, "snygg__property_value__solid_color")
            for ((i, text) in texts(Random(20261123)).withIndex()) {
                represent(n, reps[i % reps.size])
                val applied = ThemeEditorNatives.pickColor(n, 4, text, confirm = false)
                val parsed = Color.parseOrNull(text, true)
                assertEquals(parsed != null, node(applied, 7140) != null, "the picker took \"$text\"")
                if (parsed == null) {
                    ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, "6")
                    continue
                }
                val (color, typed) = parsed
                val state = ReferenceOtherThemeScreens.PickerState(color).color()
                assertEquals(typed.formatColor(state, true), node(applied, 7000)?.text("state"), text)
                val row = node(ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, "7"), 1004)
                assertEquals(reps[i % reps.size].formatColor(color, true), row?.text("state"), text)
            }
        }
    }
}
