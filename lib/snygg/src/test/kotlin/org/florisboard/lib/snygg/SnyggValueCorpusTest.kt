package org.florisboard.lib.snygg

import java.io.File
import kotlinx.serialization.json.*
import org.junit.jupiter.api.Test
import org.florisboard.lib.snygg.value.NativeSnyggValues
import kotlin.random.Random
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SnyggValueCorpusTest {
    private val valueTest = SnyggValueMatchTest()
    private fun compare(rule: SnyggRule, input: JsonElement, flags: Int) {
        val config = SnyggJsonConfiguration.of(ignoreInvalidProperties = flags and 1 != 0, ignoreInvalidValues = flags and 2 != 0)
        sameResult("property $rule $input $flags", runCatching {ReferenceSnyggPropertyCodec.decode(rule,config,input)},
            runCatching {SnyggPropertySet.fromJsonElement(rule,config,input)}, {it}, {it})
    }
    @Test fun `all real theme values and inline stylesheet fixtures match every encoder`() {
        val sheets = SnyggStylesheetMatchTest().sheets().toMutableList()
        val root = File(requireNotNull(System.getProperty("snygg.repoRoot")))
        val process = ProcessBuilder("git","ls-files","-z").directory(root).start()
        val files = process.inputStream.bufferedReader().readText().split('\u0000').filter {it.isNotEmpty()}.map {File(root,it)}
        assertEquals(0,process.waitFor())
        val triple = "\"\"\""
        for (file in files.filter { "/lib/snygg/src/test/" in it.path && it.name.endsWith("Test.kt") && !it.name.contains("MatchTest") && !it.name.contains("CorpusTest") }) {
            for (match in Regex("$triple(.*?)$triple",RegexOption.DOT_MATCHES_ALL).findAll(file.readText())) {
                var input = match.groupValues[1].trimIndent()
                if ("SCHEMA_LINE" !in input && "schemas.florisboard.org/snygg/v" !in input && "\"@defines\"" !in input) continue
                input = input.replace("\$SCHEMA_LINE", "\"\$schema\":\"${SnyggStylesheet.SCHEMA_V2}\"").replace("\${'$'}", "$")
                sheets.add(file.name to input)
            }
        }
        var values = 0
        fun visit(json: JsonElement) {
            when (json) {
                is JsonObject -> json.filterKeys {it != "\$schema"}.values.forEach(::visit)
                is JsonArray -> json.forEach(::visit)
                is JsonPrimitive -> if (json.isString) {valueTest.all(json.content);values++}
            }
        }
        for ((path,input) in sheets) {
            try { (runCatching {Json.parseToJsonElement(input)}.getOrNull() as? JsonObject)?.let(::visit) }
            catch (e: AssertionError) {throw AssertionError(path,e)}
        }
        assertEquals(12,sheets.count {it.first.startsWith("app/src/main/assets/ime/theme/")})
        assertTrue(values > 1000,"real stylesheet values: $values")
        println("Snygg value corpus: ${sheets.size} file/archive/inline documents, $values value occurrences across 28 encoders")
    }
    @Test fun `property choice missing variables required values and error precedence match`() {
        val rules = listOf(SnyggAnnotationRule.Defines, SnyggAnnotationRule.Font("Test"), SnyggElementRule("key"))
        val random = Random(0x24DEC0DE)
        val names = listOf("src","foreground","font-weight","shape","font-size","border-width","--missing","--x","--bad_name","unknown")
        val values = listOf<JsonElement>(JsonPrimitive("var(--missing)"),JsonPrimitive("inherit"),JsonPrimitive("undefined"),
            JsonPrimitive("defined"),JsonPrimitive("1dp"),JsonPrimitive("15sp"),JsonPrimitive("#abcdef"),JsonPrimitive("bold"),
            JsonPrimitive("uri(`flex:/font.ttf`)"),JsonPrimitive("cut-corner(1.5dp,0dp,0dp,0dp)"),JsonPrimitive("1px"),JsonNull,
            JsonPrimitive(3),JsonPrimitive(true),JsonObject(emptyMap()),JsonArray(emptyList()))
        for (rule in rules) for (name in names) for (value in values) for (flags in 0..3) {
            val set = JsonObject(linkedMapOf(name to value))
            compare(rule,if(rule is SnyggAnnotationRule.Font) JsonArray(listOf(set)) else set, flags)
        }
        repeat(4000) {
            val rule = rules.random(random)
            val sets = List(random.nextInt(1,4)) {JsonObject(buildMap {repeat(random.nextInt(8)) {put(names.random(random),values.random(random))}})}
            compare(rule,if(rule is SnyggAnnotationRule.Font) JsonArray(sets) else sets[0], random.nextInt(4))
        }
        // Explicitly compare native source-order choice with independent reference iteration.
        for (input in listOf("normal","none","1dp","100%","var(--missing)","inherit","transparent")) {
            val ids = NativeSnyggValues.encoders.indices.shuffled(random)
            val expected = ids.firstNotNullOfOrNull {i -> referenceValueEncoders[i].deserialize(input).getOrNull()?.production()}
            val rows = com.jtn0123.audiowhisper.rust.SnyggValues.call(8,(listOf(ids.size.toString())+ids.map {it.toString()}+input).toTypedArray()).toList()
            assertEquals(expected,rows.takeIf {it.isNotEmpty()}?.let(NativeSnyggValues::value))
        }
    }
}
