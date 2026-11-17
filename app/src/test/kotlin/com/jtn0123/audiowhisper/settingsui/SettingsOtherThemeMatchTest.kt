package com.jtn0123.audiowhisper.settingsui

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.app.ReferenceColorSerializer
import dev.patrickgold.florisboard.app.ReferenceSettingsModel
import dev.patrickgold.jetpref.datastore.model.LocalTime
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import dev.patrickgold.jetpref.material.ui.parseOrNull
import kotlinx.serialization.json.*
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/**
 * Other and Theme as Rust draws them, against [ReferenceOtherThemeScreens], with the real strings;
 * the colour dialog against Jetpref's own parser, formats and picker state and Compose's `Color`;
 * the time rows against Jetpref's LocalTime.
 */
class SettingsOtherThemeMatchTest {
    private val ref = ReferenceOtherThemeScreens
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    /** The system accent the device reports: no preset, so Default is a colour of its own. */
    private val accent = Color(0xff6750a4)
    private val presets = ref.presets
    /** Jetpref's LocalTime serializer, as the theme's time preferences hold it. */
    private val times = ReferenceSettingsModel().theme.sunriseTime.serializer
    private fun serialized(t: LocalTime) = times.serialize(t).orEmpty()
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
        n.settings(20, facts(emptyMap(), false))
    }
    /** SettingsFacts' packet: the system accent, the 24-hour setting, the theme index's labels. */
    private fun facts(labels: Map<String, String>, is24: Boolean) =
        arrayOf("system_accent", accent.toArgb().toUInt().toString(), "is24hour", is24.toString()) +
            SettingsFacts.themeLabelPairs(labels)
    private fun combinations(inputs: Map<String, List<String>>): List<Map<String, String>> =
        inputs.entries.fold(listOf(emptyMap())) { acc, (key, values) -> acc.flatMap { m -> values.map { m + (key to it) } } }

    private fun clicks(node: JsonObject) = 1 in node.getValue("actions").jsonArray.map { it.jsonPrimitive.int }

    @Test fun everyRowTitleSummaryRoleAndEnableStateMatchesFrozenCompose() {
        var checked = 0
        for (screen in ref.screens) KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, defaults())
            for (combo in combinations(screen.inputs)) for (labels in screen.labelSets) for (is24 in screen.clocks) {
                val f = ReferenceOtherThemeScreens.Facts(defaults() + combo, labels, is24)
                n.settings(2, f.p.flatMap { listOf(it.key, it.value) }.toTypedArray())
                n.settings(20, facts(labels, is24))
                val reply = state(n, 4, screen.route)
                val label = "${screen.route} $combo ${labels.size} labels, 24h $is24"
                assertEquals(screen.route, reply.text("route"), label)
                val all = nodes(reply)
                val byId = all.associateBy { it.int("id") }
                val rows = all.filter { it.int("id") in 1000 until 2000 }
                assertEquals(screen.rows.indices.map { 1000 + it }, rows.map { it.int("id") }, label)
                for (node in rows) {
                    val i = node.int("id") - 1000
                    val row = screen.rows[i]
                    assertEquals(s(row.title), node.text("label"), "title row $i $label")
                    assertEquals(ref.summary(row, f, ::s), node.text("state"), "summary row $i $label")
                    assertEquals(row.enabled(f), node.bool("enabled"), "enabled row $i $label")
                    val role = when (row.kind) { "heading" -> "heading"; "addons" -> "text"; else -> "button" }
                    assertEquals(role, node.text("role"), "role row $i $label")
                    assertEquals(row.kind !in setOf("heading", "addons") && row.enabled(f), clicks(node), "click row $i $label")
                    if (row.kind == "addons") {
                        assertEquals(ref.addonTitle(row, ::s), byId.getValue(5000 + i).text("label"), "box title $label")
                        val button = byId.getValue(6000 + i)
                        assertEquals(ref.addonButton(row, ::s), button.text("label"), "box button $label")
                        assertEquals("button", button.text("role"), label)
                        assertTrue(clicks(button), label)
                    }
                }
                assertEquals(screen.preview, all.any { it.int("id") == 2 }, "preview field $label")
                checked++
            }
        }
        // Other's 15 combinations and Theme's 4 x 4 x 3 x 4 x 2 stored values x 2 label sets x 2 clocks.
        assertEquals(15 + 1536, checked)
    }

    @Test fun routesGoWhereComposeWent() {
        for (screen in ref.screens) KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, defaults())
            state(n, 4, screen.route)
            val native = n.settings(19)[2].split("\n")
            for ((i, row) in screen.rows.withIndex()) if (row.route.isNotEmpty()) {
                // The addon box goes from its button; every other row from itself.
                val reply = tap(n, (if (row.kind == "addons") 6000 else 1000) + i)
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

    @Test fun timeSummariesMatchJetprefAtEveryMinuteOnBothClocks() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, defaults())
            state(n, 4, "settings/theme")
            val rows = listOf("theme__sunrise_time", "theme__sunset_time").map { key -> ref.theme.rows.indexOfFirst { it.key == key } }
            for (is24 in listOf(false, true)) {
                n.settings(20, facts(emptyMap(), is24))
                for (minute in 0 until 24 * 60) {
                    val time = LocalTime(minute / 60, minute % 60)
                    val stored = serialized(time)
                    n.settings(2, arrayOf("theme__sunrise_time", stored, "theme__sunset_time", stored))
                    val nodes = byId(state(n))
                    for (row in rows) assertEquals(time.stringRepresentation(is24), nodes.getValue(1000 + row).text("state"), "$stored $is24")
                }
            }
        }
    }

    /** Opens the sunrise dialog: on the dial, or switched to the keyboard the first time it is asked for. */
    private fun openTime(n: KeyboardRenderer, row: Int, keyboard: Boolean) {
        assertTrue(tap(n, 1000 + row).bool("popup"))
        val dial = byId(state(n)).containsKey(7160)
        if (keyboard == dial) tap(n, 7154)
    }

    /** Clears the field holding the keyboard and types `digits` into it one by one. */
    private fun type(n: KeyboardRenderer, digits: String) {
        state(n, 9, "set", "")
        for (c in digits) state(n, 9, "commit", c.toString(), "1")
    }

    @Test fun theTimeDialogStoresWhatJetprefStoresOnTheDialAndTyped() {
        val random = Random(20261119)
        KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, defaults() + ("theme__mode" to "FOLLOW_TIME"))
            state(n, 4, "settings/theme")
            val row = ref.theme.rows.indexOfFirst { it.key == "theme__sunrise_time" }
            for (is24 in listOf(false, true)) {
                n.settings(20, facts(emptyMap(), is24))
                // On the dial: every hour by its number, then a minute on the face, then OK.
                for (hour in 0 until 24) {
                    val minute = 5 * random.nextInt(12)
                    openTime(n, row, keyboard = false)
                    if (!is24) tap(n, if (hour >= 12) 7153 else 7152)
                    tap(n, 7160 + if (is24) hour else hour % 12)
                    tap(n, 7160 + minute / 5)
                    val stored = serialized(LocalTime(hour, minute))
                    assertEquals(listOf(listOf("write", "theme__sunrise_time", stored)), effects(tap(n, 7)), "dial $stored $is24")
                    assertEquals(LocalTime(hour, minute).stringRepresentation(is24), byId(state(n)).getValue(1000 + row).text("state"))
                }
                // Typed a digit at a time over a cleared field: every minute, each with a random hour.
                for (minute in 0 until 60) {
                    val hour = random.nextInt(24)
                    openTime(n, row, keyboard = true)
                    if (!is24) tap(n, if (hour >= 12) 7153 else 7152)
                    tap(n, 7001)
                    type(n, "%02d".format(if (is24) hour else (hour + 11) % 12 + 1))
                    // TimeInput moves on by itself after a 12-hour clock's 2 to 12; otherwise Next.
                    if (7002 !in byId(state(n))) state(n, 7, "0", "66", "0")
                    assertTrue(7002 in byId(state(n)), "the minute field after $hour")
                    type(n, "%02d".format(minute))
                    val stored = serialized(LocalTime(hour, minute))
                    assertEquals(listOf("write", "theme__sunrise_time", stored), effects(tap(n, 7)).last(), "typed $stored $is24")
                }
                tap(n, 1000 + row)
                tap(n, 7154)
                tap(n, 6)
            }
        }
    }
}
