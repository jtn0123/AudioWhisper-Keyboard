package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.app.ReferenceColorSerializer
import dev.patrickgold.florisboard.app.ReferenceSettingsModel
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import dev.patrickgold.jetpref.material.ui.parseOrNull
import kotlinx.serialization.json.*
import org.florisboard.lib.color.ColorMappings
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/**
 * Other and Theme as Rust draws them, against [ReferenceOtherThemeScreens], with the real strings;
 * the colour dialog against Jetpref's own parser, formats and picker state and Compose's `Color`.
 */
class SettingsOtherThemeMatchTest {
    private val ref = ReferenceOtherThemeScreens
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    /** The system accent the device reports: no preset, so Default is a colour of its own. */
    private val accent = Color(0xff6750a4)
    /** FlorisBoard's ColorMappings.colors, the Presets page. */
    private val presets = ColorMappings.colors.toList()
    private fun <T : Any> raw(p: PreferenceData<T>) = p.serializer.serialize(p.default).orEmpty()
    private fun defaults() = ReferenceSettingsModel().declaredPreferenceEntries.values.associate { it.key to raw(it) }
    private fun state(n: KeyboardRenderer, op: Int = 10, vararg input: String) = Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun byId(s: JsonObject) = nodes(s).associateBy { it.int("id") }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun JsonObject.int(key: String) = getValue(key).jsonPrimitive.int
    private fun JsonObject.bool(key: String) = getValue(key).jsonPrimitive.boolean
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    private fun tap(n: KeyboardRenderer, id: Int) = state(n, 5, id.toString())

    /** The app's English strings, read from the resource files the APK is built from. */
    private val strings: Map<String, String> by lazy {
        val pattern = Regex("""<string name="([^"]+)"[^>]*>(.*?)</string>""", RegexOption.DOT_MATCHES_ALL)
        listOf("strings.xml", "strings_dont_translate.xml", "strings_settings_renderer.xml").flatMap { name ->
            pattern.findAll(File("src/main/res/values/$name").readText()).map { it.groupValues[1] to it.groupValues[2] }
        }.toMap()
    }
    private fun s(key: String) = strings[key] ?: key
    /** RustSettingsView.configure's packets: strings, preferences, a window tall enough for every row, facts. */
    private fun prepare(n: KeyboardRenderer, p: Map<String, String>) {
        n.settings(1, n.settings(18)[2].split("\n").filter { it in strings }.flatMap { listOf(it, strings.getValue(it)) }.toTypedArray())
        n.settings(2, p.flatMap { listOf(it.key, it.value) }.toTypedArray())
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(3, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true")).toTypedArray())
        n.settings(20, arrayOf("system_accent", accent.toArgb().toUInt().toString()))
    }
    private fun combinations(inputs: Map<String, List<String>>): List<Map<String, String>> =
        inputs.entries.fold(listOf(emptyMap())) { acc, (key, values) -> acc.flatMap { m -> values.map { m + (key to it) } } }

    @Test fun everyRowTitleSummaryRoleAndEnableStateMatchesFrozenCompose() {
        for (screen in ref.screens) for (combo in combinations(screen.inputs)) KeyboardRenderer(arrayOf(font)).use { n ->
            val f = ReferenceOtherThemeScreens.Facts(defaults() + combo)
            prepare(n, f.p)
            val reply = state(n, 4, screen.route)
            val label = "${screen.route} $combo"
            assertEquals(screen.route, reply.text("route"), label)
            val all = nodes(reply)
            val rows = all.filter { it.int("id") in 1000 until 2000 }
            assertEquals(screen.rows.indices.map { 1000 + it }, rows.map { it.int("id") }, label)
            for (node in rows) {
                val i = node.int("id") - 1000
                val row = screen.rows[i]
                assertEquals(s(row.title), node.text("label"), "title row $i $label")
                assertEquals(ref.summary(row, f, ::s), node.text("state"), "summary row $i $label")
                assertEquals(row.enabled(f), node.bool("enabled"), "enabled row $i $label")
                assertEquals(if (row.kind == "heading") "heading" else "button", node.text("role"), "role row $i $label")
                val click = 1 in node.getValue("actions").jsonArray.map { it.jsonPrimitive.int }
                assertEquals(row.kind != "heading" && row.enabled(f), click, "click row $i $label")
            }
            assertEquals(screen.preview, all.any { it.int("id") == 2 }, "preview field $label")
        }
    }

    @Test fun routesGoWhereComposeWent() {
        for (screen in ref.screens) KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, defaults())
            state(n, 4, screen.route)
            val native = n.settings(19)[2].split("\n")
            for ((i, row) in screen.rows.withIndex()) if (row.kind == "route") {
                val reply = tap(n, 1000 + i)
                if (row.route in native) {
                    assertEquals(row.route, reply.text("route"))
                    state(n, 6, "3", "0")
                } else assertEquals(listOf(listOf("compose", row.route, native.joinToString("|"))), effects(reply))
                assertEquals(screen.route, state(n).text("route"))
            }
        }
    }

    /** The colour dialog as it stands, against Jetpref's: page, ticks, the field. */
    private fun check(n: KeyboardRenderer, d: ReferenceOtherThemeScreens.ColorDialog, editing: Boolean, label: String) {
        val reply = state(n)
        assertTrue(reply.bool("popup"), label)
        val dialog = byId(reply)
        assertEquals(d.neutralLabel, dialog.getValue(5).text("label"), label)
        if (!d.advanced) {
            for ((j, c) in presets.withIndex()) {
                val swatch = dialog.getValue(7100 + j)
                assertEquals(d.preset == j, swatch.bool("selected"), "preset $j $label")
                // Compose gives a swatch no label; Rust names it by its colour.
                assertEquals(ColorRepresentation.HEX.formatColor(c, false), swatch.text("label"), "preset $j $label")
            }
            assertEquals(d.defaultTicked, dialog.getValue(7120).bool("selected"), "default $label")
            assertNull(dialog[7000], label)
            return
        }
        assertTrue(presets.indices.none { 7100 + it in dialog }, label)
        val field = dialog.getValue(7000)
        assertEquals(if (editing) "edit" else "text", field.text("role"), label)
        if (!editing) assertEquals(d.shown(), field.text("state"), label)
        assertEquals(if (editing) listOf(7141, 7142) else listOf(7140), dialog.keys.filter { it in 7140..7142 }.sorted(), label)
    }

    /** Texts for the colour field: every form Jetpref reads, and near misses of each. */
    private fun texts(random: Random): List<String> {
        val numbers = listOf("0", "1", "9", "10", "50", "99", "100", "101", "199", "254", "255", "256", "359", "360", "361",
            "00", "01", "-1", "1.5", " 7", "7 ")
        val out = mutableListOf("#6275ac", "#6275AC", " #000000\n", "　#ffffff", "#fff", "#6275ac80", "6275ac", "#gg0000",
            "", "#", "rgb(1, 2, 3", "rgba(1, 2, 3)", "RGB(1,2,3)", "rgb (1, 2, 3)", "rgb(1, 2, 3, 1)", "hsl(0, 0, 0)",
            "hsva(120, 50, 50)", "hsv( 0 ,\t0 , 0 )", "rgb(1, 2, 3)", "\u0085#000000", "\u001f#000000")
        repeat(300) { out += "#%06x".format(random.nextInt(0x1000000)).let { if (random.nextBoolean()) it.uppercase() else it } }
        repeat(300) { out += "rgb(${numbers.random(random)}, ${random.nextInt(256)}, ${numbers.random(random)})" }
        repeat(300) { out += "hsv(${random.nextInt(362)}, ${random.nextInt(102)}, ${numbers.random(random)})" }
        // Every hue and a spread of saturations and values: Compose's HSV maths at each step.
        for (h in 0..360 step 3) out += "hsv($h, ${random.nextInt(101)}, ${random.nextInt(101)})"
        return out
    }

    @Test fun typedColoursParseShowAndStoreAsJetprefDoes() {
        val random = Random(20261117)
        KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, defaults())
            state(n, 4, "settings/other")
            val row = ref.other.rows.indexOfFirst { it.kind == "color" }
            assertTrue(tap(n, 1000 + row).bool("popup"))
            val d = ReferenceOtherThemeScreens.ColorDialog(null, accent, presets)
            tap(n, 5); d.neutral()
            check(n, d, editing = false, "custom")
            for (text in texts(random)) {
                tap(n, 7140)
                assertEquals(d.shown(), byId(state(n)).getValue(7000).text("label"), "the editor starts on the colour")
                tap(n, 7000)
                state(n, 9, "set", text)
                val applied = effects(tap(n, 7141))
                val expected = Color.parseOrNull(text, false)
                assertEquals(expected != null, d.apply(text))
                if (expected == null) {
                    assertEquals(emptyList(), applied, text)
                    check(n, d, editing = true, text)
                    assertEquals(listOf(listOf("hideEditor")), effects(tap(n, 7142)), text)
                } else assertEquals(listOf(listOf("hideEditor")), applied, text)
                check(n, d, editing = false, text)
            }
            assertEquals(listOf(listOf("write", "other__accent_color", d.stored { ReferenceColorSerializer.serialize(it) })),
                effects(tap(n, 7)))
        }
    }

    @Test fun pickingPagingCancellingAndConfirmingMatchJetpref() {
        val random = Random(20261118)
        val p = defaults().toMutableMap()
        val storedAccent = ReferenceColorSerializer.serialize(accent)
        val corpus = texts(random)
        KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, p)
            state(n, 4, "settings/other")
            val row = ref.other.rows.indexOfFirst { it.kind == "color" }
            repeat(120) { round ->
                // Sometimes the stored colour changes underneath: a preset, the accent written out, one of the user's own.
                if (random.nextInt(4) == 0) {
                    p["other__accent_color"] = (ref.accents + storedAccent).random(random)
                    n.settings(2, p.flatMap { listOf(it.key, it.value) }.toTypedArray())
                }
                val stored = p.getValue("other__accent_color").let { if (it == ref.UNSPECIFIED) null else ReferenceColorSerializer.deserialize(it) }
                val d = ReferenceOtherThemeScreens.ColorDialog(stored, accent, presets)
                assertTrue(tap(n, 1000 + row).bool("popup"))
                var editing = false
                var keyboard = false
                check(n, d, editing, "open $round")
                while (true) {
                    val step = random.nextInt(10)
                    val label = "round $round step $step"
                    val hide = if (keyboard) listOf(listOf("hideEditor")) else emptyList()
                    when {
                        step == 0 -> {
                            assertEquals(hide, effects(tap(n, 6)), label)
                            break
                        }
                        step == 1 -> {
                            val written = d.stored { ReferenceColorSerializer.serialize(it) }
                            assertEquals(hide + listOf(listOf("write", "other__accent_color", written)), effects(tap(n, 7)), label)
                            p["other__accent_color"] = written
                            n.settings(2, p.flatMap { listOf(it.key, it.value) }.toTypedArray())
                            break
                        }
                        step == 2 -> {
                            assertEquals(hide, effects(tap(n, 5)), label)
                            d.neutral(); keyboard = false
                        }
                        !d.advanced && step < 7 -> {
                            val j = random.nextInt(presets.size)
                            tap(n, 7100 + j); d.pick(j)
                        }
                        !d.advanced -> { tap(n, 7120); d.pickDefault() }
                        !editing -> { tap(n, 7140); editing = true }
                        else -> {
                            tap(n, 7000); keyboard = true
                            val text = corpus.random(random)
                            state(n, 9, "set", text)
                            if (step < 8 && d.apply(text)) {
                                assertEquals(listOf(listOf("hideEditor")), effects(tap(n, 7141)), label)
                                editing = false; keyboard = false
                            } else if (step < 8) tap(n, 7141)
                            else {
                                assertEquals(listOf(listOf("hideEditor")), effects(tap(n, 7142)), label)
                                editing = false; keyboard = false
                            }
                        }
                    }
                    check(n, d, editing, label)
                }
                assertFalse(state(n).bool("popup"))
            }
        }
    }
}
