package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.settings.SettingsComponentSerializer
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import kotlinx.serialization.json.*
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/** The theme manager routes Rust draws, in the order it lists them. */
internal val themeRoutes = listOf("settings/theme/manage/SELECT_DAY", "settings/theme/manage/SELECT_NIGHT")

/**
 * The theme manager through the real native settings, its report built by SettingsThemes' own [themeRow], against
 * what the frozen ThemeManagerScreen did with the same index: the same boxes in the same order, and the same
 * preference value written when a theme is tapped. Every id and label is made up.
 */
class SettingsThemesMatchTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) =
        Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content

    /** An extension of the index: its id, title and themes as (id, label, night). */
    private data class Ext(val id: String, val title: String, val themes: List<Triple<String, String, Boolean>>)

    /** ThemeManagerScreen's grouping: `buildMap { put(ext.meta.id, ext.themes) }`, each sorted by label. */
    private fun reference(index: List<Ext>): List<Pair<String, List<String>>> =
        buildMap { for (ext in index) put(ext.id, ext.themes) }.mapValues { (_, t) -> t.sortedBy { it.second } }
            .map { (id, t) -> id to t.map { it.second } }

    private fun report(index: List<Ext>) = index.flatMap { ext ->
        themeRow(SettingsUiThemeKind.EXTENSION, ext.id, label = ext.title).toList() +
            ext.themes.flatMap { (id, label, night) -> themeRow(SettingsUiThemeKind.THEME, ext.id, id, label, night).toList() }
    }.toTypedArray()

    /** Native settings on the day theme manager with [index] reported. */
    private fun open(n: KeyboardRenderer, index: List<Ext>, route: String = themeRoutes[0]): JsonObject {
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true")).toTypedArray())
        assertEquals(listOf(SettingsUiThemeEffect.LOAD), effects(state(n, SettingsUiOp.NAVIGATE, route))[0])
        return state(n, SettingsUiOp.THEMES, *report(index))
    }

    @Test fun theBoxesListTheIndexAsThemeManagerScreenGroupedAndSortedIt() {
        val random = Random(4242)
        val labels = listOf("Zed", "apple", "Éclair", "Ölfarbe", "zed", "Zed", "🌙 Moon", "ﬃ", "a", "A", "")
        repeat(40) {
            val ids = listOf("a.ext", "b.ext", "c.ext").shuffled(random)
            val index = List(random.nextInt(1, 5)) {
                val id = ids.random(random)
                Ext(id, "Title $id", List(random.nextInt(0, 5)) { t -> Triple("t$t", labels.random(random), random.nextBoolean()) })
            }
            KeyboardRenderer(arrayOf(font)).use { n ->
                val radios = nodes(open(n, index)).filter { it.text("role") == "radio" }.map { it.text("label") }
                assertEquals(reference(index).flatMap { it.second }, radios)
            }
        }
    }

    @Test fun aTapWritesWhatTheComponentSerializerStoresAndThePreviewNamesIt() {
        val index = listOf(Ext("my.themes", "Mine", listOf(Triple("sunny", "Sunny", false), Triple("dusk", "Dusk", true))))
        for ((route, key) in themeRoutes.zip(listOf("theme__day_theme_id", "theme__night_theme_id"))) {
            KeyboardRenderer(arrayOf(font)).use { n ->
                val sunny = nodes(open(n, index, route)).single { it.text("label") == "Sunny" }
                val tapped = state(n, SettingsUiOp.ACTIVATE, sunny.getValue("id").jsonPrimitive.content)
                val e = effects(tapped)
                val name = ExtensionComponentName("my.themes", "sunny")
                assertEquals(listOf(listOf("write", key, SettingsComponentSerializer.serialize(name)),
                    listOf(SettingsUiThemeEffect.PREVIEW, name.toString())), e)
                assertEquals(name, SettingsComponentSerializer.deserialize(e[0][2]))
                assertEquals(name, ExtensionComponentName.from(e[1][1]))
                assertTrue(nodes(tapped).single { it.text("label") == "Sunny" }.getValue("selected").jsonPrimitive.boolean)
            }
        }
    }

    @Test fun eachThemeTellsTalkBackWhetherItIsADayOrNightTheme() {
        val index = listOf(Ext("x", "X", listOf(Triple("d", "Light", false), Triple("n", "Dark", true))))
        KeyboardRenderer(arrayOf(font)).use { n ->
            n.settings(SettingsUiOp.RESOURCES, arrayOf("pref__theme__day", "Day theme", "pref__theme__night", "Night theme"))
            val radios = nodes(open(n, index)).filter { it.text("role") == "radio" }
            assertEquals(listOf("Dark" to "Night theme", "Light" to "Day theme"), radios.map { it.text("label") to it.text("state") })
        }
    }
}
