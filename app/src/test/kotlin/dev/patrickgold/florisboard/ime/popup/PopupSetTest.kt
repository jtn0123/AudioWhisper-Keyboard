package dev.patrickgold.florisboard.ime.popup

import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.key.KeyHintMode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlin.test.Test
import kotlin.test.assertEquals

/** The long-press popup of a key that has an accent, a symbol hint and a number hint. */
class PopupSetTest {
    private val accent = TextKeyData(code = 'é'.code, label = "é")
    private val symbol = TextKeyData(code = '@'.code, label = "@")
    private val number = TextKeyData(code = '3'.code, label = "3")

    private fun order(symbolMode: KeyHintMode, numberMode: KeyHintMode): List<String> {
        val set = MutablePopupSet<TextKeyData>(main = accent, symbolHint = symbol, numberHint = number)
        return set.getPopupKeys(KeyHintConfiguration(symbolMode, numberMode, mergeHintPopups = false))
            .prioritized.map { it.label }
    }

    @Test
    fun bothHintsInHintPriorityPutTheSymbolFirstThenTheNumber() {
        assertEquals(listOf("@", "3", "é"), order(KeyHintMode.HINT_PRIORITY, KeyHintMode.HINT_PRIORITY))
    }

    @Test
    fun aSymbolInHintPriorityComesBeforeTheAccentOfANumberThatIsNot() {
        assertEquals(listOf("@", "é", "3"), order(KeyHintMode.HINT_PRIORITY, KeyHintMode.ACCENT_PRIORITY))
        assertEquals(listOf("@", "é", "3"), order(KeyHintMode.HINT_PRIORITY, KeyHintMode.SMART_PRIORITY))
    }

    @Test
    fun aNumberInHintPriorityComesFirstWhenTheSymbolIsNot() {
        assertEquals(listOf("3", "é", "@"), order(KeyHintMode.SMART_PRIORITY, KeyHintMode.HINT_PRIORITY))
        assertEquals(listOf("3", "é", "@"), order(KeyHintMode.ACCENT_PRIORITY, KeyHintMode.HINT_PRIORITY))
    }

    @Test
    fun theDefaultSmartPriorityKeepsTheAccentFirst() {
        assertEquals(listOf("é", "@", "3"), order(KeyHintMode.SMART_PRIORITY, KeyHintMode.SMART_PRIORITY))
    }
}
