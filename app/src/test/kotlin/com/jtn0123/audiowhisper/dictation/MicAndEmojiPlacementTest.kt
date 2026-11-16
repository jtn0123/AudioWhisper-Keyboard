package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.rust.KeyModels
import dev.patrickgold.florisboard.app.FlorisPreferenceModelImpl
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickAction
import dev.patrickgold.florisboard.ime.smartbar.quickaction.QuickActionArrangement
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.UtilityKeyAction
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
 * The mic lives in the suggestion bar (at its left end, for a left-handed user), and emoji is the key
 * between the comma and the space bar, as in Gboard; the bar pins no emoji button.
 */
class MicAndEmojiPlacementTest {
    private val layouts = File("src/main/assets/ime/keyboard/org.florisboard.layouts/layouts/charactersMod")
    private val prefs = FlorisPreferenceModelImpl()

    private fun bottomRowCodes(layout: File): List<Int> =
        Json.parseToJsonElement(layout.readText()).jsonArray.last().jsonArray
            .mapNotNull { it.jsonObject["code"]?.jsonPrimitive?.int }

    @Test
    fun `the bottom row has no mic key`() {
        val codes = bottomRowCodes(File(layouts, "default.json"))
        assertFalse(KeyCode.VOICE_INPUT in codes)
    }

    @Test
    fun `the emoji key beside the space bar is on by default`() {
        assertTrue(prefs.keyboard.utilityKeyEnabled.default)
        assertEquals(UtilityKeyAction.SWITCH_TO_EMOJIS, prefs.keyboard.utilityKeyAction.default)
        val action = prefs.keyboard.utilityKeyAction.default.ordinal
        for (language in listOf(false, true)) {
            assertTrue(KeyModels.visible(KeyCode.IME_UI_MODE_MEDIA, true, action, language), "language=$language")
            assertFalse(KeyModels.visible(KeyCode.LANGUAGE_SWITCH, true, action, language), "language=$language")
        }
    }

    @Test
    fun `every character layout puts the emoji key right before the space bar`() {
        val files = layouts.listFiles { f -> f.extension == "json" }.orEmpty().sortedBy { it.name }
        assertEquals(listOf("default", "dvorak", "dvorak_de", "dvorak_se", "neo2"), files.map { it.nameWithoutExtension })
        for (file in files) {
            val codes = bottomRowCodes(file)
            val space = codes.indexOf(KeyCode.SPACE)
            assertTrue(space > 0, file.name)
            assertEquals(KeyCode.IME_UI_MODE_MEDIA, codes[space - 1], file.name)
        }
    }

    @Test
    fun `the mic is pinned at the left end of the suggestion bar`() {
        assertEquals(QuickAction.InsertKey(TextKeyData.VOICE_INPUT), QuickActionArrangement.Default.stickyAction)
        assertTrue(prefs.smartbar.flipToggles.default)
    }

    @Test
    fun `the expanded actions do not repeat the bottom row's emoji key`() {
        val emoji = QuickAction.InsertKey(TextKeyData.IME_UI_MODE_MEDIA)
        assertTrue(emoji in QuickActionArrangement.Default.hiddenActions)
        assertFalse(emoji in QuickActionArrangement.Default.dynamicActions)
    }
}
