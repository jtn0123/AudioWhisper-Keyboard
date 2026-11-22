package com.jtn0123.audiowhisper.settingsui

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import com.jtn0123.audiowhisper.rust.SettingsUiPermission
import com.jtn0123.audiowhisper.rust.SettingsUiSetupEffect
import com.jtn0123.audiowhisper.rust.SettingsUiSetupFact
import kotlinx.serialization.json.*
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * SetupScreen through the real natives, as FlorisAppActivity, RustSettingsView and SettingsSetup
 * drive it (stage4-setup.md): START, the input method facts in CONFIG, the permission facts in
 * FACTS, the PERMISSION answer, the effects SettingsSetup carries out, the TalkBack nodes and the
 * saved state.
 */
class SettingsSetupMatchTest {
    private val font = File("../fastlane/staging/fonts/Roboto-Light.ttf").absolutePath
    private val setUp = "internal__is_ime_set_up"
    private val answer = "internal__notification_permission_state"
    private fun state(n: KeyboardRenderer, op: SettingsUiOp, vararg input: String) = Json.parseToJsonElement(n.settings(op, arrayOf(*input))[0]).jsonObject
    private fun nodes(s: JsonObject) = s.getValue("nodes").jsonArray.map { it.jsonObject }
    private fun effects(s: JsonObject) = s.getValue("effects").jsonArray.map { a -> a.jsonArray.map { it.jsonPrimitive.content } }
    private fun JsonObject.text(key: String) = getValue(key).jsonPrimitive.content
    private fun JsonObject.flag(key: String) = getValue(key).jsonPrimitive.boolean
    private fun labelled(s: JsonObject, label: String) = nodes(s).single { it.text("label") == label }
    private fun heading(s: JsonObject) = nodes(s).single { it.text("role") == "heading" }.text("label")

    /** The strings as aapt compiles them: runs of white space, the line breaks in the notification text's, are one space. */
    private val strings: Map<String, String> by lazy {
        val pattern = Regex("""<string name="([^"]+)"[^>]*>(.*?)</string>""", RegexOption.DOT_MATCHES_ALL)
        pattern.findAll(File("src/main/res/values/strings.xml").readText()).associate { it.groupValues[1] to it.groupValues[2].replace(Regex("\\s+"), " ").trim() }
    }
    private val titles = listOf("setup__enable_ime__title", "setup__select_ime__title", "setup__grant_notification_permission__title", "setup__finish_up__title")
    /** What TalkBack reads for step [n]'s header. */
    private fun step(n: Int) = strings.getValue("setup__step__label").replace("{n}", "$n").replace("{count}", "4")
        .replace("{title}", strings.getValue(titles[n - 1]))
    private val colors = listOf("4294967295", "4278190080", "4283782485", "4281558681", "4291611852", "4293848814")
    /** RustSettingsView.configure's CONFIG for a 1080 by 2400 px phone at density 3, with the input method facts. */
    private fun config(n: KeyboardRenderer, enabled: Boolean, selected: Boolean) = state(n, SettingsUiOp.CONFIG, *(listOf("1080", "2400", "3", "1") + colors +
        listOf(enabled.toString(), selected.toString()) + colors + (0..32).map { (3 * it).toFloat().toString() } +
        listOf("0", "4294967295", "true", "true", "4278190080", "4278198272", "4278190080")).toTypedArray())
    private fun facts(n: KeyboardRenderer, granted: Boolean, rationale: Boolean) =
        state(n, SettingsUiOp.FACTS, SettingsUiSetupFact.NOTIFICATIONS, granted.toString(), SettingsUiSetupFact.RATIONALE, rationale.toString())
    /** RustSettingsView's first packets: the preferences, then the strings. */
    private fun bind(n: KeyboardRenderer, stored: String) {
        n.settings(SettingsUiOp.PREFERENCES, arrayOf(setUp, "false", answer, stored))
        n.settings(SettingsUiOp.RESOURCES, n.settings(SettingsUiOp.STRING_KEYS)[2].split("\n").filter { it in strings }.flatMap { listOf(it, strings.getValue(it)) }.toTypedArray())
    }
    /** A fresh activity as FlorisAppActivity opens it: bound, START, then the first configure. */
    private fun open(n: KeyboardRenderer, enabled: Boolean, selected: Boolean, stored: String, granted: Boolean = false): JsonObject {
        bind(n, stored)
        state(n, SettingsUiOp.START)
        config(n, enabled, selected)
        return facts(n, granted, rationale = true)
    }
    private fun press(n: KeyboardRenderer, s: JsonObject, label: String) = state(n, SettingsUiOp.ACTIVATE, labelled(s, label).text("id"))

    @Test fun aFirstRunWalksEveryStepAndEndsOnHome() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            var s = open(n, enabled = false, selected = false, stored = "NOT_SET")
            assertEquals("setup", s.text("route"))
            assertFalse(s.flag("canBack"), "Back leaves the app, as from Compose's start destination")
            assertEquals(step(1), heading(s))
            assertEquals(listOf(listOf("enable")), effects(press(n, s, strings.getValue("setup__enable_ime__open_settings_btn"))))
            // Enabled in the system's settings: the app comes back, on the next step.
            s = config(n, enabled = true, selected = false)
            assertEquals(listOf(listOf(SettingsUiSetupEffect.RETURN)), effects(s))
            assertEquals(step(2), heading(s))
            assertEquals(listOf(listOf("picker")), effects(press(n, s, strings.getValue("setup__select_ime__switch_keyboard_btn"))))
            s = config(n, enabled = true, selected = true)
            assertEquals(step(3), heading(s))
            assertEquals(listOf(listOf(SettingsUiSetupEffect.ASK)), effects(press(n, s, strings.getValue("setup__grant_notification_permission__btn"))))
            s = state(n, SettingsUiOp.PERMISSION, *SettingsUiPermission.row(SettingsUiPermission.GRANTED to "true", SettingsUiPermission.RATIONALE to "false"))
            assertEquals(listOf(listOf("write", answer, "GRANTED")), effects(s))
            assertEquals(step(4), heading(s))
            s = press(n, s, strings.getValue("setup__finish_up__finish_btn"))
            assertEquals(listOf(listOf("write", setUp, "true")), effects(s))
            assertEquals("settings/home", s.text("route"))
        }
    }

    @Test fun headersReadTheirStateAndADoneOneOpensAgain() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            val s = open(n, enabled = true, selected = false, stored = "NOT_SET")
            val done = labelled(s, step(1))
            assertEquals(listOf("button", strings.getValue("setup__step__done")), listOf(done.text("role"), done.text("state")))
            assertEquals(strings.getValue("setup__step__current"), labelled(s, step(2)).text("state"))
            val later = labelled(s, step(3))
            assertEquals(listOf("text", strings.getValue("setup__step__later")), listOf(later.text("role"), later.text("state")))
            assertTrue(later.getValue("actions").jsonArray.isEmpty(), "a later step does nothing yet")
            val reopened = state(n, SettingsUiOp.ACTIVATE, done.text("id"))
            assertEquals(step(1), heading(reopened))
            assertEquals(strings.getValue("setup__step__next"), labelled(reopened, step(2)).text("state"))
        }
    }

    @Test fun aDenialForGoodSendsTheUserToTheNotificationSettings() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            open(n, enabled = true, selected = true, stored = "NOT_SET")
            var s = state(n, SettingsUiOp.PERMISSION, *SettingsUiPermission.row(SettingsUiPermission.GRANTED to "false", SettingsUiPermission.RATIONALE to "false"))
            assertEquals(listOf(listOf("write", answer, "DENIED")), effects(s))
            s = state(n, SettingsUiOp.ACTIVATE, labelled(s, step(3)).text("id"))
            val status = labelled(s, strings.getValue("setup__notifications__off"))
            assertEquals("status", status.text("role"))
            assertTrue(status.flag("live"), "TalkBack announces the answer")
            s = press(n, s, strings.getValue("setup__notifications__settings_btn"))
            assertEquals(listOf(listOf(SettingsUiSetupEffect.NOTIFICATION_SETTINGS)), effects(s))
            // Turned on there: the next report answers the question.
            assertEquals(listOf(listOf("write", answer, "GRANTED")), effects(facts(n, granted = true, rationale = false)))
        }
    }

    @Test fun notificationsAlreadyAllowedSkipTheQuestion() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            val s = open(n, enabled = true, selected = true, stored = "NOT_SET", granted = true)
            assertEquals(listOf(listOf("write", answer, "GRANTED")), effects(s))
            assertEquals(step(4), heading(s))
        }
    }

    @Test fun theChosenStepSurvivesRecreationAndSetupOnlyRestoresAsTheRoot() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            val s = open(n, enabled = true, selected = false, stored = "NOT_SET")
            state(n, SettingsUiOp.ACTIVATE, labelled(s, step(1)).text("id"))
            val saved = n.settings(SettingsUiOp.SAVE).drop(3).toTypedArray()
            assertEquals(listOf("11", "setup", "1|0|false"), listOf(saved[0], saved[1], saved[19]))
            KeyboardRenderer(arrayOf(font)).use { recreated ->
                bind(recreated, "NOT_SET")
                state(recreated, SettingsUiOp.RESTORE, *saved)
                assertEquals(step(1), heading(config(recreated, enabled = true, selected = false)))
            }
            KeyboardRenderer(arrayOf(font)).use { tampered ->
                val restored = state(tampered, SettingsUiOp.RESTORE, *saved.copyOf().also { it[1] = "settings/home|setup"; it[2] = "0|0" })
                assertEquals("settings/home", restored.text("route"), "a saved state with setup above home is refused")
            }
        }
    }

    @Test fun aMalformedPacketIsRefused() {
        KeyboardRenderer(arrayOf(font)).use { n ->
            open(n, enabled = true, selected = true, stored = "NOT_SET")
            assertFailsWith<IllegalArgumentException> { n.settings(SettingsUiOp.PERMISSION, arrayOf("yes", "false")) }
            assertFailsWith<IllegalArgumentException> { n.settings(SettingsUiOp.PERMISSION, arrayOf("true")) }
            assertFailsWith<IllegalArgumentException> { n.settings(SettingsUiOp.START, arrayOf("settings/about", "settings/other")) }
        }
    }
}
