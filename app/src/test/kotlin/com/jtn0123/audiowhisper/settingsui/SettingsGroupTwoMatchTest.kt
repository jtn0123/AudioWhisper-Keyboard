package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.dictation.ModelState
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import com.jtn0123.audiowhisper.rust.SettingsUiWordEffect
import dev.patrickgold.florisboard.app.ReferenceSettingsModel
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import kotlinx.serialization.json.*
import java.io.File
import kotlin.random.Random
import kotlin.test.*

/** The group 2 screens Rust draws, against [ReferenceGroupTwoScreens], with the real strings. */
class SettingsGroupTwoMatchTest {
    private val ref = ReferenceGroupTwoScreens
    /** A real font: the card's hyperlink is as wide as its text, and no width is no node. */
    private val font = java.io.File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    /** Rows with no click of their own: a card's link is a separate node. */
    private val passive = listOf("heading", "info", "header", "card", "status", "note")
    /** Rows nothing on them acts on. */
    private val inert = passive - "card"
    private fun <T : Any> raw(p: PreferenceData<T>) = p.serializer.serialize(p.default).orEmpty()
    private fun defaults() = ReferenceSettingsModel().declaredPreferenceEntries.values.associate { it.key to raw(it) }
    private fun state(n: KeyboardRenderer, op: SettingsUiOp = SettingsUiOp.SAVE, vararg input: String) = Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun JsonObject.int(key: String) = getValue(key).jsonPrimitive.int
    private fun JsonObject.bool(key: String) = getValue(key).jsonPrimitive.boolean
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    private fun route(n: KeyboardRenderer) = state(n).text("route")

    /** The app's English strings, read from the resource files the APK is built from. */
    private val strings: Map<String, String> by lazy {
        val pattern = Regex("""<string name="([^"]+)"[^>]*>(.*?)</string>""", RegexOption.DOT_MATCHES_ALL)
        listOf("strings.xml", "strings_dont_translate.xml").flatMap { name ->
            pattern.findAll(File("src/main/res/values/$name").readText()).map { it.groupValues[1] to it.groupValues[2] }
        }.toMap()
    }
    private fun s(key: String) = strings[key] ?: key
    private fun facts(f: ReferenceGroupTwoScreens.Facts) = SettingsFacts.factPairs(f.versionName, f.versionCode, f.packageName,
        f.keyboardAttached, f.spellEnabled, f.spellId) { f.spellLabel } + SettingsFacts.modelPairs(f.speech, f.voice, f.checked)
    /** RustSettingsView.configure's packets: strings, preferences, a window tall enough for every row, facts. */
    private fun prepare(n: KeyboardRenderer, f: ReferenceGroupTwoScreens.Facts) {
        n.settings(SettingsUiOp.RESOURCES, n.settings(SettingsUiOp.STRING_KEYS)[2].split("\n").filter { it in strings }.flatMap { listOf(it, strings.getValue(it)) }.toTypedArray())
        n.settings(SettingsUiOp.PREFERENCES, f.p.flatMap { listOf(it.key, it.value) }.toTypedArray())
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "true", "true")).toTypedArray())
        n.settings(SettingsUiOp.FACTS, facts(f))
    }
    private fun combinations(inputs: Map<String, List<String>>): List<Map<String, String>> =
        inputs.entries.fold(listOf(emptyMap())) { acc, (key, values) -> acc.flatMap { m -> values.map { m + (key to it) } } }
    /** Every fact combination a screen reads, with each of its preference combinations. */
    private fun cases(screen: ReferenceGroupTwoScreens.Screen): List<ReferenceGroupTwoScreens.Facts> {
        val spell = if (screen == ref.typing) ref.spellStates else listOf(Triple("1", null, null))
        val keyboard = if (screen == ref.physicalKeyboard) listOf(false, true) else listOf(false)
        val models = if (screen == ref.dictation) ref.modelStates.flatMap { a -> ref.modelStates.flatMap { b ->
            listOf(false, true).map { Triple(a, b, it) } } } else listOf(Triple(ModelState.Ready, ModelState.Ready, true))
        return combinations(screen.inputs).flatMap { combo -> spell.flatMap { (e, id, label) -> keyboard.flatMap { k ->
            models.map { (speech, voice, checked) ->
                ReferenceGroupTwoScreens.Facts(defaults() + combo, k, e, id, label, speech = speech, voice = voice, checked = checked)
            }
        } } }
    }
    private fun role(kind: String) = when (kind) {
        "toggle" -> "toggle"; "heading" -> "heading"; "info", "header", "card", "status", "note" -> "text"; "spell" -> "card"
        "field" -> "edit"; else -> "button"
    }

    @Test fun everyRowVisibilityEnableStateRoleAndTextMatchesFrozenCompose() {
        for (screen in ref.screens) for (f in cases(screen)) KeyboardRenderer(arrayOf(font)).use { n ->
            prepare(n, f)
            val s = state(n, SettingsUiOp.NAVIGATE, screen.route)
            val label = "${screen.route} ${f.p.filterKeys { it in screen.inputs }} ${f.spellEnabled} ${f.spellId} ${f.spellLabel} ${f.keyboardAttached}"
            assertEquals(screen.route, s.text("route"), label)
            val all = nodes(s)
            val rows = all.filter { it.int("id") in 1000 until 2000 }
            assertEquals(screen.rows.indices.filter { screen.rows[it].visible(f) }.map { 1000 + it }, rows.map { it.int("id") }, label)
            for (node in rows) {
                val i = node.int("id") - 1000
                val row = screen.rows[i]
                assertEquals(row.enabled(f), node.bool("enabled"), "enabled row $i $label")
                assertEquals(role(row.kind), node.text("role"), "role row $i $label")
                val clickable = row.kind !in passive
                assertEquals(clickable && row.enabled(f), 1 in node.getValue("actions").jsonArray.map { it.jsonPrimitive.int }, "click row $i $label")
                if (row.kind == "toggle") assertEquals(f.p.getValue(row.key).toBoolean(), node.bool("checked"))
                if (row.kind == "spell") {
                    val (_, text, secondary) = ref.spellCard(f, ::s)
                    assertEquals(text, node.text("label"), label); assertEquals(secondary, node.text("state"), label)
                }
                ref.summary(row.key, f, ::s)?.let { assertEquals(it, node.text("state"), "summary row $i") }
            }
            // The card's hyperlink and the header's image are their own nodes.
            val second = all.filter { it.int("id") in 6000 until 7000 }.associateBy { it.int("id") - 6000 }
            val expected = screen.rows.indices.filter { screen.rows[it].visible(f) && screen.rows[it].kind in listOf("card", "header") }
            assertEquals(expected, second.keys.sorted(), label)
            for ((i, node) in second) when (screen.rows[i].kind) {
                "card" -> { assertEquals("link", node.text("role")); assertEquals(s("pref__correction__auto_space_punctuation__feedback_link"), node.text("label")) }
                else -> { assertEquals("image", node.text("role")); assertEquals(s("about__app_icon__description"), node.text("description")) }
            }
            assertEquals(screen.preview, all.any { it.int("id") == 2 } && all.any { it.int("id") == 3 }, "preview field $label")
        }
    }

    /** A list dialog: pick an entry, then confirm, cancel or reset to the default. */
    private fun choice(n: KeyboardRenderer, i: Int, row: ReferenceGroupTwoScreens.Row, p: MutableMap<String, String>, random: Random) {
        assertTrue(state(n, SettingsUiOp.ACTIVATE, (1000 + i).toString()).bool("popup"))
        val at = random.nextInt(row.choices.size)
        assertEquals(row.choices.size, nodes(state(n, SettingsUiOp.ACTIVATE, (3000 + at).toString())).count { it.text("role") == "radio" })
        val (button, expected) = when (random.nextInt(4)) {
            0 -> "6" to emptyList()
            1 -> "5" to listOf(listOf("reset", row.key)).also { p[row.key] = defaults().getValue(row.key) }
            else -> "7" to listOf(listOf("write", row.key, row.choices[at])).also { p[row.key] = row.choices[at] }
        }
        val reply = state(n, SettingsUiOp.ACTIVATE, button)
        assertFalse(reply.bool("popup")); assertEquals(expected, effects(reply), "list row $i")
    }
    private fun confirm(n: KeyboardRenderer, i: Int, row: ReferenceGroupTwoScreens.Row, f: ReferenceGroupTwoScreens.Facts, random: Random) {
        val open = state(n, SettingsUiOp.ACTIVATE, (1000 + i).toString())
        assertTrue(open.bool("popup"))
        val dialog = nodes(open).associateBy { it.int("id") }
        assertNull(dialog[5], "the confirmation has no neutral button")
        assertEquals(s("action__cancel"), dialog.getValue(6).text("label")); assertEquals(s("action__delete"), dialog.getValue(7).text("label"))
        val yes = random.nextBoolean()
        val reply = state(n, SettingsUiOp.ACTIVATE, if (yes) "7" else "6")
        assertFalse(reply.bool("popup"))
        assertEquals(if (yes) listOf(row.click(f, ::s)) else emptyList(), effects(reply), "confirm row $i")
    }
    /** The user dictionary whose words Rust asked for; it asks again only for another, as no report answers here. */
    private var wordsAskedFor: String? = null
    private fun navigate(n: KeyboardRenderer, i: Int, row: ReferenceGroupTwoScreens.Row, screen: ReferenceGroupTwoScreens.Screen) {
        val native = n.settings(SettingsUiOp.ROUTES)[2].split("\n")
        val reply = state(n, SettingsUiOp.ACTIVATE, (1000 + i).toString())
        if (row.route in native) {
            assertEquals(row.route, reply.text("route"))
            // The licences list asks Kotlin for the AboutLibraries resource as it opens, and a user dictionary for its words.
            val plans = when (row.route) {
                "settings/about/third-party-licenses" -> listOf(listOf("load", "libraries"))
                in wordRoutes -> row.route.substringAfterLast('/').let { d ->
                    listOfNotNull(listOf(SettingsUiWordEffect.LOAD, d).takeIf { wordsAskedFor != d }).also { wordsAskedFor = d }
                }
                else -> emptyList()
            }
            assertEquals(plans, effects(reply))
            state(n, SettingsUiOp.BACK, "3", "0")
        } else assertEquals(listOf(listOf("compose", row.route, native.joinToString("|"))), effects(reply))
        assertEquals(screen.route, route(n))
    }

    @Test fun clicksConfirmationsLinksAndFactChangesMatchOriginalActions() {
        val random = Random(20261115)
        for (screen in ref.screens) KeyboardRenderer(arrayOf(font)).use { n ->
            val p = defaults().toMutableMap()
            var (e, id, label) = ref.spellStates.random(random)
            var attached = random.nextBoolean()
            val f = { ReferenceGroupTwoScreens.Facts(p, attached, e, id, label) }
            prepare(n, f())
            state(n, SettingsUiOp.NAVIGATE, screen.route)
            val touched = mutableSetOf<Int>()
            var owner = -1
            repeat(400) {
                // The device can change underneath: the system spell checker, a keyboard plugged in.
                if (random.nextInt(8) == 0) {
                    // Half the time the Floris spell checker, which enables the language list.
                    (if (random.nextBoolean()) ref.spellStates[ref.spellStates.size - 7] else ref.spellStates.random(random))
                        .let { e = it.first; id = it.second; label = it.third }
                    attached = random.nextBoolean()
                    n.settings(SettingsUiOp.FACTS, facts(f()))
                }
                val i = screen.rows.indices.filter {
                    val r = screen.rows[it]
                    r.visible(f()) && r.enabled(f()) && r.kind !in inert
                }.random(random)
                val row = screen.rows[i]
                touched += i
                when (row.kind) {
                    "toggle" -> {
                        p[row.key] = (!p.getValue(row.key).toBoolean()).toString()
                        assertEquals(listOf(listOf("write", row.key, p.getValue(row.key))), effects(state(n, SettingsUiOp.ACTIVATE, (1000 + i).toString())))
                    }
                    "choice" -> choice(n, i, row, p, random)
                    "confirm" -> confirm(n, i, row, f(), random)
                    "route" -> navigate(n, i, row, screen)
                    "field" -> {
                        // The first activation hands the editor over and restarts the IME's input.
                        val opened = effects(state(n, SettingsUiOp.ACTIVATE, (1000 + i).toString()))
                        assertEquals((if (owner != i) listOf(listOf("restartInput")) else emptyList()) + listOf(listOf("editor")), opened)
                        owner = i
                        val typed = listOf("a", "Kubernetes", "\n", "post gress -> Postgres", "é").random(random)
                        p[row.key] = p.getValue(row.key) + typed
                        assertEquals(listOf(listOf("write", row.key, p.getValue(row.key))), effects(state(n, SettingsUiOp.EDIT, "commit", typed, "1")))
                    }
                    "card" -> assertEquals(listOf(row.click(f(), ::s)), effects(state(n, SettingsUiOp.ACTIVATE, (6000 + i).toString())), "link row $i")
                    else -> assertEquals(listOf(row.click(f(), ::s)), effects(state(n, SettingsUiOp.ACTIVATE, (1000 + i).toString())), "click row $i")
                }
                assertEquals(screen.route, route(n))
            }
            val actionable = screen.rows.indices.filter { screen.rows[it].kind !in inert && screen.rows[it].visible(
                ReferenceGroupTwoScreens.Facts(p + ("correction__auto_space_punctuation" to "true"), true, "1", null, null)) }
            assertEquals(actionable.toSet(), touched, "${screen.route} rows never exercised")
        }
    }

    @Test fun factPairsLeaveNullSettingsOutAndParseThePackageAsComponentNameDoes() {
        for ((e, id, label) in ref.spellStates) {
            val f = ReferenceGroupTwoScreens.Facts(emptyMap(), true, e, id, label)
            val pairs = facts(f).toList().chunked(2).associate { it[0] to it[1] }
            assertEquals(e, pairs["spell_enabled"]); assertEquals(id, pairs["spell_id"])
            assertEquals(f.spellPackage, pairs["spell_package"]); assertEquals(label, pairs["spell_label"])
            assertEquals("true", pairs["physical_keyboard"]); assertEquals(f.packageName, pairs["package"])
        }
        assertEquals("a.b", SettingsFacts.spellPackage("a.b/.C")); assertEquals("null", SettingsFacts.spellPackage("/"))
    }

    @Test fun nativeRoutesAddGroupTwoAndKeepTheirScreensAcrossRecreation() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            val native = n.settings(SettingsUiOp.ROUTES)[2].split("\n")
            // Other and Theme, then Back up and Restore, then the language and user dictionary screens follow group two.
            val group = native.dropLast(ReferenceOtherThemeScreens.screens.size + 2 + languageRoutes.size + wordRoutes.size)
            assertEquals(ref.screens.map { it.route } + "settings/about/third-party-licenses", group.takeLast(ref.screens.size + 1))
            val f = ReferenceGroupTwoScreens.Facts(defaults(), false, "1", null, null)
            prepare(n, f)
            state(n, SettingsUiOp.NAVIGATE, "settings/typing"); state(n, SettingsUiOp.NAVIGATE, "settings/dictionary")
            val saved = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
            assertEquals("settings/home|settings/typing|settings/dictionary", saved[1])
            KeyboardRenderer(arrayOf(font)).use { recreated ->
                prepare(recreated, f); state(recreated, SettingsUiOp.RESTORE, *saved)
                assertEquals("settings/dictionary", route(recreated))
            }
        }
    }
}
