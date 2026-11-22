package dev.patrickgold.florisboard.app

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * The Compose fallback's link handling. Creating or editing a theme from the theme manager opened
 * the extension editor through a browsable link; Save popped the only screen on the fallback's
 * back stack, the stale editor stayed drawn, and a second Save crashed on its closed workspace.
 */
class ComposeFallbackLinksTest {
    private val actions = listOf("android.intent.action.VIEW", "android.intent.action.SEND", "android.intent.action.MAIN", null)

    @Test fun onlyLaterBrowsableLinksAreTaken() {
        for (action in actions) for (browsable in listOf(true, false)) {
            val taken = action == "android.intent.action.VIEW" && browsable
            assertEquals(taken, fallbackHandles(action, browsable, initial = false), "$action $browsable")
        }
    }

    @Test fun theStartingLinkIsLeftToTheNavController() {
        assertFalse(fallbackHandles("android.intent.action.VIEW", browsable = true, initial = true))
    }

    /** Files opened in, or shared with, the app are imported by Rust now (SettingsEntryTest), never here. */
    @Test fun filesAndSharesAreNotTaken() {
        for (initial in listOf(true, false)) {
            assertFalse(fallbackHandles("android.intent.action.VIEW", browsable = false, initial = initial))
            assertFalse(fallbackHandles("android.intent.action.SEND", browsable = false, initial = initial))
        }
    }

    @Test fun theFallbackLeavesOnceItsLastScreenIsPopped() {
        assertTrue(fallbackDone(ready = true, emptied = true))
        assertFalse(fallbackDone(ready = true, emptied = false))
    }

    @Test fun theFallbackStaysBeforeItsGraphIsSet() {
        assertFalse(fallbackDone(ready = false, emptied = true))
    }
}
