package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import kotlinx.serialization.json.*
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/**
 * ThirdPartyLicensesScreen as Rust draws it, against [ReferenceLicenceScreens]: every row's text,
 * role, enabled state and badges in AboutLibraries' order, every license dialog's text and
 * button, and the `load` plan that asks Kotlin for the resource.
 */
class SettingsLicencesMatchTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private val route = "settings/about/third-party-licenses"
    private val rowId = 10000
    private fun state(n: KeyboardRenderer, op: Int = 10, vararg input: String) = Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun JsonObject.int(key: String) = getValue(key).jsonPrimitive.int
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content

    private val strings: Map<String, String> by lazy {
        val pattern = Regex("""<string name="([^"]+)"[^>]*>(.*?)</string>""", RegexOption.DOT_MATCHES_ALL)
        pattern.findAll(File("src/main/res/values/strings.xml").readText()).associate { it.groupValues[1] to it.groupValues[2] }
    }
    /** RustSettingsView.configure's packets for a window tall enough for every row. */
    private fun prepare(n: KeyboardRenderer) {
        n.settings(1, n.settings(18)[2].split("\n").filter { it in strings }.flatMap { listOf(it, strings.getValue(it)) }.toTypedArray())
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(3, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true", "4278190080",
            "4278198272", "4278190080")).toTypedArray())
    }

    private val pieces = listOf("&amp;", "&lt;b&gt;", "&copy;", "&#169;", "&#x41;", "&bogus;", "&nbsp;", "  ", "\n", "\r\n", "\t",
        "<p>", "</p>", "<br />", "<b>", "</i>", "<li>", "<ul>", "</ul>", "<div>", "<h2>", "</h2>", "<img src=x>",
        "<script>x<y</script>", "<a href=\"u\">", "é", "日本", "😀", "<", ">", "Copyright", "Permission is hereby granted")
    private fun content(r: Random) = (0 until r.nextInt(1, 30)).joinToString("") { pieces[r.nextInt(pieces.size)] }
    private val names = listOf("activity", "Activity", "ätsch", "Zebra", "zeta", "𝒜lpha", "Ａpp", "core", "Core KTX", "okio")
    private val licenceKeys = listOf("Apache-2.0", "MIT", "icu4c", "BSD-3-Clause", "GPL-2.0", "ISC", "Zlib", "CC0-1.0")

    /** A generated resource: names that tie once lowercased, blank versions, every author source. */
    private fun resource(seed: Int, count: Int): String {
        val r = Random(seed)
        val licences = buildJsonObject {
            for (key in licenceKeys) putJsonObject(key) {
                put("name", "$key License")
                when (r.nextInt(4)) { 0 -> {}; 1 -> put("content", ""); else -> put("content", content(r)) }
            }
        }
        val libraries = buildJsonArray {
            for (i in 0 until count) addJsonObject {
                put("uniqueId", "group:lib-$i")
                if (r.nextInt(8) != 0) put("name", names[r.nextInt(names.size)] + if (r.nextBoolean()) " $i" else "")
                when (r.nextInt(4)) { 0 -> {}; 1 -> put("artifactVersion", " "); else -> put("artifactVersion", "1.$i.0") }
                when (r.nextInt(4)) {
                    0 -> {}
                    1 -> putJsonArray("developers") {}
                    else -> putJsonArray("developers") {
                        repeat(r.nextInt(1, 3)) { addJsonObject { if (r.nextInt(5) != 0) put("name", "Dev ${r.nextInt(9)}") } }
                    }
                }
                if (r.nextBoolean()) putJsonObject("organization") { if (r.nextInt(4) != 0) put("name", "Org $i") }
                putJsonArray("licenses") { repeat(r.nextInt(0, 5)) { add(licenceKeys[r.nextInt(licenceKeys.size)]) }; if (r.nextInt(6) == 0) add("unknown") }
            }
        }
        return buildJsonObject { put("libraries", libraries); put("licenses", licences) }.toString()
    }

    private fun open(n: KeyboardRenderer, json: String): JsonObject {
        prepare(n)
        val s = state(n, 4, route)
        assertEquals(route, s.text("route"))
        assertEquals(listOf(listOf("load", "libraries")), effects(s))
        assertTrue(nodes(s).none { it.int("id") >= rowId })
        val loaded = state(n, 21, "libraries", json)
        assertEquals(emptyList(), effects(loaded))
        return loaded
    }

    @Test fun everyRowsTextRoleEnabledStateAndBadgesMatchAboutLibraries() {
        for (seed in 0 until 12) KeyboardRenderer(arrayOf(font)).use { n ->
            val json = resource(seed, 60)
            val expected = ReferenceLicenceScreens.rows(json)
            val rows = nodes(open(n, json)).filter { it.int("id") >= rowId }
            assertEquals(expected.indices.map { rowId + it }, rows.map { it.int("id") }, "seed $seed")
            for ((node, row) in rows.zip(expected)) {
                assertEquals(row.name, node.text("label"), "seed $seed")
                assertEquals(listOfNotNull(row.version, row.author).plus(row.badges).joinToString(", "), node.text("state"), "seed $seed ${row.name}")
                assertEquals("card", node.text("role"))
                assertTrue(node.getValue("enabled").jsonPrimitive.boolean)
                assertEquals(listOf(1), node.getValue("actions").jsonArray.map { it.jsonPrimitive.int })
            }
        }
    }

    @Test fun everyLicenseDialogShowsTheTextFromHtmlGivesAndAnOkButton() {
        for (seed in 0 until 6) KeyboardRenderer(arrayOf(font)).use { n ->
            val json = resource(100 + seed, 40)
            val expected = ReferenceLicenceScreens.rows(json)
            open(n, json)
            for ((j, row) in expected.withIndex()) {
                val dialog = state(n, 5, (rowId + j).toString())
                assertTrue(dialog.getValue("popup").jsonPrimitive.boolean)
                val shown = nodes(dialog)
                val body = shown.filter { it.int("id") == 9 }
                assertEquals(listOfNotNull(row.dialog), body.map { it.text("label") }, "seed $seed row $j")
                body.forEach { assertEquals("text", it.text("role")) }
                val ok = shown.single { it.int("id") == 7 }
                assertEquals(strings.getValue("action__ok"), ok.text("label"))
                assertEquals("button", ok.text("role"))
                assertTrue(shown.none { it.int("id") >= rowId }, "the dialog covers the list")
                val closed = state(n, 5, "7")
                assertFalse(closed.getValue("popup").jsonPrimitive.boolean)
                assertEquals(route, closed.text("route"))
            }
        }
    }

    @Test fun theRealResourcesShapeParsesTheSameWay() {
        // The plugin's own output shape: one licence per library, scm, website, funding, empty arrays.
        val json = """{"libraries":[{"uniqueId":"androidx.activity:activity","artifactVersion":"1.13.0","name":"Activity",
            "description":"d","website":"https://x","developers":[{"name":"The Android Open Source Project"}],
            "organization":{"name":"The Android Open Source Project"},"scm":{"url":"https://y"},"licenses":["Apache-2.0"],
            "funding":[]},{"uniqueId":"com.ibm.icu:icu4j","artifactVersion":"78.3","name":"ICU4J","licenses":["icu4c"],
            "funding":[{"platform":"GitHub","url":"https://z"}]}],"licenses":{"Apache-2.0":{"name":"Apache License 2.0",
            "url":"https://a","content":"Apache License\nVersion 2.0, January 2004\n\n   TERMS"},"icu4c":{"name":"ICU4C License",
            "url":"https://b","content":"COPYRIGHT AND PERMISSION NOTICE (ICU 58 and later)\n\nCopyright © 2016"}}}"""
        KeyboardRenderer(arrayOf(font)).use { n ->
            val expected = ReferenceLicenceScreens.rows(json)
            val rows = nodes(open(n, json)).filter { it.int("id") >= rowId }
            assertEquals(expected.map { it.name }, rows.map { it.text("label") })
            assertEquals("Apache License\nVersion 2.0, January 2004\n\nTERMS", expected[0].dialog)
            for (j in expected.indices) {
                assertEquals(expected[j].dialog, nodes(state(n, 5, (rowId + j).toString())).single { it.int("id") == 9 }.text("label"))
                state(n, 5, "7")
            }
        }
    }

    /** AndroidParser's catch-all logs through android.util.Log, which the JVM cannot run; its result is empty data. */
    @Test fun aMalformedResourceShowsAnEmptyList() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            val shown = open(n, """{"libraries":[{"name":"no id"}],"licenses":{}}""")
            assertTrue(nodes(shown).none { it.int("id") >= rowId })
        }
    }
}
