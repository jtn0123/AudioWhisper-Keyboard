package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.settingsui.ThemeEditorNatives.removeProperty
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.settings.theme.CustomRuleComparator
import dev.patrickgold.florisboard.app.settings.theme.PrettyPrintConfig
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import kotlinx.serialization.json.*
import org.florisboard.lib.snygg.*
import org.florisboard.lib.snygg.value.*
import java.io.File
import kotlin.test.*

/** The theme editor's routes Rust draws, in the order it lists them: the component's rules, then its editors. */
internal val themeEditorRoutes = listOf("settings/theme/edit", "settings/theme/editor/rule/add", "settings/theme/editor/rule",
    "settings/theme/editor/meta", "settings/theme/editor/tune")

/** The theme editor's native routes and helpers its match tests share. */
internal object ThemeEditorNatives {
    const val UUID = "3f1c-workspace"
    const val ROUTE = "settings/theme/edit/$UUID/0"
    const val SAVE_ID = 400_040
    val font: String = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) = Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    /** The stylesheet the last [kind] effect of [s] carries. */
    fun json(s: JsonObject, kind: String) = effects(s).last { it[0] == kind }[3]
    /** Rule `r`'s `k`th property's Delete, as `paint::rule_id` numbers it. */
    fun deleteId(r: Int, k: Int) = (500_000 + 4096 * r + 65 + 2 * k).toString()

    /** RustSettingsView.configure's CONFIG for a tall phone at density 1. */
    fun configure(n: KeyboardRenderer) {
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        state(n, SettingsUiOp.CONFIG, *(listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true", "4278190080", "4278198272", "4278190080")).toTypedArray())
    }
    /** Native settings on the editor of component 0, asked for it. */
    fun open(n: KeyboardRenderer) {
        configure(n)
        assertEquals(listOf(SettingsUiThemeEditorEffect.LOAD, UUID, "0"), effects(state(n, SettingsUiOp.NAVIGATE, ROUTE)).single())
    }
    /** SettingsThemeEditor's report of component 0 with [sheet] as its stylesheet. */
    fun report(sheet: String) = themeEditorRow(SettingsUiThemeEditorOutcome.LOADED, UUID, "0",
        SettingsUiThemeEditor.STYLESHEET to sheet, SettingsUiThemeEditor.EXTENSION to "local.my.themes",
        SettingsUiThemeEditor.ID to "sunny", SettingsUiThemeEditor.LABEL to "Sunny", SettingsUiThemeEditor.AUTHORS to "Ada",
        SettingsUiThemeEditor.NIGHT to WireFlag.encode(false), SettingsUiThemeEditor.PATH to "stylesheets/sunny.json")
    fun loaded(n: KeyboardRenderer, sheet: String) = state(n, SettingsUiOp.THEME_EDITOR, *report(sheet))

    /** loadThemeEditorStylesheet's editor for [sheet]: parsed with PrettyPrintConfig, ordered by CustomRuleComparator,
     * with `@defines` added when it had none. */
    fun composeEditor(sheet: String) = SnyggStylesheet.fromJson(sheet, PrettyPrintConfig).getOrThrow().edit(CustomRuleComparator)
        .also { it.rules.putIfAbsent(SnyggAnnotationRule.Defines, SnyggSinglePropertySetEditor()) }
    /** What ThemeEditorScreen wrote for [sheet] after [edit]: its editor built and `toJson(PrettyPrintConfig)`. */
    fun composeWrites(sheet: String, edit: SnyggStylesheetEditor.() -> Unit = {}) =
        composeEditor(sheet).apply(edit).build().toJson(PrettyPrintConfig).getOrThrow()
    /** The first [count] rules of [sheet] with a single set of at least [size] properties, by index in the list. */
    fun fullRules(sheet: String, count: Int, size: Int = 3) = composeEditor(sheet).rules.values.withIndex()
        .filter { ((it.value as? SnyggSinglePropertySetEditor)?.properties?.size ?: 0) >= size }.map { it.index }.take(count)
    /** Removes the `k`th property of the `r`th rule, as ThemeEditorScreen's property delete did. */
    fun SnyggStylesheetEditor.removeProperty(r: Int, k: Int) {
        val set = rules.values.elementAt(r) as SnyggSinglePropertySetEditor
        set.properties.remove(set.properties.keys.elementAt(k))
    }
}

/**
 * The theme editor's stylesheet through the real natives against Compose's own writer: every shipped stylesheet and
 * stylesheets Compose's editor changed come out of Rust as the bytes `SnyggStylesheet.toJson(PrettyPrintConfig)`
 * wrote, so a theme saved by Rust opens unchanged in an older build; and Rust's element and property names are
 * FlorisImeUi's and ThemeTranslations'.
 */
class SettingsThemeEditorMatchTest {
    private val shipped = File("src/main/assets/ime/theme").walk().filter { it.parentFile.name == "stylesheets" && it.extension == "json" }
        .sortedBy { it.path }.toList()

    @Test fun everyShippedStylesheetIsWrittenAsComposeWroteIt() {
        assertEquals(12, shipped.size)
        for (file in shipped) {
            val sheet = file.readText()
            KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
                ThemeEditorNatives.open(n)
                val loaded = ThemeEditorNatives.loaded(n, sheet)
                val expected = ThemeEditorNatives.composeWrites(sheet)
                assertEquals(expected, ThemeEditorNatives.json(loaded, SettingsUiThemeEditorEffect.PREVIEW), file.name)
                val saved = ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, ThemeEditorNatives.SAVE_ID.toString())
                assertEquals(expected, ThemeEditorNatives.json(saved, SettingsUiThemeEditorEffect.SAVE), file.name)
            }
        }
    }

    /** Attributes as EditRuleDialog built them, a value at a time with `including`. */
    private fun attrs(vararg pairs: Pair<String, Any>) = SnyggElementRule("key").attributes.including(*pairs)
    /** Rules added and renamed with `updateRule`, a rule and properties removed, and values of every number form. */
    private fun SnyggStylesheetEditor.composeEdits() {
        assertTrue(updateRule(SnyggElementRule("key"), SnyggElementRule("key", attrs("code" to -1, "code" to 32, "code" to 10),
            SnyggSelector.PRESSED), allowAdd = true))
        assertTrue(updateRule(SnyggElementRule("smartbar"), SnyggElementRule("smartbar", attrs("mode" to "numeric"),
            SnyggSelector.DISABLED), allowAdd = true))
        rules.remove(rules.keys.first { it is SnyggElementRule && it.elementName == "window" })
        val (first, second) = rules.values.withIndex().filter { ((it.value as? SnyggSinglePropertySetEditor)?.properties?.size ?: 0) >= 2 }
            .map { it.index }
        removeProperty(second, 1)
        removeProperty(first, 0)
        val added = SnyggElementRule("glide-trail", attrs("shiftstate" to "unshifted", "shiftstate" to "caps_lock"), SnyggSelector.FOCUS)
        rules[added] = SnyggSinglePropertySetEditor().apply {
            "background" to SnyggStaticColorValue(Color(12, 34, 56, 128))
            "font-size" to SnyggSpSizeValue(14.5.sp)
            "shadow-elevation" to SnyggDpSizeValue(0.1.dp)
            "letter-spacing" to SnyggSpSizeValue(1.25.sp)
            "border-width" to SnyggDpSizeValue(12.dp)
            "text-max-lines" to SnyggTextMaxLinesValue(3)
            "shape" to SnyggRoundedCornerPercentShapeValue(10, 20, 30, 40)
            "foreground" to SnyggDefinedVarValue("--primary")
        }
        rules[SnyggAnnotationRule.Font("Mono Two")] = SnyggMultiplePropertySetsEditor().apply {
            add { "src" to SnyggUriValue("flex:/fonts/mono_two.ttf"); "font-weight" to SnyggFontWeightValue(androidx.compose.ui.text.font.FontWeight.Bold) }
            add { "src" to SnyggUriValue("flex:/fonts/mono-two-bold.ttf") }
        }
    }

    @Test fun aStylesheetComposeEditedIsReadBackAndWrittenUnchanged() {
        for (file in shipped.take(4)) {
            val edited = ThemeEditorNatives.composeWrites(file.readText()) { composeEdits() }
            KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
                ThemeEditorNatives.open(n)
                val loaded = ThemeEditorNatives.loaded(n, edited)
                assertEquals(edited, ThemeEditorNatives.json(loaded, SettingsUiThemeEditorEffect.PREVIEW), file.name)
                assertEquals(edited, ThemeEditorNatives.composeWrites(edited), "Compose's own output is its fixed point")
            }
        }
    }

    @Test fun propertiesDeletedInRustAreWrittenAsComposeWroteThemWithout() {
        val sheet = shipped.first { it.name == "floris_day.json" }.readText()
        val (a, b, c) = ThemeEditorNatives.fullRules(sheet, 3)
        // Deleted in this order, a later index is the property list as the earlier deletes left it.
        for (picks in listOf(listOf(a to 2), listOf(b to 1, b to 1), listOf(c to 0, a to 1, b to 0))) {
            KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
                ThemeEditorNatives.open(n)
                ThemeEditorNatives.loaded(n, sheet)
                for ((r, k) in picks) ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, ThemeEditorNatives.deleteId(r, k))
                val saved = ThemeEditorNatives.state(n, SettingsUiOp.ACTIVATE, ThemeEditorNatives.SAVE_ID.toString())
                val expected = ThemeEditorNatives.composeWrites(sheet) { for ((r, k) in picks) removeProperty(r, k) }
                assertEquals(expected, ThemeEditorNatives.json(saved, SettingsUiThemeEditorEffect.SAVE), "$picks")
            }
        }
    }

    private val names: String by lazy { File("../lib/rust/crates/render/src/settings_ui/theme_editor/names.rs").readText() }
    /** The strings of Rust array `name` in names.rs. */
    private fun rustArray(name: String) = Regex("""pub const $name: \[[^\]]+\] = \[(.*?)\];""", RegexOption.DOT_MATCHES_ALL)
        .find(names)!!.groupValues[1]
    private val stringNames: Map<Int, String> by lazy { R.string::class.java.fields.associate { it.getInt(null) to it.name } }

    @Test fun elementsAreFlorisImeUisInItsOrderWithItsStrings() {
        val elements = Regex("\"([^\"]+)\"").findAll(rustArray("ELEMENTS")).map { it.groupValues[1] }.toList()
        assertEquals(FlorisImeUi.elementNames, elements)
        val keys = Regex("\"([^\"]+)\"").findAll(rustArray("ELEMENT_KEYS")).map { it.groupValues[1] }.toList()
        assertEquals(elements.map { stringNames.getValue(checkNotNull(FlorisImeUi.elementNamesToTranslation.getValue(it))) }, keys)
    }

    @Test fun propertiesAreThemeTranslationsNamesWithItsStrings() {
        val field = Class.forName("dev.patrickgold.florisboard.app.settings.theme.ThemeTranslationsKt").getDeclaredField("PropertyNameMap")
        field.isAccessible = true
        val map = (field.get(null) as Map<*, *>).map { (k, v) -> k as String to stringNames.getValue(v as Int) }
        val rust = Regex("""\(\s*"([^"]+)",\s*"([^"]+)",?\s*\)""").findAll(rustArray("PROPERTIES")).map { it.groupValues[1] to it.groupValues[2] }.toList()
        assertEquals(map, rust)
    }
}
