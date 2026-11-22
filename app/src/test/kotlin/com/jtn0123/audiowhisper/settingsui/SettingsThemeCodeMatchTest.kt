package com.jtn0123.audiowhisper.settingsui

import com.ibm.icu.lang.UCharacter
import com.ibm.icu.lang.UCharacterCategory
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlinx.serialization.json.*
import org.florisboard.lib.snygg.SnyggElementRule
import java.io.File
import kotlin.test.*

/**
 * The code editor's data crossing between Kotlin and the real natives: the key, stopped and described reports
 * SettingsKeyRecorder builds, read by Rust as it means them; and the key data preview Rust draws from them against
 * TextKeyDataPreviewBox for every predefined key and a spread of characters. ICU4J stands in for android.icu, which the
 * JVM only has as stubs: the same UCharacter names, and Unicode.isNonSpacingMark's category check. A key's icon is
 * ComputingEvaluator.computeImageVector's in both, as path data here since the JVM cannot draw a vector into a Path.
 */
class SettingsThemeCodeMatchTest {
    private val sheet = """{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key[code=10]":{"background":"#0000ff"}}"""
    private val rule = ThemeEditorNatives.composeEditor(sheet).rules.keys.indexOfFirst { it is SnyggElementRule }
    private val name = { code: Int -> UCharacter.getName(code) ?: UCharacter.getExtendedName(code) }
    private val mark = { code: Int -> UCharacter.getType(code).toByte() == UCharacterCategory.NON_SPACING_MARK }

    private fun KeyboardRenderer.tap(id: Int) = ThemeEditorNatives.state(this, SettingsUiOp.ACTIVATE, id.toString())
    /** The code field, row 1, given [text] as the IME's InputConnection gives it. */
    private fun KeyboardRenderer.type(text: String): JsonObject {
        tap(1001)
        return ThemeEditorNatives.state(this, SettingsUiOp.EDIT, "set", text)
    }
    private fun KeyboardRenderer.report(row: Array<String>) = ThemeEditorNatives.state(this, SettingsUiOp.THEME_CODE, *row)
    /** Native settings on Add code from the rule editor of `key[code=10]`. */
    private fun KeyboardRenderer.addCode() {
        ThemeEditorNatives.open(this)
        ThemeEditorNatives.loaded(this, sheet)
        tap(500_000 + 4096 * rule + 2)
        assertEquals("settings/theme/editor/code/add", tap(CODE_ADD_ID).getValue("route").jsonPrimitive.content)
    }
    private fun node(s: JsonObject, id: Int) =
        s.getValue("nodes").jsonArray.map { it.jsonObject }.first { it.getValue("id").jsonPrimitive.int == id }
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content

    /** ComputingEvaluator.computeLabel's resource for each label token it takes from resources, read from its source. */
    private val labelResources = Regex("""((?:KeyCode\.\w+, )*KeyCode\.\w+) -> context\(\)\?\.getString\(R\.string\.(\w+)\)""")
        .findAll(File("src/main/kotlin/dev/patrickgold/florisboard/ime/keyboard/ComputingEvaluator.kt").readText())
        .flatMap { m -> m.groupValues[1].split(", ").map { keyCode(it.removePrefix("KeyCode.")) to m.groupValues[2] } }
        .toMap()
    /** KeyCode's value named [name], read through its getter. */
    private fun keyCode(name: String) = KeyCode::class.java.getMethod("get$name").invoke(KeyCode) as Int

    /** TextKeyDataPreviewBox's label, name and kind for [code], with its evaluator's mode; a label from resources is
     * the string key, which the natives here show for want of the strings. */
    private fun composePreview(code: Int): Pair<String, String> {
        val data = TextKeyData.getCodeInfoAsTextKeyData(code) ?: TextKeyData.UNSPECIFIED
        val token = InputRules.visual(longArrayOf(data.type.ordinal.toLong(), data.code.toLong(), KeyboardMode.NUMERIC_ADVANCED.toInt().toLong()))
        val label = when (token) {
            0 -> null
            // TextKeyData.asString(isForDisplay = true).
            1 -> KeyModels.text(data.code, data.label, true, mark(data.code))
            else -> checkNotNull(labelResources[token]) { "token $token has no resource" }
        }
        val shown = if (data.code > 0) name(data.code) else data.label
        return listOfNotNull(label?.let(::wellFormed), shown).filter { it.isNotEmpty() }.joinToString(", ") to data.type.toString()
    }
    /** [s] with each lone surrogate as U+FFFD, the one difference: a Rust string cannot hold one, and the field takes the
     * surrogates' codes as characters. */
    private fun wellFormed(s: String) = buildString {
        var i = 0
        while (i < s.length) {
            val cp = s.codePointAt(i)
            appendCodePoint(if (cp in 0xD800..0xDFFF) 0xFFFD else cp)
            i += Character.charCount(cp)
        }
    }

    @Test fun thePreviewShowsWhatTextKeyDataPreviewBoxShowedForEveryKeyAndSomeCharacters() {
        val predefined = KeyModels.predefined().map { it.substringBefore(',').toInt() }
        val characters = listOf(32, 65, 97, 233, 0x301, 0x0E31, 0x05B4, 0x20AC, 0x4E2D, 0xFFFD, 0xD800, 0xE000, 65535)
        val codes = (predefined + characters + listOf(-9000, 70000)).distinct()
        assertTrue(predefined.size > 50, "${predefined.size} predefined keys")
        assertEquals(labelResources.keys, setOf(KeyCode.PHONE_PAUSE, KeyCode.PHONE_WAIT, KeyCode.IME_UI_MODE_TEXT, KeyCode.VIEW_CHARACTERS,
            KeyCode.VIEW_NUMERIC, KeyCode.VIEW_NUMERIC_ADVANCED, KeyCode.VIEW_PHONE, KeyCode.VIEW_PHONE2, KeyCode.VIEW_SYMBOLS,
            KeyCode.VIEW_SYMBOLS2, KeyCode.HALF_SPACE, KeyCode.KESHIDA))
        KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
            n.addCode()
            for (code in codes) {
                var s = n.type(code.toString())
                val asked = ThemeEditorNatives.effects(s).filter { it[0] == SettingsUiThemeCodeEffect.DESCRIBE }
                val data = TextKeyData.getCodeInfoAsTextKeyData(code) ?: TextKeyData.UNSPECIFIED
                // Kotlin is asked about the key the code names, once, and never about the unspecified key.
                assertEquals(if (data.code == 0) emptyList() else listOf(listOf(SettingsUiThemeCodeEffect.DESCRIBE, "${data.code}")), asked, "$code")
                if (asked.isNotEmpty()) s = n.report(describedCode(data.code, name, mark) { "" })
                val preview = node(s, PREVIEW_ID)
                assertEquals(composePreview(code), preview.text("label") to preview.text("state"), "$code")
            }
        }
    }

    @Test fun anIconCrossesAsPathDataAndTakesTheLabelsPlace() {
        val path = "M0,0h24v24h-24z"
        val row = describedCode(KeyCode.VIEW_SYMBOLS, name, mark) { if (it.code == KeyCode.VIEW_SYMBOLS) path else "" }
        assertEquals(path, row[SettingsUiThemeCode.ICON])
        assertEquals(listOf(SettingsUiThemeCodeReport.DESCRIBED, "${KeyCode.VIEW_SYMBOLS}", "", WireFlag.encode(false)),
            row.take(4))
        val acute = describedCode(0x301, name, mark) { "" }
        assertEquals(listOf("COMBINING ACUTE ACCENT", WireFlag.encode(true), ""), acute.drop(2))
        KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
            n.addCode()
            assertEquals("key__view_symbols, view_symbols", node(n.type("${KeyCode.VIEW_SYMBOLS}"), PREVIEW_ID).text("label"))
            assertEquals("view_symbols", node(n.report(row), PREVIEW_ID).text("label"))
        }
    }

    @Test fun aRecordedKeyAndAStopCrossAsTheRendererReadsThem() {
        KeyboardRenderer(arrayOf(ThemeEditorNatives.font)).use { n ->
            n.addCode()
            n.type("13, 9")
            val started = n.tap(RECORD_ID)
            assertTrue(listOf(SettingsUiThemeCodeEffect.RECORD) in ThemeEditorNatives.effects(started))
            assertEquals(true, node(started, RECORD_ID).getValue("checked").jsonPrimitive.boolean)
            val key = n.report(themeCodeRow(SettingsUiThemeCodeReport.KEY, "${KeyCode.DELETE}"))
            assertEquals("13, ${KeyCode.DELETE}", node(key, 1001).text("label"))
            assertEquals(false, node(key, RECORD_ID).getValue("checked").jsonPrimitive.boolean)
            // A key when not recording changes nothing.
            assertEquals("13, ${KeyCode.DELETE}", node(n.report(themeCodeRow(SettingsUiThemeCodeReport.KEY, "97")), 1001).text("label"))
            n.tap(RECORD_ID)
            val stopped = n.report(themeCodeRow(SettingsUiThemeCodeReport.STOPPED))
            assertEquals(false, node(stopped, RECORD_ID).getValue("checked").jsonPrimitive.boolean)
            n.tap(RECORD_ID)
            assertTrue(listOf(SettingsUiThemeCodeEffect.STOP) in ThemeEditorNatives.effects(n.tap(RECORD_ID)))
        }
    }

    private companion object {
        const val PREVIEW_ID = 400_300
        const val RECORD_ID = 400_301
        const val CODE_ADD_ID = 400_310
    }
}
