package dev.patrickgold.florisboard.ime.keyboard

import dev.patrickgold.florisboard.ime.keyboard.ReferenceKeyCodes as KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import kotlin.math.abs

/** Kotlin rules before stage 2.1. Android queries/casing are supplied by the caller. */
object ReferenceKeyRules {
    fun currency(code: Int) = when (code) {
        KeyCode.CURRENCY_SLOT_1, KeyCode.CURRENCY_SLOT_2, KeyCode.CURRENCY_SLOT_3,
        KeyCode.CURRENCY_SLOT_4, KeyCode.CURRENCY_SLOT_5, KeyCode.CURRENCY_SLOT_6 -> true
        else -> false
    }
    fun currencySlot(code: Int) = abs(code) - abs(KeyCode.CURRENCY_SLOT_1)
    val currencyDefaults = intArrayOf(36, 162, 8364, 163, 165, 8369)
    fun space(type: KeyType, code: Int) = type == KeyType.CHARACTER &&
        (code == KeyCode.SPACE || code == KeyCode.CJK_SPACE || code == KeyCode.HALF_SPACE || code == KeyCode.KESHIDA)

    fun text(code: Int, label: String, display: Boolean, nonSpacing: Boolean) = buildString {
        if (display || code == KeyCode.URI_COMPONENT_TLD || code < KeyCode.SPACE) {
            if (nonSpacing && !label.startsWith("◌")) append("◌")
            append(label)
        } else {
            try { appendCodePoint(code) } catch (_: Throwable) { }
        }
    }
    fun multi(codes: IntArray, label: String, display: Boolean) = buildString {
        if (display) append(label) else for (code in codes) {
            try { appendCodePoint(code) } catch (_: Throwable) { }
        }
    }
    fun popupMode(mode: Int) = mode == 0 || mode == 5 || mode == 2 || mode == 3
    fun popupGroup(group: Int): String? = when (group) {
        KeyData.GROUP_ENTER -> "~enter"
        KeyData.GROUP_LEFT -> "~left"
        KeyData.GROUP_RIGHT -> "~right"
        else -> null
    }
    // The original null-coalescing cascade; custom and fallback are separate per variation.
    fun popupSource(variation: Int, present: Int): Int {
        fun source(v: Int): Int? = (2 * v).takeIf { present and (1 shl it) != 0 }
            ?: (2 * v + 1).takeIf { present and (1 shl it) != 0 }
        var chosen: Int? = null
        if (chosen == null && variation == 3) chosen = source(3)
        if (chosen == null && (variation == 2 || variation == 3)) chosen = source(2)
        if (chosen == null && variation == 1) chosen = source(1)
        if (chosen == null && (variation == 1 || variation == 4)) chosen = source(4)
        if (chosen == null) chosen = source(0)
        return chosen ?: -1
    }
    fun phoneHint(code: Int): String? = when (code) {
        48 -> "+"; 49 -> ""; 50 -> "ABC"; 51 -> "DEF"; 52 -> "GHI"
        53 -> "JKL"; 54 -> "MNO"; 55 -> "PQRS"; 56 -> "TUV"; 57 -> "WXYZ"
        else -> null
    }
    fun enabled(code: Int, a: Int, b: Int) = when (code) {
        KeyCode.CLIPBOARD_COPY, KeyCode.CLIPBOARD_CUT -> a != 0 && b != 0
        KeyCode.CLIPBOARD_PASTE -> a == 0 && b != 0
        KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP, KeyCode.CLIPBOARD_SELECT_ALL, KeyCode.VOICE_INPUT -> a != 0
        KeyCode.TOGGLE_INCOGNITO_MODE -> when (a) { 0, 1 -> false; else -> b == 0 }
        KeyCode.LANGUAGE_SWITCH -> a > 1
        else -> true
    }
    fun visible(code: Int, enabled: Boolean, action: Int, language: Boolean): Boolean {
        val effective = if (enabled) action else 4
        return when (code) {
            KeyCode.IME_UI_MODE_TEXT, KeyCode.IME_UI_MODE_MEDIA -> when (effective) {
                4, 1, 2 -> false; 0 -> true; else -> !language
            }
            KeyCode.LANGUAGE_SWITCH -> when (effective) {
                4, 0 -> false; 1, 2 -> true; else -> language
            }
            else -> true
        }
    }

}
