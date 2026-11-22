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

    /** What onNewIntent took before the fix, for every intent it was handed. */
    private fun before(action: String?, browsable: Boolean, data: Boolean, clip: Boolean) =
        (action == "android.intent.action.VIEW" && (browsable || data)) || (action == "android.intent.action.SEND" && clip)

    @Test fun laterIntentsAreTakenAsBefore() {
        for (action in actions) for (bits in 0 until 8) {
            val (browsable, data, clip) = Triple(bits and 1 != 0, bits and 2 != 0, bits and 4 != 0)
            assertEquals(before(action, browsable, data, clip), fallbackHandles(action, browsable, data, clip, initial = false), "$action $bits")
        }
    }

    @Test fun theStartingLinkIsLeftToTheNavController() {
        assertFalse(fallbackHandles("android.intent.action.VIEW", browsable = true, data = true, clip = false, initial = true))
        assertFalse(fallbackHandles("android.intent.action.VIEW", browsable = true, data = false, clip = false, initial = true))
    }

    @Test fun startingFilesAndSharesAreStillImported() {
        assertTrue(fallbackHandles("android.intent.action.VIEW", browsable = false, data = true, clip = false, initial = true))
        assertTrue(fallbackHandles("android.intent.action.SEND", browsable = false, data = false, clip = true, initial = true))
        assertFalse(fallbackHandles("android.intent.action.SEND", browsable = false, data = true, clip = false, initial = true))
        assertFalse(fallbackHandles("android.intent.action.MAIN", browsable = true, data = true, clip = true, initial = true))
    }

    @Test fun theFallbackLeavesOnceItsLastScreenIsPopped() {
        assertTrue(fallbackDone(ready = true, emptied = true))
        assertFalse(fallbackDone(ready = true, emptied = false))
    }

    @Test fun theFallbackStaysBeforeItsGraphIsSet() {
        assertFalse(fallbackDone(ready = false, emptied = true))
    }
}
