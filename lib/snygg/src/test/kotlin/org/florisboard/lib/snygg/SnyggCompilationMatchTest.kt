package org.florisboard.lib.snygg

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.colorspace.ColorSpaces
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.jtn0123.audiowhisper.rust.KeyModels
import java.io.File
import kotlinx.serialization.json.*
import org.florisboard.lib.snygg.value.*
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SnyggCompilationMatchTest {
    private val root = File(requireNotNull(System.getProperty("snygg.repoRoot")))
    private val initial = SnyggSinglePropertySet(mapOf(Snygg.FontSize to SnyggSpSizeValue(14.sp)))
    private fun ReferenceSnyggTheme.ask(name: String, attrs: SnyggQueryAttributes, selector: SnyggSelector,
        parent: SnyggSinglePropertySet = initial, multiplier: Float = 1f) =
        query(name, attrs, selector, parent, yellowLightScheme, yellowDarkScheme, multiplier)
    private fun SnyggTheme.ask(name: String, attrs: SnyggQueryAttributes, selector: SnyggSelector,
        parent: SnyggSinglePropertySet = initial, multiplier: Float = 1f) =
        query(name, attrs, selector, parent, yellowLightScheme, yellowDarkScheme, multiplier)
    private fun compare(a: ReferenceSnyggTheme, b: SnyggTheme, name: String, attrs: SnyggQueryAttributes,
        selector: SnyggSelector, parent: SnyggSinglePropertySet = initial, multiplier: Float = 1f): SnyggSinglePropertySet {
        val expected = a.ask(name, attrs, selector, parent, multiplier)
        val actual = b.ask(name, attrs, selector, parent, multiplier)
        assertEquals(expected.properties.toList(), actual.properties.toList(), "$name $attrs $selector x$multiplier")
        return expected
    }
    private fun elements(): List<String> {
        val src = File(root, "app/src/main/kotlin/dev/patrickgold/florisboard/ime/theme/FlorisImeUi.kt").readText()
        return Regex("elementName = \"([^\"]+)\"").findAll(src).map { it.groupValues[1] }.toList().also {
            assertTrue(it.size > 70)
        }
    }
    private fun enumValues(path: String) = Regex("^    ([A-Z][A-Z0-9_]+)\\(-?\\d+\\)", RegexOption.MULTILINE)
        .findAll(File(root, path).readText()).map { it.groupValues[1].lowercase() }.toList()
    private val modes by lazy { enumValues("app/src/main/kotlin/dev/patrickgold/florisboard/ime/keyboard/KeyboardMode.kt") }
    private val shifts by lazy { enumValues("app/src/main/kotlin/dev/patrickgold/florisboard/ime/input/InputShiftState.kt") }
    private fun domains(): Map<String, List<String>> {
        val codes = mutableSetOf<Int>()
        codes.addAll(KeyModels.codes().toList())
        File(root, "app/src/main/assets").walkTopDown().filter { it.extension == "json" }.forEach { file ->
            Regex("\"code\"\\s*:\\s*(-?\\d+)").findAll(file.readText()).forEach { codes.add(it.groupValues[1].toInt()) }
        }
        return mapOf("mode" to modes, "shiftstate" to shifts, "code" to codes.sorted().map { it.toString() },
            "auto-commit" to listOf("0", "1"), "type" to listOf("text", "image", "video", "audio", "other"),
            "state" to listOf("active", "inactive"), "action" to listOf("yes", "no"))
    }
    /** Exact predicate partition: all values within a class match every rule identically.
     * Cross every class (including absent/unknown) rather than sampling state combinations. */
    private fun queries(rules: List<SnyggElementRule>, domains: Map<String, List<String>>): List<SnyggQueryAttributes> {
        val keys = rules.flatMap { it.attributes.keys }.distinct()
        var combinations = listOf<Map<String, Any>>(emptyMap())
        for (key in keys) {
            val lists = rules.mapNotNull { it.attributes[key] }.distinct()
            val values = (domains[key].orEmpty() + lists.flatten() + "__unmatched__").distinct()
            val classes = values.groupBy { v -> lists.map { v in it } }.values.map { it.first() }
            combinations = combinations.flatMap { attrs -> listOf(attrs) + classes.map { attrs + (key to it) } }
        }
        return combinations.distinct()
    }
    @Test fun `every bundled stylesheet every app element and state predicate combination matches`() {
        val domains = domains()
        var count = 0
        for ((path, json) in SnyggStylesheetMatchTest().sheets()) {
            val refSheet = ReferenceSnyggStylesheet.fromJson(json).getOrThrow()
            val sheet = SnyggStylesheet.fromJson(json).getOrThrow()
            assertEquals(refSheet.rules.values.toList(), sheet.rules.values.toList())
            val a = ReferenceSnyggTheme.compileFrom(SnyggStylesheet(refSheet.schema, refSheet.rules.mapKeys { it.key.production() }))
            val b = SnyggTheme.compileFrom(sheet)
            assertEquals(a.style.toList(), b.style.toList(), path)
            val names = (elements() + sheet.rules.keys.filterIsInstance<SnyggElementRule>().map { it.elementName } + "absent").distinct()
            // Every root mode/shift/selector combination supplies the actual inherited context.
            val parents = modes.flatMap { mode -> shifts.flatMap { shift -> SnyggSelector.entries.map { selector ->
                val attrs = mapOf("mode" to mode, "shiftstate" to shift)
                val r = compare(a, b, "root", attrs, SnyggSelector.NONE)
                attrs to compare(a, b, "window", attrs, selector, r)
            } } }.distinct()
            for (name in names) {
                val rules = sheet.rules.keys.filterIsInstance<SnyggElementRule>().filter { it.elementName == name }
                for (attrs in queries(rules, domains)) for (selector in SnyggSelector.entries) {
                    for ((_, parent) in parents.distinctBy { it.second }) {
                        compare(a, b, name, attrs, selector, parent); count++
                    }
                }
                // Exercise every concrete app attribute value as well as all predicate products.
                for ((key, values) in domains) for (v in values) {
                    compare(a, b, name, mapOf(key to v), SnyggSelector.NONE); count++
                }
            }
        }
        println("Snygg resolution corpus: $count queries; ${elements().size} app elements; ${modes.size} modes; ${shifts.size} shift states; all five selectors and complete rule-predicate products")
    }
    private fun values(random: Random): Map<String, SnyggValue> = buildMap {
        for (key in listOf("foreground", "background", "font-size", "letter-spacing", "shape", "custom")) {
            if (random.nextBoolean()) put(key, when (random.nextInt(10)) {
                0 -> SnyggUndefinedValue
                1 -> SnyggInheritValue
                2 -> SnyggDefinedVarValue("--${random.nextInt(4)}")
                3 -> SnyggSpSizeValue(random.nextInt(-20, 80).sp)
                4 -> SnyggDynamicLightColorValue("primary")
                5 -> SnyggDynamicDarkColorValue("secondary")
                6 -> SnyggSpSizeValue(TextUnit.Unspecified)
                7 -> SnyggSpSizeValue(2.em)
                8 -> SnyggStaticColorValue(Color(.2f, .4f, .7f, .5f, ColorSpaces.DisplayP3))
                else -> NativeSnyggValues.encoders[random.nextInt(28)].defaultValue()
            })
        }
    }
    @Test fun `seeded programmatic stylesheets queries parent chains and float bits match`() {
        val random = Random(250510)
        repeat(1000) {
            val rules = linkedMapOf<SnyggRule, SnyggPropertySet>(SnyggAnnotationRule.Defines to
                SnyggSinglePropertySet((0..3).associate { "--$it" to SnyggDefinedVarValue("--${(it + 1) % 4}") }))
            repeat(random.nextInt(1, 18)) {
                val attrs = SnyggAttributes(mapOf("code" to List(random.nextInt(4)) { random.nextInt(4).toString() }))
                rules[SnyggElementRule("e${random.nextInt(4)}", attrs, SnyggSelector.entries.random(random))] = SnyggSinglePropertySet(values(random))
            }
            val sheet = SnyggStylesheet(SnyggStylesheet.SCHEMA_V2, rules)
            val a = ReferenceSnyggTheme.compileFrom(sheet)
            val b = SnyggTheme.compileFrom(sheet)
            assertEquals(a.style.toList(), b.style.toList())
            var parent = SnyggSinglePropertySet(values(random))
            repeat(30) {
                parent = compare(a, b, "e${random.nextInt(5)}", mapOf("code" to random.nextInt(5)),
                    SnyggSelector.entries.random(random), parent,
                    listOf(0f, -0f, .5f, 1f, 1.75f, Float.POSITIVE_INFINITY).random(random))
            }
        }
    }
    @Test fun `prototype light and dark handoff matches reference and exports Rust oracle`() {
        val result = buildJsonObject {
            for ((path, input) in SnyggStylesheetMatchTest().sheets().filter { it.first.endsWith("/floris_day.json") || it.first.endsWith("/floris_night.json") }) {
                val theme = ReferenceSnyggTheme.compileFrom(SnyggStylesheet.fromJson(input).getOrThrow())
                val attrs = mapOf("mode" to "characters", "shiftstate" to "unshifted", "code" to 97)
                val parent = theme.ask("window", attrs, SnyggSelector.NONE, theme.ask("root", attrs, SnyggSelector.NONE))
                put(path, buildJsonObject {
                    for ((label, name, selector) in listOf(Triple("key", "key", SnyggSelector.NONE),
                        Triple("pressed-key", "key", SnyggSelector.PRESSED), Triple("candidate", "smartbar-candidate-word", SnyggSelector.NONE),
                        Triple("popup", "key-popup-box", SnyggSelector.NONE), Triple("focus", "key-popup-element", SnyggSelector.FOCUS),
                        Triple("window", "window", SnyggSelector.NONE), Triple("hint", "key-hint", SnyggSelector.NONE))) {
                        val style = theme.ask(name, attrs, selector, parent)
                        put(label, buildJsonObject { style.properties.forEach { (k, v) -> put(k, v.encoder().serialize(v).getOrThrow()) } })
                    }
                })
            }
        }
        val output = File(root, "build/stage25/renderer-expected.json")
        checkNotNull(output.parentFile).mkdirs(); output.writeText(result.toString())
        val fixture = File(root, "lib/rust/crates/snygg/tests/renderer-expected.json")
        if (fixture.exists()) assertEquals(Json.parseToJsonElement(fixture.readText()), result)
        assertEquals(2, result.size)
    }
}
