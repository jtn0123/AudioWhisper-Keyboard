package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.SettingsUiOp
import com.jtn0123.audiowhisper.rust.SettingsUiReply
import kotlinx.serialization.json.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

/**
 * Every way into settings opens the Rust host, which picks the first screen as FlorisAppActivity
 * and SetupScreen did (stage4-setup.md): setup until it is finished, unless a settings link asks
 * for a screen. The intents are the ones the app sends or declares.
 */
class SettingsEntryTest {
    private val view = "android.intent.action.VIEW"
    private val send = "android.intent.action.SEND"
    private val main = "android.intent.action.MAIN"

    /** The screen a fresh activity opens on, and the preferences it writes, as FlorisAppActivity asks. */
    private fun start(setUp: Boolean, notifications: String, action: String?, data: String?): Pair<String, List<List<String>>> =
        KeyboardRenderer(emptyArray()).use { n ->
            n.settings(SettingsUiOp.PREFERENCES, arrayOf("internal__is_ime_set_up", setUp.toString(), "internal__notification_permission_state", notifications))
            val link = SettingsEntry.linkRoute(action, data)
            val state = Json.parseToJsonElement(n.settings(SettingsUiOp.START, listOfNotNull(link).toTypedArray())[SettingsUiReply.STATE]).jsonObject
            val writes = state.getValue("effects").jsonArray.map { e -> e.jsonArray.map { it.jsonPrimitive.content } }.filter { it[0] == "write" }
            state.getValue("route").jsonPrimitive.content to writes
        }

    @Test fun theLauncherAndTheKeyboardsSettingsButtonOpenSetupUntilItIsFinished() {
        // SettingsLauncherAlias sends MAIN; FlorisImeService.launchSettings sends no action or data.
        for (action in listOf(main, null)) {
            assertEquals("settings/home" to emptyList(), start(setUp = true, "GRANTED", action, null))
            assertEquals("settings/home" to emptyList(), start(setUp = true, "DENIED", action, null))
            assertEquals("setup" to emptyList(), start(setUp = false, "GRANTED", action, null), "Setup still comes first")
        }
    }

    @Test fun anUnansweredNotificationQuestionSendsAFinishedUserThroughSetupAgain() {
        val reset = listOf(listOf("write", "internal__is_ime_set_up", "false"))
        assertEquals("setup" to reset, start(setUp = true, "NOT_SET", main, null))
        assertEquals("setup" to emptyList(), start(setUp = false, "NOT_SET", main, null))
    }

    @Test fun aSettingsLinkOpensItsScreenEvenDuringSetup() {
        // Dictation's "model settings" button, and any other ui://florisboard link.
        for (setUp in listOf(true, false)) {
            assertEquals("settings/dictation", start(setUp, "GRANTED", view, "ui://florisboard/settings/dictation").first)
        }
        assertEquals("settings/dictation", start(setUp = true, "NOT_SET", view, "ui://florisboard/settings/dictation").first)
        assertEquals("settings/dictation", SettingsEntry.linkRoute(view, "ui://florisboard/settings/dictation"))
        assertEquals("ext/view?id=a", SettingsEntry.linkRoute(view, "ui://florisboard/ext/view?id=a"))
    }

    @Test fun anExtensionFileIsNotASettingsLink() {
        assertNull(SettingsEntry.linkRoute(view, "content://downloads/1"))
        assertNull(SettingsEntry.linkRoute(send, null))
        assertNull(SettingsEntry.linkRoute(send, "ui://florisboard/settings/home"))
        assertNull(SettingsEntry.linkRoute(main, null))
        assertEquals("setup", start(setUp = false, "GRANTED", view, "content://downloads/1").first)
    }
}
