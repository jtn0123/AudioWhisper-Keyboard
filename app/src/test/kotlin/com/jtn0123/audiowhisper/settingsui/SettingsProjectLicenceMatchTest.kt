package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.RenderProtocol
import com.jtn0123.audiowhisper.rust.SettingsUiAssetOutcome
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import com.jtn0123.audiowhisper.rust.SettingsUiSelection
import com.jtn0123.audiowhisper.rust.SettingsUiTextEffect
import com.jtn0123.audiowhisper.rust.SettingsUiTextNode
import kotlinx.serialization.json.*
import java.io.File
import kotlin.test.*

/**
 * ProjectLicenseScreen through the real natives, as RustSettingsView and SettingsReadOnlyText
 * drive it: the asset plan and the ASSET op that answers it, the text node's TalkBack facts, the
 * SELECTION op's Select all, Copy and traversal, the toolbar's place in the reply, the error
 * text, and the horizontal scroll kept across recreation.
 */
class SettingsProjectLicenceMatchTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private val route = "settings/about/project-license"
    private val path = "license/project_license.txt"
    private val licence = File("src/main/assets/$path").readText()
    private fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) = Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun node(s: JsonObject, id: Int) = nodes(s).firstOrNull { it.int("id") == id }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun toolbar(s: JsonObject) = s.getValue("textToolbar").takeUnless { it == JsonNull }?.jsonArray?.map { it.jsonPrimitive.float }
    private fun JsonObject.int(key: String) = getValue(key).jsonPrimitive.int
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    /** The text node's text facts; null when the reply drops them, so a test asserts rather than throws. */
    private fun facts(s: JsonObject) = node(s, SettingsUiTextNode.TEXT)?.get("text") as? JsonObject
    private fun traversed(s: JsonObject) = (facts(s)?.get("traversed") as? JsonArray)?.map { it.jsonPrimitive.int }

    private val strings: Map<String, String> by lazy {
        val pattern = Regex("""<string name="([^"]+)"[^>]*>(.*?)</string>""", RegexOption.DOT_MATCHES_ALL)
        pattern.findAll(File("src/main/res/values/strings.xml").readText()).associate { it.groupValues[1] to it.groupValues[2] }
    }
    /**
     * RustSettingsView.configure's packets for a 720 by 2400 px phone at density 3, its sp sizes
     * included, where the text scrolls both ways (no monospace font is checked in, so the lines are
     * Roboto's, narrower than DroidSansMono's).
     */
    private fun prepare(n: KeyboardRenderer) {
        n.settings(SettingsUiOp.RESOURCES, n.settings(SettingsUiOp.STRING_KEYS)[2].split("\n").filter { it in strings }.flatMap { listOf(it, strings.getValue(it)) }.toTypedArray())
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("720", "2400", "3", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { (3 * it).toFloat().toString() } + listOf("0", "4294967295", "true", "true", "4278190080",
            "4278198272", "4278190080")).toTypedArray())
    }
    /** About, then the licence: Rust asks for the asset once and draws nothing until it comes. */
    private fun open(n: KeyboardRenderer): JsonObject {
        prepare(n)
        state(n, SettingsUiOp.NAVIGATE, "settings/about")
        val asked = state(n, SettingsUiOp.NAVIGATE, route)
        assertEquals(route, asked.text("route"))
        assertEquals(listOf(listOf(SettingsUiTextEffect.ASSET, path)), effects(asked))
        assertNull(node(asked, SettingsUiTextNode.TEXT))
        return state(n, SettingsUiOp.ASSET, path, SettingsUiAssetOutcome.OK, licence)
    }

    @Test fun theAssetOpAnswersThePlanAndTheTextNodeCarriesTalkBacksFacts() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            val s = open(n)
            assertEquals(emptyList(), effects(s))
            val text = assertNotNull(node(s, SettingsUiTextNode.TEXT))
            assertEquals(licence, text.text("label"))
            assertEquals("selectable", text.text("role"))
            // The label is the whole licence; the description is left empty so the reply carries it once.
            assertEquals("", text.text("description"))
            assertEquals(-1, text.int("anchor"))
            assertEquals(31, facts(s)?.get("granularities")?.jsonPrimitive?.int)
            assertNull(traversed(s))
            assertNull(toolbar(s))
            // Both scrollers say they can move forward and not back.
            val horizontal = assertNotNull(node(s, SettingsUiTextNode.SCROLL)).getValue("actions").jsonArray.map { it.jsonPrimitive.int }
            assertEquals(listOf(RenderProtocol.ACCESS_SCROLL_FORWARD), horizontal)
            assertTrue(s.getValue("maxScroll").jsonPrimitive.float > 0f)
        }
    }

    @Test fun selectAllCopyAndTraversalGoThroughTheSelectionOp() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            open(n)
            val all = state(n, SettingsUiOp.SELECTION, SettingsUiSelection.SELECT_ALL)
            val text = node(all, SettingsUiTextNode.TEXT)!!
            assertEquals(0 to licence.length, text.int("anchor") to text.int("focus"))
            // Everything is selected, so the toolbar offers no Select all.
            assertEquals(0f, assertNotNull(toolbar(all))[4])
            val copied = state(n, SettingsUiOp.SELECTION, SettingsUiSelection.COPY)
            assertEquals(listOf(listOf(SettingsUiTextEffect.CLIP, licence)), effects(copied))
            assertNull(toolbar(copied))
            assertEquals(-1, node(copied, SettingsUiTextNode.TEXT)!!.int("anchor"))
            // A word forward from the start: "Apache", after the first line's 33 spaces.
            val word = state(n, SettingsUiOp.SELECTION, SettingsUiSelection.TRAVERSE, "2", "1", "0")
            assertEquals(listOf(33, 39, 2, 256), traversed(word))
            assertEquals(39 to 39, node(word, SettingsUiTextNode.TEXT)!!.let { it.int("anchor") to it.int("focus") })
            val set = state(n, SettingsUiOp.SELECTION, SettingsUiSelection.SET, "33", "47")
            assertEquals(listOf(listOf(SettingsUiTextEffect.CLIP, "Apache License")), effects(state(n, SettingsUiOp.SELECTION, SettingsUiSelection.COPY)))
            assertEquals(1f, assertNotNull(toolbar(set))[4])
            assertFailsWith<IllegalArgumentException> { n.settings(SettingsUiOp.SELECTION, arrayOf("paste")) }
        }
    }

    @Test fun aFailedReadShowsTheErrorStringAndAStrayAnswerIsRefused() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            open(n)
            val failed = state(n, SettingsUiOp.ASSET, path, SettingsUiAssetOutcome.FAILED, "boom")
            val error = strings.getValue("about__project_license__error_license_text_failed").replace("{error_message}", "boom")
            assertEquals(error, node(failed, SettingsUiTextNode.TEXT)!!.text("label"))
            assertFailsWith<IllegalArgumentException> { n.settings(SettingsUiOp.ASSET, arrayOf("other.txt", SettingsUiAssetOutcome.OK, "x")) }
        }
    }

    @Test fun theHorizontalScrollSurvivesRecreation() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            open(n)
            state(n, SettingsUiOp.ACCESSIBILITY_SCROLL, SettingsUiTextNode.SCROLL.toString(), RenderProtocol.ACCESS_SCROLL_FORWARD.toString())
            val saved = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
            assertEquals("10", saved[0])
            assertEquals(21, saved.size)
            assertTrue(saved[18].toFloat() > 0f)
            KeyboardRenderer(arrayOf(font)).use { recreated ->
                prepare(recreated)
                val restored = state(recreated, SettingsUiOp.RESTORE, *saved)
                assertEquals(route, restored.text("route"))
                assertEquals(listOf(listOf(SettingsUiTextEffect.ASSET, path)), effects(restored))
                state(recreated, SettingsUiOp.ASSET, path, SettingsUiAssetOutcome.OK, licence)
                assertEquals(saved[18], recreated.settings(SettingsUiOp.SAVE).drop(3)[18])
            }
        }
    }
}
