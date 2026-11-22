package com.jtn0123.audiowhisper.settingsui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.settingsui.ThemeEditorNatives.removeProperty
import dev.patrickgold.florisboard.R
import com.jtn0123.audiowhisper.settingsui.ReferenceThemeEditor.CustomRuleComparator
import dev.patrickgold.florisboard.app.PrettyPrintConfig
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import kotlinx.serialization.json.*
import org.florisboard.lib.snygg.*
import org.florisboard.lib.snygg.value.*
import java.io.File
import kotlin.test.*

/** The theme editor's routes Rust draws, in the order it lists them: the component's rules, then its editors. */
internal val themeEditorRoutes = listOf("settings/theme/edit", "settings/theme/editor/rule/add", "settings/theme/editor/rule",
    "settings/theme/editor/property/add", "settings/theme/editor/property", "settings/theme/editor/code/add",
    "settings/theme/editor/code", "settings/theme/editor/meta", "settings/theme/editor/tune")

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
    /** Opens the property editor's dropdown on row [row] and taps its entry labelled [label]. */
    fun choose(n: KeyboardRenderer, row: Int, label: String) {
        val open = state(n, SettingsUiOp.ACTIVATE, (1000 + row).toString())
        val entries = open.getValue("nodes").jsonArray.map { it.jsonObject }.filter { it.getValue("id").jsonPrimitive.int >= 3000 }
        val entry = assertNotNull(entries.firstOrNull { it.getValue("label").jsonPrimitive.content == label },
            "$label is not in row $row's ${entries.map { it.getValue("label").jsonPrimitive.content }}")
        state(n, SettingsUiOp.ACTIVATE, entry.getValue("id").jsonPrimitive.content)
    }
    /** Row [row]'s static colour picked as [text]: the colour picker opened, [text] typed in its field, Apply changes and
     * the picker's OK; the state after Apply changes, whose picker is still open when it refused [text]. */
    fun pickColor(n: KeyboardRenderer, row: Int, text: String, confirm: Boolean = true): JsonObject {
        for (id in listOf(1000 + row, 7140, 7000)) state(n, SettingsUiOp.ACTIVATE, id.toString())
        state(n, SettingsUiOp.EDIT, "set", text)
        val applied = state(n, SettingsUiOp.ACTIVATE, "7141")
        if (confirm) state(n, SettingsUiOp.ACTIVATE, "7")
        return applied
    }

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
    private val shipped = File("src/main/assets/ime/theme").walk().filter { it.parentFile?.name == "stylesheets" && it.extension == "json" }
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
    /** Rules added and renamed with `updateRule`, a rule and properties removed, values of every number form, and empty rules. */
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
        // Add rule's rules, which have no properties yet.
        rules[SnyggElementRule("media", selector = SnyggSelector.HOVER)] = SnyggSinglePropertySetEditor()
        rules[SnyggAnnotationRule.Font("Empty")] = SnyggMultiplePropertySetsEditor()
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

    /** A property added in Rust through the property editor: its rule's element, its name, its encoder's string key's
     * end, the entry chosen in the value dropdown or the text typed in its fields (a static colour's in the colour
     * picker); and the value Compose's dialog built. */
    private class Added(val element: String, val name: String, val encoder: String, val value: SnyggValue,
        val pick: String? = null, val typed: List<String> = emptyList())

    private val valueKinds = listOf(
        Added("key", "foreground", "explicit_inherit", SnyggInheritValue),
        Added("key", "clip", "yes", SnyggYesValue),
        Added("key", "shape", "circle_shape", SnyggCircleShapeValue()),
        Added("key", "font-family", "font_family_generic", SnyggGenericFontFamilyValue(FontFamily.Monospace), "monospace"),
        Added("key", "font-style", "font_style", SnyggFontStyleValue(FontStyle.Italic), "italic"),
        Added("key", "font-weight", "font_weight", SnyggFontWeightValue(FontWeight.SemiBold), "semi-bold"),
        Added("key", "text-align", "text_align", SnyggTextAlignValue(TextAlign.Center), "center"),
        Added("key", "text-decoration-line", "text_decoration_line", SnyggTextDecorationLineValue(TextDecoration.Underline), "underline"),
        Added("key", "text-overflow", "text_overflow", SnyggTextOverflowValue(TextOverflow.Ellipsis), "ellipsis"),
        Added("key", "content-scale", "content_scale", SnyggContentScaleValue(ContentScale.Crop), "crop"),
        // The file selector's `"flex:" + Uri.encode("/images/sky blue.png", "/")`.
        Added("key", "background-image", "uri", SnyggUriValue("flex:/images/sky%20blue.png"), "flex:/images/sky%20blue.png"),
        Added("key", "border-width", "dp_size", SnyggDpSizeValue(2.5.dp), typed = listOf("2.5")),
        Added("key", "font-size", "sp_size", SnyggSpSizeValue(14.5.sp), typed = listOf("14.5")),
        Added("key", "text-max-lines", "text_max_lines", SnyggTextMaxLinesValue(3), typed = listOf("3")),
        Added("key", "padding", "padding", SnyggPaddingValue(PaddingValues(1.dp, 2.5.dp, 3.dp, 4.dp)), typed = listOf("1", "2.5", "3", "4")),
        Added("key", "background", "solid_color", SnyggStaticColorValue(Color(12, 34, 56, 128)), typed = listOf("#0c223880")),
        Added("key", "border-color", "material_you_light_color", SnyggDynamicLightColorValue("onSurface"), "onSurface"),
        Added("key", "shadow-color", "material_you_dark_color", SnyggDynamicDarkColorValue("primary")),
        Added("key", "line-height", "defined_var", SnyggDefinedVarValue("--primary"),
            "snygg__property_name__var_primary"),
        Added("key-hint", "shape", "rectangle_shape", SnyggRectangleShapeValue()),
        Added("key-hint", "font-family", "font_family_custom", SnyggCustomFontFamilyValue("Mono"), "Mono"),
        Added("key-hint", "clip", "no", SnyggNoValue),
        Added("key-hint", "text-max-lines", "text_max_lines", SnyggTextMaxLinesValue(Int.MAX_VALUE)),
        Added("root", "shape", "rounded_corner_shape_dp", SnyggRoundedCornerDpShapeValue(1.dp, 2.dp, 3.dp, 4.5.dp),
            typed = listOf("1", "2", "3", "4.5")),
        Added("window", "shape", "rounded_corner_shape_percent", SnyggRoundedCornerPercentShapeValue(10, 20, 30, 40),
            typed = listOf("10", "20", "30", "40")),
        Added("key-popup-box", "shape", "cut_corner_shape_dp", SnyggCutCornerDpShapeValue(1.dp, 2.dp, 3.dp, 4.dp),
            typed = listOf("1", "2", "3", "4")),
        Added("key-popup-element", "shape", "cut_corner_shape_percent", SnyggCutCornerPercentShapeValue(5, 6, 7, 60),
            typed = listOf("5", "6", "7", "60")),
    )
    private val editSheet = """{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","@defines":{"--primary":"#ff0000"},
        "@font `Mono`":[{"src":"uri(`flex:/fonts/mono.ttf`)"}],"root":{},"window":{},"key":{},"key-hint":{},
        "key-popup-box":{},"key-popup-element":{}}"""

    private fun KeyboardRenderer.choose(row: Int, label: String) = ThemeEditorNatives.choose(this, row, label)
    /** Focuses the field on row [row] and gives it [text], as the IME's InputConnection does. */
    private fun KeyboardRenderer.type(row: Int, text: String) {
        ThemeEditorNatives.state(this, SettingsUiOp.ACTIVATE, (1000 + row).toString())
        ThemeEditorNatives.state(this, SettingsUiOp.EDIT, "set", text)
    }
    private fun KeyboardRenderer.tap(id: Int) = ThemeEditorNatives.state(this, SettingsUiOp.ACTIVATE, id.toString())
    /** The bottom bar's Add or Apply on row 12, which goes back to the rules unless it refused [what]. */
    private fun KeyboardRenderer.apply(what: String) =
        assertEquals(ThemeEditorNatives.ROUTE, tap(1012).getValue("route").jsonPrimitive.content, "$what was refused")
    /** The property editor's rows: the name 2, the encoder 3, the value 4, the sides 5 to 8, and Add or Apply 12. */
    private fun KeyboardRenderer.add(rule: Int, a: Added) {
        tap(500_000 + 4096 * rule + 4)
        choose(2, "snygg__property_name__" + a.name.replace('-', '_'))
        choose(3, "snygg__property_value__" + a.encoder)
        a.pick?.let { choose(4, it) }
        when {
            a.encoder == "solid_color" -> ThemeEditorNatives.pickColor(this, 4, a.typed[0])
            a.typed.size == 1 -> type(4, a.typed[0])
            else -> a.typed.forEachIndexed { k, t -> type(5 + k, t) }
        }
        apply("${a.element} ${a.name}")
    }
    private fun SnyggStylesheetEditor.element(name: String) =
        rules.entries.first { (it.key as? SnyggElementRule)?.elementName == name }.value as SnyggSinglePropertySetEditor

    /** Codes edited in Rust's code editor, written as EditRuleDialog's EditCodeValueDialog callbacks changed them:
     * each code added with `including`, an edited code `excluding` the old one first, and Delete `excluding` it. */
    @Test fun codesEditedInRustAreWrittenAsComposesDialogWroteThem() {
        val sheet = """{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key[code=10]":{"background":"#0000ff"},"key":{}}"""
        val rule = ThemeEditorNatives.composeEditor(sheet).rules.keys.indexOfFirst { (it as? SnyggElementRule)?.attributes?.isEmpty() == false }
        KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
            ThemeEditorNatives.open(n)
            ThemeEditorNatives.loaded(n, sheet)
            n.tap(500_000 + 4096 * rule + 2)
            // Add code, with two at once; codes are chips from 401_000 in order, and the code editor's Save is row 4.
            n.tap(400_310)
            n.type(1, "13, 32")
            n.tap(1004)
            n.tap(401_000)
            n.type(1, "-7")
            n.tap(1004)
            // Edit 32, whose top bar Delete names it.
            val delete = n.tap(401_002).getValue("nodes").jsonArray.map { it.jsonObject }.first { it.getValue("id").jsonPrimitive.int == 400_041 }
            assertEquals("32", delete.getValue("state").jsonPrimitive.content)
            n.tap(400_041)
            // The rule editor's Apply, row 8, then Save.
            n.tap(1008)
            val saved = n.tap(ThemeEditorNatives.SAVE_ID)
            val expected = ThemeEditorNatives.composeWrites(sheet) {
                val old = rules.keys.first { (it as? SnyggElementRule)?.attributes?.isEmpty() == false } as SnyggElementRule
                var attributes = old.attributes.including("code" to "13").including("code" to "32")
                attributes = attributes.excluding("code" to "10").including("code" to "-7")
                attributes = attributes.excluding("code" to "32")
                assertTrue(updateRule(old, old.copy(attributes = attributes)))
            }
            assertEquals(expected, ThemeEditorNatives.json(saved, SettingsUiThemeEditorEffect.SAVE))
            assertTrue("key[code=-7,13]" in expected, expected)
        }
    }

    @Test fun propertiesEditedInRustAreWrittenAsComposesDialogWroteThem() {
        val order = ThemeEditorNatives.composeEditor(editSheet).rules.keys.toList()
        val index = { element: String -> order.indexOfFirst { (it as? SnyggElementRule)?.elementName == element } }
        val (defines, font) = order.indexOf(SnyggAnnotationRule.Defines) to order.indexOf(SnyggAnnotationRule.Font("Mono"))
        KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
            ThemeEditorNatives.open(n)
            val report = ThemeEditorNatives.report(editSheet)
            report[SettingsUiThemeEditor.FONTS] = "mono bold.ttf"
            report[SettingsUiThemeEditor.IMAGES] = "sky blue.png"
            ThemeEditorNatives.state(n, SettingsUiOp.THEME_EDITOR, *report)
            for (a in valueKinds) n.add(index(a.element), a)
            // Edited in place: the variable's colour.
            n.tap(500_000 + 4096 * defines + 64)
            n.choose(3, "snygg__property_value__solid_color")
            ThemeEditorNatives.pickColor(n, 4, "#00ff00")
            n.apply("--primary")
            // A font: Add source set, `src` added to the new set, and a weight to the first.
            n.tap(500_000 + 4096 * font + 4)
            n.tap(100_000_000 + 65_536 * font + 8 + 4)
            n.choose(2, "snygg__property_name__src")
            n.choose(3, "snygg__property_value__uri")
            n.choose(4, "flex:/fonts/mono%20bold.ttf")
            n.apply("src")
            n.tap(100_000_000 + 65_536 * font + 4)
            n.choose(2, "snygg__property_name__font_weight")
            n.choose(3, "snygg__property_value__font_weight")
            n.choose(4, "bold")
            n.apply("font-weight")
            val saved = n.tap(ThemeEditorNatives.SAVE_ID)
            val expected = ThemeEditorNatives.composeWrites(editSheet) {
                for (a in valueKinds) element(a.element).properties[a.name] = a.value
                (rules.getValue(SnyggAnnotationRule.Defines) as SnyggSinglePropertySetEditor).properties["--primary"] =
                    SnyggStaticColorValue(Color(0, 255, 0))
                (rules.getValue(SnyggAnnotationRule.Font("Mono")) as SnyggMultiplePropertySetsEditor).apply {
                    add { "src" to SnyggUriValue("flex:/fonts/mono%20bold.ttf") }
                    sets[0].properties["font-weight"] = SnyggFontWeightValue(FontWeight.Bold)
                }
            }
            assertEquals(expected, ThemeEditorNatives.json(saved, SettingsUiThemeEditorEffect.SAVE))
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
        val map = ReferenceThemeEditor.PropertyNameMap.map { (k, v) -> k to stringNames.getValue(v) }
        val rust = Regex("""\(\s*"([^"]+)",\s*"([^"]+)",?\s*\)""").findAll(rustArray("PROPERTIES")).map { it.groupValues[1] to it.groupValues[2] }.toList()
        assertEquals(map, rust)
    }
}
