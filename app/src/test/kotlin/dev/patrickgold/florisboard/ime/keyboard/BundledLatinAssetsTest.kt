package dev.patrickgold.florisboard.ime.keyboard

import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Resolve the actual bundled asset graph, so deleting a component cannot leave a broken preset or modifier. */
class BundledLatinAssetsTest {
    private val root = File("src/main/assets/ime/keyboard")
    private fun extension(id: String) = Json.parseToJsonElement(File(root, "$id/extension.json").readText()).jsonObject
    private val layouts = extension("org.florisboard.layouts").getValue("layouts").jsonObject
    private val localization = extension("org.florisboard.localization")

    private fun JsonElement.id() = jsonObject.getValue("id").jsonPrimitive.content
    private fun JsonElement.text() = jsonPrimitive.content

    @Test
    fun `every preset resolves its layouts composer popup currency and punctuation`() {
        val composerIds = extension("org.florisboard.composers").getValue("composers").jsonArray.map {
            "org.florisboard.composers:" + (it.jsonObject["id"] ?: it.jsonObject.getValue("$")).text()
        }.toSet()
        val popups = localization.getValue("popupMappings").jsonArray.map { "org.florisboard.localization:" + it.id() }.toSet()
        val currencies = extension("org.florisboard.currencysets").getValue("currencySets").jsonArray
            .map { "org.florisboard.currencysets:" + it.id() }.toSet()
        val punctuation = localization.getValue("punctuationRules").jsonArray
            .map { "org.florisboard.localization:" + it.id() }.toSet()
        val locales = mutableSetOf<String>()
        for (preset in localization.getValue("subtypePresets").jsonArray.map { it.jsonObject }) {
            locales += preset.getValue("languageTag").text()
            assertTrue(preset.getValue("composer").text() in composerIds, "$preset")
            assertTrue((preset["popupMapping"]?.text() ?: "org.florisboard.localization:default") in popups, "$preset")
            assertTrue(preset.getValue("currencySet").text() in currencies, "$preset")
            preset["punctuationRule"]?.let { assertTrue(it.text() in punctuation, "$preset") }
            assertTrue("nlpProviders" !in preset, "bundled presets use the default Latin provider")
            for ((type, name) in preset.getValue("preferred").jsonObject) {
                assertTrue(layouts.getValue(type).jsonArray.any { "org.florisboard.layouts:" + it.id() == name.text() }, "$type $name")
            }
        }
        assertTrue(locales.containsAll(listOf("en-US", "de-DE", "fr-FR", "pl", "vi-VN", "tr", "sr-RS")))
    }

    @Test
    fun `every indexed layout and popup has a readable file and every modifier resolves`() {
        for ((type, components) in layouts) {
            for (element in components.jsonArray) {
                val component = element.jsonObject
                val file = component["arrangementFile"]?.text() ?: "layouts/$type/${element.id()}.json"
                Json.parseToJsonElement(File(root, "org.florisboard.layouts/$file").readText())
                component["modifier"]?.let { modifier ->
                    val modifierType = when (type) {
                        "characters" -> "charactersMod"
                        "symbols" -> "symbolsMod"
                        "symbols2" -> "symbols2Mod"
                        else -> error("Unexpected modifier on $type")
                    }
                    assertTrue(layouts.getValue(modifierType).jsonArray.any {
                        "org.florisboard.layouts:" + it.id() == modifier.text()
                    }, "$type ${element.id()}: $modifier")
                }
            }
        }
        for (popup in localization.getValue("popupMappings").jsonArray) {
            Json.parseToJsonElement(File(root, "org.florisboard.localization/popupMappings/${popup.id()}.json").readText())
        }
        for (type in listOf("numeric", "numericAdvanced", "numericRow", "symbols", "symbols2", "phone", "phone2")) {
            assertTrue(layouts.getValue(type).jsonArray.isNotEmpty(), type)
        }
        assertEquals(setOf("appender", "telex"), extension("org.florisboard.composers").getValue("composers").jsonArray.map {
            (it.jsonObject["id"] ?: it.jsonObject.getValue("$")).text()
        }.toSet())
    }
}
