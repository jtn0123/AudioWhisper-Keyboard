package dev.patrickgold.florisboard.app

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * Every way into settings opens the Rust host once the keyboard is set up (docs/decisions.md,
 * "Rust draws every settings screen it can"). The intents are the ones the app sends or declares.
 */
class SettingsEntryTest {
    private val view = "android.intent.action.VIEW"
    private val send = "android.intent.action.SEND"
    private val main = "android.intent.action.MAIN"

    private fun rust(imeSetUp: Boolean, action: String?, data: String?, allowRust: Boolean = true) =
        SettingsEntry.rust(allowRust, imeSetUp, SettingsEntry.linkRoute(action, data))

    @Test fun theLauncherAndTheKeyboardsSettingsButtonOpenRustOnceSetUp() {
        // SettingsLauncherAlias sends MAIN; FlorisImeService.launchSettings sends no action or data.
        for (action in listOf(main, null)) {
            assertTrue(rust(imeSetUp = true, action, null))
            assertFalse(rust(imeSetUp = false, action, null), "Setup still comes first")
        }
    }

    @Test fun aSettingsLinkOpensRustEvenDuringSetup() {
        // Dictation's "model settings" button, and any other ui://florisboard link.
        for (setUp in listOf(true, false)) {
            assertTrue(rust(setUp, view, "ui://florisboard/settings/dictation"))
            assertTrue(rust(setUp, view, "ui://florisboard/settings/theme"))
        }
        assertEquals("settings/dictation", SettingsEntry.linkRoute(view, "ui://florisboard/settings/dictation"))
        assertEquals("ext/view?id=a", SettingsEntry.linkRoute(view, "ui://florisboard/ext/view?id=a"))
    }

    @Test fun anExtensionFileIsNotASettingsLink() {
        assertNull(SettingsEntry.linkRoute(view, "content://downloads/1"))
        assertNull(SettingsEntry.linkRoute(send, null))
        assertNull(SettingsEntry.linkRoute(send, "ui://florisboard/settings/home"))
        assertNull(SettingsEntry.linkRoute(main, null))
        assertFalse(rust(imeSetUp = false, view, "content://downloads/1"))
    }

    @Test fun theComposeFallbackNeverOpensRust() {
        for (setUp in listOf(true, false)) {
            assertFalse(rust(setUp, view, "ui://florisboard/settings/theme", allowRust = false))
            assertFalse(rust(setUp, main, null, allowRust = false))
        }
    }
}
