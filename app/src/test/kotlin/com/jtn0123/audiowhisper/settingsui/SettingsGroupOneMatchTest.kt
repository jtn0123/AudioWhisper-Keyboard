package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import dev.patrickgold.florisboard.app.ReferenceSettingsModel
import dev.patrickgold.florisboard.app.referenceSettingsBounds
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import kotlinx.serialization.json.*
import kotlin.math.roundToInt
import kotlin.random.Random
import kotlin.test.*

/** The five group 1 screens Rust draws, against [ReferenceGroupOneScreens]. */
class SettingsGroupOneMatchTest {
    private val ref = ReferenceGroupOneScreens
    private fun <T : Any> raw(p: PreferenceData<T>) = p.serializer.serialize(p.default).orEmpty()
    private fun defaults() = ReferenceSettingsModel().declaredPreferenceEntries.values.associate { it.key to raw(it) }
    private fun state(n: KeyboardRenderer, op: SettingsUiOp = SettingsUiOp.SAVE, vararg input: String) = Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun JsonObject.int(key: String) = getValue(key).jsonPrimitive.int
    private fun JsonObject.bool(key: String) = getValue(key).jsonPrimitive.boolean
    /** RustSettingsView.configure's packet: tall enough that every row is on screen. */
    private fun prepare(n: KeyboardRenderer, p: Map<String, String>, vibrator: Boolean = true, amplitude: Boolean = true) {
        n.settings(SettingsUiOp.PREFERENCES, p.flatMap { listOf(it.key, it.value) }.toTypedArray())
        val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
        n.settings(SettingsUiOp.CONFIG, (listOf("1080", "8192", "1", "1") + colors + listOf("true", "true") + colors +
            (0..32).map { it.toFloat().toString() } + listOf("0", "4294967295", "$vibrator", "$amplitude")).toTypedArray())
    }
    private fun combinations(inputs: Map<String, List<String>>): List<Map<String, String>> =
        inputs.entries.fold(listOf(emptyMap())) { acc, (key, values) -> acc.flatMap { m -> values.map { m + (key to it) } } }

    @Test fun referenceKeysAreDeclaredPreferencesAndBoundsAreFrozen() {
        val declared = defaults().keys
        for (screen in ref.screens) for (row in screen.rows) {
            if (row.kind in listOf("heading", "fixed", "confirm")) continue
            for (key in listOf(row.key, row.switch, row.second).filter { it.isNotEmpty() }) assertTrue(key in declared, key)
            if (row.kind == "slider") assertTrue(row.key in referenceSettingsBounds, row.key)
        }
    }

    @Test fun everyRowVisibilityEnableStateAndRoleMatchesFrozenCompose() {
        for (screen in ref.screens) for (vibrator in listOf(false to false, true to false, true to true)) {
            for (combo in combinations(screen.inputs)) {
                val p = defaults() + combo
                val facts = ReferenceGroupOneScreens.Facts(p, vibrator.first, vibrator.second)
                KeyboardRenderer(emptyArray()).use { n ->
                    prepare(n, p, vibrator.first, vibrator.second)
                    val s = state(n, SettingsUiOp.NAVIGATE, screen.route)
                    assertEquals(screen.route, s.getValue("route").jsonPrimitive.content)
                    val rows = nodes(s).filter { it.int("id") in 1000 until 2000 }
                    val label = "${screen.route} $combo $vibrator"
                    assertEquals(screen.rows.indices.filter { screen.rows[it].visible(facts) }.map { 1000 + it }, rows.map { it.int("id") }, label)
                    for (node in rows) {
                        val i = node.int("id") - 1000
                        val row = screen.rows[i]
                        // The fixed row is a Preference without onClick. Its condition only fades it:
                        // Material's ListItem node carries no disabled state, so Compose reports it enabled.
                        assertEquals(row.enabled(facts) || row.kind == "fixed", node.bool("enabled"), "enabled row $i $label")
                        assertEquals(when (row.kind) { "toggle" -> "toggle"; "heading" -> "heading"; "fixed" -> "text"; else -> "button" },
                            node.getValue("role").jsonPrimitive.content, "role row $i $label")
                        if (!row.enabled(facts)) assertTrue(node.getValue("actions").jsonArray.isEmpty(), "disabled row $i offers an action")
                        if (row.kind == "toggle") assertEquals(p.getValue(row.key).toBoolean(), node.bool("checked"))
                        else assertEquals(JsonNull, node.getValue("checked"), "row $i has toggle semantics")
                    }
                    val switches = nodes(s).filter { it.int("id") in 2000 until 3000 }.map { it.int("id") - 2000 }
                    assertEquals(screen.rows.indices.filter { screen.rows[it].switch.isNotEmpty() && screen.rows[it].visible(facts) }, switches)
                }
            }
        }
    }

    /** One random dialog transaction on [row]; returns the preferences Compose would have written. */
    private fun dialog(n: KeyboardRenderer, i: Int, row: ReferenceGroupOneScreens.Row, p: MutableMap<String, String>,
                       defaults: Map<String, String>, random: Random) {
        var reply = state(n, SettingsUiOp.ACTIVATE, (1000 + i).toString())
        assertTrue(reply.bool("popup"))
        val expected = mutableListOf<List<String>>()
        if (row.kind == "confirm") {
            assertTrue(nodes(reply).none { it.int("id") == 5 }, "the reset confirmation has no neutral button")
            val confirm = random.nextBoolean()
            reply = state(n, SettingsUiOp.ACTIVATE, if (confirm) "7" else "6")
            if (confirm) expected += listOf("deleteEmojiHistory", row.confirm)
            assertFalse(reply.bool("popup"))
            assertEquals(expected, effects(reply), "confirm row $i")
            return
        }
        val reset = random.nextInt(5) == 0
        val cancel = random.nextInt(4) == 0
        val keys = listOf(row.key, if (row.kind == "choice") row.switch else row.second).filter { it.isNotEmpty() }
        var choice = ""
        var switchValue = row.switch.isEmpty() || p.getValue(row.switch).toBoolean()
        val sliderValues = mutableMapOf<String, String>()
        if (row.kind == "choice") {
            if (!switchValue) { state(n, SettingsUiOp.ACTIVATE, "8"); switchValue = true }
            val at = random.nextInt(row.choices.size)
            reply = state(n, SettingsUiOp.ACTIVATE, (3000 + at).toString()); choice = row.choices[at]
            assertEquals(row.choices.size, nodes(reply).count { it.getValue("role").jsonPrimitive.content == "radio" })
            if (row.switch.isNotEmpty() && random.nextInt(3) == 0) { state(n, SettingsUiOp.ACTIVATE, "8"); switchValue = !switchValue }
        } else {
            for ((slot, key) in keys.withIndex()) {
                val (min, max, step) = referenceSettingsBounds.getValue(key).map { it.toFloat() }
                val input = min - 10f + random.nextFloat() * (max - min + 20f)
                val value = (min + ((input.coerceIn(min, max) - min) / step).roundToInt() * step).coerceIn(min, max).toInt().toString()
                val moved = effects(state(n, SettingsUiOp.SLIDER, (4000 + slot).toString(), input.toString()))
                sliderValues[key] = value
                // onValueChangeFinished: JetPref previews with the other haptic setting's stored value.
                val preview = if (!row.preview) emptyList() else listOf(listOf("vibrate",
                    if (key == "input_feedback__haptic_vibration_duration") value else p.getValue("input_feedback__haptic_vibration_duration"),
                    if (key == "input_feedback__haptic_vibration_strength") value else p.getValue("input_feedback__haptic_vibration_strength")))
                assertEquals(preview, moved, "slider preview row $i")
            }
        }
        if (reset) {
            reply = state(n, SettingsUiOp.ACTIVATE, "5")
            for (key in keys) { p[key] = defaults.getValue(key); expected += listOf("reset", key) }
        } else {
            if (!cancel) for (key in keys) {
                val value = if (row.kind == "choice") { if (key == row.key) choice else switchValue.toString() } else sliderValues.getValue(key)
                p[key] = value; expected += listOf("write", key, value)
            }
            reply = state(n, SettingsUiOp.ACTIVATE, if (cancel) "6" else "7")
        }
        assertFalse(reply.bool("popup"))
        assertEquals(expected, effects(reply), "transaction row $i")
    }

    @Test fun togglesListsSlidersConfirmationsAndPreviewsMatchOriginalTransactions() {
        val random = Random(20261114)
        for (screen in ref.screens) KeyboardRenderer(emptyArray()).use { n ->
            val defaults = defaults()
            val p = defaults.toMutableMap()
            prepare(n, p)
            state(n, SettingsUiOp.NAVIGATE, screen.route)
            val touched = mutableSetOf<Int>()
            repeat(400) {
                val facts = { ReferenceGroupOneScreens.Facts(p, vibrator = true, amplitude = true) }
                val i = screen.rows.indices.filter {
                    screen.rows[it].visible(facts()) && screen.rows[it].enabled(facts()) && screen.rows[it].kind !in listOf("heading", "fixed")
                }.random(random)
                val row = screen.rows[i]
                touched += i
                if (row.kind == "toggle") {
                    p[row.key] = (!p.getValue(row.key).toBoolean()).toString()
                    assertEquals(listOf(listOf("write", row.key, p.getValue(row.key))), effects(state(n, SettingsUiOp.ACTIVATE, (1000 + i).toString())))
                } else dialog(n, i, row, p, defaults, random)
            }
            val actionable = screen.rows.indices.filter { screen.rows[it].kind !in listOf("heading", "fixed") }
            assertEquals(actionable.toSet(), touched, "${screen.route} rows never exercised")
        }
    }

    @Test fun pluralRequestsCoverEveryPluralSliderRangeAndTheTimeDial() {
        KeyboardRenderer(emptyArray()).use { n ->
            val requests = n.settings(SettingsUiOp.PLURALS)[2].split("\n").toSet()
            val expected = ref.plurals.map { (key, plural) ->
                val (min, max) = referenceSettingsBounds.getValue(key).map { it.toInt() }
                "$plural|$min|$max"
            }.toSet()
            assertEquals(expected + ReferenceOtherThemeScreens.dialPlurals, requests)
        }
    }

    @Test fun nativeRoutesAreHomeKeyboardAndEveryDrawnScreenAndOthersFallBackWithThem() {
        KeyboardRenderer(emptyArray()).use { n ->
            val native = listOf("settings/home", "settings/keyboard") + ref.screens.map { it.route } +
                ReferenceGroupTwoScreens.screens.map { it.route } + "settings/about/third-party-licenses" + "settings/about/project-license" +
                ReferenceOtherThemeScreens.screens.map { it.route } + "settings/other/backup" + "settings/other/restore" +
                languageRoutes + wordRoutes + themeRoutes + "setup" + themeEditorRoutes + extensionRoutes
            assertEquals(native, n.settings(SettingsUiOp.ROUTES)[2].split("\n"))
            prepare(n, defaults())
            // Editing an extension stays in Compose.
            val compose = "ext/edit/my.themes"
            assertEquals(listOf(listOf("compose", compose, native.joinToString("|"))), effects(state(n, SettingsUiOp.NAVIGATE, compose)))
            assertEquals("settings/home", state(n).getValue("route").jsonPrimitive.content)
        }
    }

    @Test fun deepStacksScrollsAndDialogDraftsSurviveRecreation() {
        KeyboardRenderer(emptyArray()).use { n ->
            val p = defaults()
            val small = arrayOf("480", "640", "1", "1", "4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814", "true", "true")
            prepare(n, p)
            n.settings(SettingsUiOp.CONFIG, small)
            state(n, SettingsUiOp.NAVIGATE, "settings/keyboard"); state(n, SettingsUiOp.SCROLL, "0.5")
            state(n, SettingsUiOp.NAVIGATE, "settings/keyboard/input-feedback"); state(n, SettingsUiOp.SCROLL, "0.05")
            val slider = ref.inputFeedback.rows.indexOfFirst { it.key == "input_feedback__audio_volume" }
            state(n, SettingsUiOp.ACTIVATE, (1000 + slider).toString()); state(n, SettingsUiOp.SLIDER, "4000", "77")
            val saved = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
            assertEquals("settings/home|settings/keyboard|settings/keyboard/input-feedback", saved[1])
            assertEquals(3, saved[2].split("|").size)
            assertTrue(saved[2].split("|").drop(1).all { it.toFloat() > 0f }, "each entry keeps its own scroll")
            KeyboardRenderer(emptyArray()).use { recreated ->
                // The same window: a scroll is kept only where the content still reaches it.
                prepare(recreated, p); recreated.settings(SettingsUiOp.CONFIG, small)
                state(recreated, SettingsUiOp.RESTORE, *saved)
                assertContentEquals(saved, recreated.settings(SettingsUiOp.SAVE).drop(3).toTypedArray())
                assertEquals(listOf(listOf("write", "input_feedback__audio_volume", "77")), effects(state(recreated, SettingsUiOp.ACTIVATE, "7")))
                state(recreated, SettingsUiOp.BACK, "3", "0")
                assertEquals("settings/keyboard", state(recreated).getValue("route").jsonPrimitive.content)
            }
            // A stack Rust cannot draw, or scrolls that do not match it, restores to home.
            for (bad in listOf(saved.copyOf().apply { this[1] = "settings/home|ext/edit/my.themes" },
                saved.copyOf().apply { this[2] = "0" })) KeyboardRenderer(emptyArray()).use { r ->
                prepare(r, p); state(r, SettingsUiOp.RESTORE, *bad)
                assertEquals("settings/home", state(r).getValue("route").jsonPrimitive.content)
            }
        }
    }
}
