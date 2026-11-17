package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SemanticRules
import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import kotlinx.serialization.json.*
import java.io.File
import java.util.Locale
import kotlin.test.*

class RendererAccessibilityMatchTest {
    @Test fun everyRetainedLayoutAndPopupLabelMatchesComposeAndLatinImeFallback() {
        val files = File("src/main/assets/ime/keyboard").walkTopDown()
            .filter { it.isFile && it.extension == "json" }.sortedBy { it.path }.toList()
        assertTrue(files.count { "/layouts/" in it.path } > 50)
        val fixtures = files.flatMap { file -> leaves(Json.parseToJsonElement(file.readText())).toList() }
        assertTrue(fixtures.size > 1000)
        var checks = 0
        for (shift in listOf("unshifted", "shifted_manual", "shifted_automatic", "caps_lock")) {
            val data = fixtures.map { n ->
                val code = n["code"]?.jsonPrimitive?.intOrNull ?: -902
                var label = n["label"]?.jsonPrimitive?.contentOrNull ?: ""
                if (n["$"]?.jsonPrimitive?.contentOrNull == "auto_text_key") {
                    label = if (shift == "unshifted") label.lowercase(Locale.ROOT) else label.uppercase(Locale.ROOT)
                }
                code to label
            }
            val actual = SemanticRules.labels(data.map { it.first }.toIntArray(), data.map { it.second }.toTypedArray(), shift)
            data.forEachIndexed { i, (code, label) ->
                assertEquals(ReferenceAccessibilityLabels.key(code, label, shift), actual[i], "fixture $i state $shift")
                checks++
            }
        }
        println("Accessibility labels: ${files.size} retained JSON files, ${fixtures.size} key/popup leaves, $checks shift cases")
    }
    @Test fun everyComposeQuickActionAndLitStateAndInsertTextMatches() {
        val data = ReferenceAccessibilityLabels.allActions().keys.map { it to "" } +
            listOf(0 to "whole text, not its first glyph", 0 to "", -902 to "é🙂")
        for (lit in 0..2) {
            val actual = SemanticRules.labels(data.map { it.first }.toIntArray(), data.map { it.second }.toTypedArray(), "unshifted", lit)
            data.forEachIndexed { i, (code, text) -> assertEquals(ReferenceAccessibilityLabels.action(code,text,lit),actual[i]) }
        }
    }
    @Test fun actualSceneSnapshotAndActionPacketsCrossJniWithoutAndroid() {
        KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            r.semanticConfig(arrayOf("access_shift_on"),arrayOf("Localized shift"),emptyArray(),intArrayOf(0,1),intArrayOf(),intArrayOf(),intArrayOf(0,0))
            fun update(shift: String) = r.update(floatArrayOf(200f,200f,40f,50f,60f),arrayOf("characters",shift,"modifier","character"),
                intArrayOf(),floatArrayOf(0f,0f,50f,60f,0f,0f,50f,60f,1f,-11f,50f,0f,100f,60f,50f,0f,100f,60f,1f,101f),
                arrayOf("","","","e","",""),emptyArray())
            update("unshifted")
            val first = Json.parseToJsonElement(r.semanticSnapshot()).jsonObject.getValue("nodes").jsonArray
            assertEquals("Shift",first[0].jsonObject.getValue("description").jsonPrimitive.content)
            update("shifted_manual")
            val shifted = Json.parseToJsonElement(r.semanticSnapshot()).jsonObject.getValue("nodes").jsonArray
            assertEquals("Localized shift",shifted[0].jsonObject.getValue("description").jsonPrimitive.content)
            assertEquals(first.map { it.jsonObject["id"] },shifted.map { it.jsonObject["id"] })
            assertContentEquals(intArrayOf(KEY_DOWN,1,-1,KEY_UP,1,-1),r.semanticAction(2,ACCESS_CLICK))
            assertContentEquals(intArrayOf(KEY_DOWN,1,-1,OPEN_POPUP,1,-1),r.semanticAction(2,ACCESS_LONG_CLICK))
            r.popup(1,arrayOf("é","è"),intArrayOf(233,232));r.popupGrid(0,false)
            val all = Json.parseToJsonElement(r.semanticSnapshot()).jsonObject.getValue("nodes").jsonArray.map { it.jsonObject }
            val popup = all.filter { it.getValue("role").jsonPrimitive.content == "popup item" }
            assertEquals(2,popup.size)
            // The choices read straight after the held key, each naming the node read before it,
            // and stay drawn on top for touch.
            val ids = all.map { it.getValue("id").jsonPrimitive.int }
            assertEquals(ids.indexOf(2) + 1, ids.indexOf(popup[0].getValue("id").jsonPrimitive.int))
            assertEquals(listOf<Int?>(null) + ids.dropLast(1), all.map { it.getValue("after").jsonPrimitive.intOrNull })
            fun layer(n: JsonObject) = n.getValue("layer").jsonPrimitive.int
            assertTrue(popup.minOf(::layer) > (all - popup.toSet()).maxOf(::layer))
            assertContentEquals(intArrayOf(POPUP_SELECT,0,-1),r.semanticAction(popup[0].getValue("id").jsonPrimitive.int,ACCESS_CLICK))
            assertFalse(Json.parseToJsonElement(r.semanticSnapshot()).jsonObject.getValue("popup").jsonPrimitive.boolean)
        }
    }
    private fun leaves(value: JsonElement): Sequence<JsonObject> = sequence {
        when (value) {
            is JsonObject -> {
                if ("code" in value || "codePoints" in value) yield(value)
                value.values.forEach { yieldAll(leaves(it)) }
            }
            is JsonArray -> value.forEach { yieldAll(leaves(it)) }
            else -> Unit
        }
    }
}
