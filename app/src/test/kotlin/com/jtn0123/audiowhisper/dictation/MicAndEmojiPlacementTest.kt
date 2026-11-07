package com.jtn0123.audiowhisper.dictation

import dev.patrickgold.florisboard.app.FlorisPreferenceModelImpl
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionArrangement
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * The mic and emoji live in the suggestion bar (mic at its left end, emoji just right of the suggestions),
 * not in the bottom row, so the space bar gets their width.
 */
class MicAndEmojiPlacementTest {
    private val bottomRowLayout =
        File("src/main/assets/ime/keyboard/org.florisboard.layouts/layouts/charactersMod/default.json")
    private val prefs = FlorisPreferenceModelImpl()

    @Test
    fun `the bottom row has no mic key`() {
        val bottomRow = Json.parseToJsonElement(bottomRowLayout.readText()).jsonArray.last().jsonArray
        val codes = bottomRow.mapNotNull { it.jsonObject["code"]?.jsonPrimitive?.int }
        assertFalse(KeyCode.VOICE_INPUT in codes)
    }

    @Test
    fun `the emoji key beside the space bar is off`() {
        assertFalse(prefs.keyboard.utilityKeyEnabled.default)
    }

    @Test
    fun `the mic is pinned at the left end of the suggestion bar`() {
        assertEquals(QuickAction.InsertKey(TextKeyData.VOICE_INPUT), QuickActionArrangement.Default.stickyAction)
        assertTrue(prefs.smartbar.flipToggles.default)
    }

    @Test
    fun `the expanded actions do not repeat the pinned emoji button`() {
        val emoji = QuickAction.InsertKey(TextKeyData.IME_UI_MODE_MEDIA)
        assertTrue(emoji in QuickActionArrangement.Default.hiddenActions)
        assertFalse(emoji in QuickActionArrangement.Default.dynamicActions)
    }
}
