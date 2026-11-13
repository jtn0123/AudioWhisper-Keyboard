package org.florisboard.lib.snygg

import java.io.File
import java.util.zip.ZipFile
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.jupiter.api.Test
import kotlin.random.Random
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SnyggStylesheetMatchTest {
    private val root = File(requireNotNull(System.getProperty("snygg.repoRoot")))
    private val schemaLine = "\"\$schema\":\"${SnyggStylesheet.SCHEMA_V2}\""

    private fun config(flags: Int, pretty: Boolean = false) = SnyggJsonConfiguration.of(
        ignoreMissingSchema = flags and 1 != 0, ignoreInvalidSchema = flags and 2 != 0,
        ignoreUnsupportedSchema = flags and 4 != 0, ignoreInvalidRules = flags and 8 != 0,
        ignoreInvalidProperties = flags and 16 != 0, ignoreInvalidValues = flags and 32 != 0,
        prettyPrint = pretty, prettyPrintIndent = if (flags and 1 == 0) "  " else "\t",
    )

    private fun tracked(): List<File> {
        val process = ProcessBuilder("git", "ls-files", "-z").directory(root).start()
        val files = process.inputStream.bufferedReader().readText().split('\u0000').filter { it.isNotEmpty() }.map { File(root, it) }
        assertEquals(0, process.waitFor())
        return files
    }

    internal fun sheets(): List<Pair<String, String>> = buildList {
        for (file in tracked()) {
            if (file.extension == "json" && file.name != "stylesheet.schema.json") {
                val content = file.readText()
                if ("/stylesheets/" in file.path || "schemas.florisboard.org/snygg/v" in content ||
                    ("\"\$schema\"" in content && ("/test/" in file.path || "/tests/" in file.path))) {
                    add(file.relativeTo(root).path to content)
                }
            } else if (file.extension in setOf("flex", "zip", "floris")) {
                ZipFile(file).use { zip ->
                    for (entry in zip.entries()) if (!entry.isDirectory && entry.name.endsWith(".json")) {
                        val content = zip.getInputStream(entry).bufferedReader().readText()
                        if ("stylesheets/" in entry.name || "schemas.florisboard.org/snygg/v" in content) {
                            add("${file.relativeTo(root)}!${entry.name}" to content)
                        }
                    }
                }
            }
        }
    }

    @Test
    fun `every repo stylesheet and inline test document matches with exact round trips`() {
        val sheets = sheets()
        assertEquals(12, sheets.count { it.first.startsWith("app/src/main/assets/ime/theme/") })
        assertTrue(sheets.isNotEmpty())
        for ((path, input) in sheets) {
            for (flags in 0..63) {
                try { checkSheet(input, config(flags, flags and 1 != 0)) }
                catch (e: AssertionError) { throw AssertionError(path, e) }
            }
            val keys = (Json.parseToJsonElement(input) as JsonObject).keys.filter { it != "\$schema" }
            keys.forEach(::checkRule)
        }
        var literals = 0
        val triple = "\"\"\""
        for (file in tracked().filter { "/lib/snygg/src/test/" in it.path && it.name.endsWith("Test.kt") && !it.name.contains("MatchTest") }) {
            for (match in Regex("$triple(.*?)$triple", RegexOption.DOT_MATCHES_ALL).findAll(file.readText())) {
                var input = match.groupValues[1].trimIndent()
                if ("SCHEMA_LINE" !in input && "schemas.florisboard.org/snygg/v" !in input && "\"@defines\"" !in input) continue
                input = input.replace("\$SCHEMA_LINE", schemaLine).replace("\${'$'}", "$")
                for (flags in listOf(0, 63)) checkSheet(input, config(flags))
                literals++
            }
        }
        assertTrue(literals > 20, "inline test fixtures discovered: $literals")
        println("Snygg corpus: ${sheets.size} file/archive stylesheets, $literals inline test documents")
    }

    @Test
    fun `schema versions invalid JSON duplicate rules and error precedence match`() {
        val schemas = listOf<String?>(null, "null", "1", "true", "{}", "[]", "\"\"", "\"bad\"",
            "\"${SnyggStylesheet.SCHEMA_V2}\"", "\"https://schemas.florisboard.org/snygg/v1/stylesheet\"",
            "\"https://schemas.florisboard.org/snygg/v02/stylesheet\"",
            "\"https://schemasXflorisboardYorg/snygg/v2/stylesheet\"",
            "\"https://schemas😀florisboard😀org/snygg/v2/stylesheet\"",
            "\"https://schemas.florisboard.org/snygg/v2/stylesheet\\n\"")
        val entries = listOf("", "\"version\":2", "\"version\":{}", "\"unknown[unknown=`yes`]:focus\":{}", "\"bad_name\":{}", "\"key:bad\":{}",
            "\"key[x=2147483648..2]\":{}", "\"key\":[],\"bad:\":{}", "\"key\":{\"unknown\":\"wrong\"},\"bad:\":{}",
            "\"key\":{},\"key\":{\"foreground\":\"transparent\"}",
            "\"key[b=1][a=2]\":{},\"z\":{},\"key[a=2][b=1]\":{\"foreground\":\"transparent\"}",
            "\"@font `A`\":[{}],\"bad:\":{}", "\"bad:\":{},\"@font `A`\":[{}]",
            "\"key\":{\"foreground\":[]},\"bad:\":{}")
        for (schema in schemas) for (entry in entries) for (flags in 0..63) {
            val fields = listOfNotNull(schema?.let { "\"\$schema\":$it" }, entry.takeIf { it.isNotEmpty() })
            checkSheet(fields.joinToString(",", "{", "}"), config(flags))
        }
        val malformed = listOf("", "{", "}", "null", "[]", "1", "true", "\"text\"", "{,}", "{\"x\":}",
            "{$schemaLine,}", "{$schemaLine} trailing", "{unquoted:2}", "{\"x\":NaN}", "{\"x\":\"\\uZZZZ\"}")
        for (input in malformed) for (flags in 0..63) checkSheet(input, config(flags))
    }

    @Test
    fun `seeded and mutated documents match`() {
        val random = Random(0x23C0DE)
        val names = listOf("key", "unknown[unknown=2,1]", "key:pressed", "bad_name", "key:bad", "@defines", "@font `A B`",
            "key[x=1..3]", "key[x=3,2,1]", "key[x=`\uD800`]", "key[a=1][b=2]", "key[b=2][a=1]")
        val values = listOf("{}", "[]", "null", "1", "\"text\"", "{\"foreground\":\"transparent\"}",
            "{\"--var\":\"1dp\"}", "{\"unknown\":\"invalid\"}", "[{\"src\":\"uri(`flex:/a`)\"}]", "{\"foreground\":[]}")
        repeat(4000) {
            val fields = mutableListOf<String>()
            if (random.nextBoolean()) fields.add(if (random.nextBoolean()) schemaLine else "\"\$schema\":\"wrong\"")
            repeat(random.nextInt(9)) { fields.add(JsonPrimitive(names.random(random)).toString() + ":" + values.random(random)) }
            checkSheet(fields.joinToString(",", "{", "}"), config(random.nextInt(64), random.nextBoolean()))
        }
        for ((_, input) in sheets()) repeat(100) { iteration ->
            val changed = when (iteration % 5) {
                0 -> input.take(random.nextInt(input.length))
                1 -> input.replace("\"\$schema\"", "\"schema\"")
                2 -> input.replace("/v2/", "/v${random.nextInt(100)}/")
                3 -> input.replace(":pressed", ":bad-selector")
                else -> {
                    val obj = Json.parseToJsonElement(input) as JsonObject
                    val key = obj.keys.filter { it != "\$schema" }.random(random)
                    input.trimEnd().dropLast(1) + "," + JsonPrimitive(key) + ":{} }"
                }
            }
            checkSheet(changed, config(random.nextInt(64), random.nextBoolean()))
        }
    }
}
