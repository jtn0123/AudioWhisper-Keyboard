package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.dictation.Dictation.Companion.KEEP_LOADED_HIDDEN_MS
import com.jtn0123.audiowhisper.dictation.Dictation.Companion.KEEP_LOADED_MS
import kotlin.test.Test
import kotlin.test.assertEquals

/** When the speech model is let go of, for a dictation that ended at uptime 10,000. */
class ReleaseAtTest {
    private val ended = 10_000L

    @Test
    fun withTheKeyboardUpTheModelIsKeptForTheFullTime() {
        assertEquals(ended + KEEP_LOADED_MS, releaseAt(ended, now = ended, keyboardShown = true))
        assertEquals(ended + KEEP_LOADED_MS, releaseAt(ended, now = ended + 90_000, keyboardShown = true))
    }

    @Test
    fun hidingTheKeyboardLetsGoOfItSooner() {
        val hidden = ended + 30_000
        assertEquals(hidden + KEEP_LOADED_HIDDEN_MS, releaseAt(ended, now = hidden, keyboardShown = false))
    }

    @Test
    fun hidingItNearTheEndDoesNotKeepItLonger() {
        val hidden = ended + KEEP_LOADED_MS - 10_000
        assertEquals(ended + KEEP_LOADED_MS, releaseAt(ended, now = hidden, keyboardShown = false))
    }

    @Test
    fun showingTheKeyboardAgainRestoresTheFullTime() {
        assertEquals(ended + KEEP_LOADED_MS, releaseAt(ended, now = ended + 45_000, keyboardShown = true))
    }
}
