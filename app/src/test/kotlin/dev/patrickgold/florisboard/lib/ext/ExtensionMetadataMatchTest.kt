package dev.patrickgold.florisboard.lib.ext

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.ExtensionPolicy
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentEditor
import dev.patrickgold.florisboard.lib.ValidationRule
import dev.patrickgold.florisboard.lib.validate
import java.io.File
import kotlinx.serialization.json.*
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ExtensionMetadataMatchTest {
    private fun outcome(body: () -> Any?): Any? = try { body() } catch (e: Exception) { e.javaClass.name to e.message }
    private fun encoded(extension: Extension) = ExtensionJsonConfig.encodeToString(Extension.serializer(), extension)
    private fun compare(text: String, tape: GoldenTape) {
        val reference = outcome { encoded(ReferenceExtensionPolicy.decode(text)) }
        tape.answer("decoded", reference)
        assertEquals(reference, outcome { encoded(ExtensionCodec.decode(text)) })
    }

    /** [result]'s text with each string resource by name: the ids change whenever a string is added. */
    private fun named(result: Any) = MESSAGE_ID.replace(result.toString()) { m ->
        "MessageId=" + (STRING_NAMES[m.groupValues[1].toInt()] ?: m.value)
    }

    @Test fun everyBundledExtensionAndGeneratedManifestMatches() {
        val bundled = File("src/main/assets/ime").walkTopDown().filter { it.name == "extension.json" }.sortedBy { it.path }.toList()
        assertEquals(7, bundled.size)
        // The reference's answers also go on a golden tape (docs/phase6-kotlin-references.md). The
        // bundled extensions are on it, in path order, so a change to one means writing it again.
        val tape = GoldenTape.ofReference("extension-manifests")
        for (file in bundled) {
            compare(file.readText(), tape)
            val meta = ExtensionCodec.decode(file.readText()).meta
            assertTrue(meta.validate())
        }
        val base = ExtensionJsonConfig.parseToJsonElement(bundled.first { "/theme/" in it.path }.readText()).jsonObject
        val variants = mutableListOf<JsonObject>(base)
        for (field in listOf("id", "version", "title", "maintainers", "license")) {
            val original = base.getValue("meta").jsonObject
            for (value in listOf<JsonElement?>(null, JsonNull, JsonPrimitive(""), JsonPrimitive(" "), JsonPrimitive(12), JsonArray(emptyList()), JsonObject(emptyMap()))) {
                val meta = original.toMutableMap().apply { if (value == null) remove(field) else put(field, value) }
                variants += JsonObject(base + ("meta" to JsonObject(meta)))
            }
        }
        for (id in listOf("a", "a.9", "A", "a..b", "../bad", "a._b", "é", "a\n", "a.1_b", "\uD800")) {
            variants += JsonObject(base + ("meta" to JsonObject(base.getValue("meta").jsonObject + ("id" to JsonPrimitive(id)))))
        }
        for (version in listOf("bad version", "1..2", "-1", "1.2.3-alpha+9", "999999999999999999999", "١.٢.٣")) {
            variants += JsonObject(base + ("meta" to JsonObject(base.getValue("meta").jsonObject + ("version" to JsonPrimitive(version)))))
        }
        for (schema in listOf(JsonNull, JsonPrimitive("unknown/v999"), JsonPrimitive(3), JsonObject(emptyMap()))) {
            variants += JsonObject(base + ("\$schema" to schema))
        }
        for (kind in listOf(JsonPrimitive("unknown"), JsonPrimitive("ime.extension.keyboard"), JsonPrimitive("ime.extension.languagepack"), JsonNull, JsonPrimitive(42), JsonObject(emptyMap()))) {
            variants += JsonObject(base + ("$" to kind))
        }
        val theme = base.getValue("themes").jsonArray.first()
        variants += JsonObject(base + ("themes" to JsonArray(listOf(theme, theme))))
        val unknownComponent = JsonObject(theme.jsonObject + ("$" to JsonPrimitive("unknown-component")))
        variants += JsonObject(base + ("themes" to JsonArray(listOf(unknownComponent))))
        variants += JsonObject(base - "$")
        for (composer in listOf("unknown", "appender", "with-rules")) {
            val item = buildJsonObject {
                put("$", composer); put("id", "fixture"); put("label", "Fixture")
                putJsonObject("rules") { put("a", "b") }
            }
            variants += JsonObject(base + mapOf("$" to JsonPrimitive("ime.extension.keyboard"), "composers" to JsonArray(listOf(item))))
        }
        for (document in variants) compare(document.toString(), tape)
        for (document in variants) compare(JsonObject(document.entries.reversed().associate { it.toPair() }).toString(), tape)
        for (first in listOf("ime.extension.theme", "unknown")) for (last in listOf("ime.extension.keyboard", "ime.extension.theme")) {
            compare(base.toString().replaceFirst("\"ime.extension.theme\"", "\"$first\"").dropLast(1) + ",\"$\":\"$last\"}", tape)
        }
        // The platform syntax boundary remains lenient, and retains exact error classes/messages.
        for (text in listOf("{}", "[]", "null", "5", "{", "{\"$\":", base.toString().dropLast(1), base.toString().replace("\"id\":", "id:"))) compare(text, tape)
        tape.finish()
        // No semantic-version/schema restriction existed in import; identifiers are an editor rule.
        assertTrue(ExtensionCodec.decode(variants[0].toString()).components().isNotEmpty())
        val duplicate = ExtensionCodec.decode(JsonObject(base + ("themes" to JsonArray(listOf(theme, theme)))).toString())
        assertEquals(2, duplicate.components().size)
    }

    private val rules: List<Pair<ValidationRule<String>, ValidationRule<String>>> get() = listOf(
        ReferenceExtensionValidation.MetaId to ExtensionValidation.MetaId,
        ReferenceExtensionValidation.MetaVersion to ExtensionValidation.MetaVersion,
        ReferenceExtensionValidation.MetaTitle to ExtensionValidation.MetaTitle,
        ReferenceExtensionValidation.MetaMaintainers to ExtensionValidation.MetaMaintainers,
        ReferenceExtensionValidation.MetaLicense to ExtensionValidation.MetaLicense,
        ReferenceExtensionValidation.ComponentId to ExtensionValidation.ComponentId,
        ReferenceExtensionValidation.ComponentLabel to ExtensionValidation.ComponentLabel,
        ReferenceExtensionValidation.ComponentAuthors to ExtensionValidation.ComponentAuthors,
        ReferenceExtensionValidation.ThemeComponentStylesheetPath to ExtensionValidation.ThemeComponentStylesheetPath,
    )
    private fun validations(tape: GoldenTape, value: String, selected: List<Pair<ValidationRule<String>, ValidationRule<String>>> = rules) {
        for ((reference, actual) in selected) {
            // Resource IDs, status/error class and substituted regex text determine Compose's exact message.
            tape.record(named(validate(reference, value)))
            assertEquals(validate(reference, value).toString(), validate(actual, value).toString())
        }
    }

    @Test fun allUtf16AndSeededMetadataErrorsMatchComposeRules() {
        val tape = GoldenTape.ofReference("extension-validations")
        for (unit in Char.MIN_VALUE..Char.MAX_VALUE) validations(tape, unit.toString(), listOf(rules[0], rules[5], rules[6]))
        val random = Random(20261113)
        val pieces = listOf("a", "Z", "0", "_", ".", "..", "é", "١", "\uD800", "😀", " ", "\n", "\t", "\u00a0", "\u001c", "/", "'", ":", "<", "*", "x".repeat(31))
        repeat(20_000) { validations(tape, List(random.nextInt(8)) { pieces.random(random) }.joinToString("")) }
        for (value in listOf("a.9", "a.0_b", "x".repeat(30), "x".repeat(31), "\u00a0" + "x".repeat(29) + "\u00a0")) validations(tape, value)
        val fields = listOf("local.test", "version?", "Title", "A <x>", "MIT")
        assertTrue(ExtensionPolicy.metadataValid(fields[0], fields[1], fields[2], fields[3], fields[4]))
        repeat(5) { missing ->
            val values = fields.mapIndexed { i, v -> if (i == missing) " " else v }
            val reference = listOf(rules[0].first, rules[1].first, rules[2].first, rules[3].first, rules[4].first)
                .zip(values).all { (rule, value) -> validate(rule, value).isValid() }
            tape.answer("valid without $missing", reference)
            assertEquals(reference, ExtensionPolicy.metadataValid(values[0], values[1], values[2], values[3], values[4]))
        }
        tape.finish()
    }

    @Test fun componentNamesEditorPlansAndTypeDecisionsMatch() {
        val random = Random(20261114)
        val parts = listOf("", "a", ":", "a:b", "a:b:c", "\uD800", " ", "\n", "é", "local.theme")
        val tape = GoldenTape.ofReference("extension-components")
        repeat(5_000) {
            val value = List(random.nextInt(5)) { parts.random(random) }.joinToString("")
            tape.answer("name", outcome { ReferenceExtensionPolicy.componentName(value) })
            assertEquals(outcome { ReferenceExtensionPolicy.componentName(value) }, outcome { ExtensionComponentName.from(value) })
            val label = parts.random(random)
            val authors = List(random.nextInt(4)) { parts.random(random) }
            val path = parts.random(random)
            tape.answer("theme", outcome { ReferenceExtensionPolicy.themeComponent(value, label, authors, path) })
            tape.answer("stylesheet", ReferenceExtensionPolicy.stylesheet(value, path))
            assertEquals(outcome { ReferenceExtensionPolicy.themeComponent(value, label, authors, path) },
                outcome { ThemeExtensionComponentEditor(id = value, label = label, authors = authors, stylesheetPath = path).build() })
            assertEquals(ReferenceExtensionPolicy.stylesheet(value, path), ExtensionPolicy.stylesheet(value, path))
        }
        val serials = listOf("ime.extension.keyboard", "ime.extension.theme", "ime.extension.languagepack", "unknown", "")
        for (serial in serials) {
            tape.answer("group $serial", outcome { ReferenceExtensionPolicy.group(serial) })
            assertEquals(outcome { ReferenceExtensionPolicy.group(serial) }, outcome { ExtensionPolicy.group(serial) })
        }
        tape.finish()
        assertEquals("appender", ExtensionPolicy.composerType("unknown"))
        assertEquals("appender", ExtensionPolicy.composerType(null))
        assertEquals("with-rules", ExtensionPolicy.composerType("with-rules"))
    }

    private companion object {
        val MESSAGE_ID = Regex("MessageId=(-?\\d+)")
        val STRING_NAMES: Map<Int, String> by lazy { R.string::class.java.fields.associate { it.getInt(null) to it.name } }
    }
}
