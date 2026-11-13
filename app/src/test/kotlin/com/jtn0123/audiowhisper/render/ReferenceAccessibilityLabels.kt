package com.jtn0123.audiowhisper.render

import dev.patrickgold.florisboard.ime.text.key.KeyCode

/** Frozen Compose QuickAction.computeDisplayName/Button rules; LatinIME fallback for keys
 * whose Compose icons have null contentDescription. Independent of production Rust/resources. */
internal object ReferenceAccessibilityLabels {
    private val actions = mapOf(
        KeyCode.ARROW_UP to "Arrow up", KeyCode.ARROW_DOWN to "Arrow down", KeyCode.ARROW_LEFT to "Arrow left", KeyCode.ARROW_RIGHT to "Arrow right",
        KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP to "Clear primary clipboard clip", KeyCode.CLIPBOARD_COPY to "Copy", KeyCode.CLIPBOARD_CUT to "Cut",
        KeyCode.CLIPBOARD_PASTE to "Paste", KeyCode.CLIPBOARD_SELECT_ALL to "Select all", KeyCode.FORWARD_DELETE to "Forward delete",
        KeyCode.IME_UI_MODE_CLIPBOARD to "Clipboard", KeyCode.IME_UI_MODE_MEDIA to "Emoji", KeyCode.LANGUAGE_SWITCH to "Switch language",
        KeyCode.SETTINGS to "Settings", KeyCode.UNDO to "Undo", KeyCode.REDO to "Redo", KeyCode.TOGGLE_ACTIONS_OVERFLOW to "More actions",
        KeyCode.TOGGLE_INCOGNITO_MODE to "Incognito", KeyCode.TOGGLE_AUTOCORRECT to "Autocorrect", KeyCode.VOICE_INPUT to "Voice input",
        KeyCode.READ_ALOUD to "Read aloud", KeyCode.IME_HIDE_UI to "Hide keyboard", KeyCode.TOGGLE_COMPACT_LAYOUT to "One-handed",
        KeyCode.NOOP to "None", KeyCode.DRAG_MARKER to "",
    )
    private val symbols = mapOf(
        "!" to "Exclamation mark", "\"" to "Quotation mark", "#" to "Number sign", "$" to "Dollar sign",
        "%" to "Percent", "&" to "Ampersand", "'" to "Apostrophe", "(" to "Left parenthesis", ")" to "Right parenthesis",
        "*" to "Asterisk", "+" to "Plus", "," to "Comma", "-" to "Hyphen", "." to "Period", "/" to "Slash",
        ":" to "Colon", ";" to "Semicolon", "<" to "Less than", "=" to "Equals", ">" to "Greater than", "?" to "Question mark",
        "@" to "At sign", "[" to "Left bracket", "\\" to "Backslash", "]" to "Right bracket", "^" to "Caret",
        "_" to "Underscore", "`" to "Grave accent", "{" to "Left brace", "|" to "Vertical bar", "}" to "Right brace",
        "~" to "Tilde", "€" to "Euro", "£" to "Pound sterling", "¥" to "Yen", "¢" to "Cent", "°" to "Degree",
        "×" to "Multiplication sign", "÷" to "Division sign",
    )
    fun action(code: Int, text: String, lit: Int = 0): String = when {
        code == KeyCode.VOICE_INPUT && lit == 1 -> "Stop dictating"
        code == KeyCode.READ_ALOUD && lit == 2 -> "Stop reading"
        else -> actions[code] ?: text
    }
    fun key(code: Int, text: String, shift: String): String = when (code) {
        KeyCode.SHIFT -> when (shift) { "caps_lock" -> "Caps lock enabled"; "unshifted" -> "Shift"; else -> "Shift enabled" }
        KeyCode.CAPS_LOCK -> "Caps lock enabled"
        KeyCode.DELETE, KeyCode.DELETE_WORD -> "Delete"
        KeyCode.SPACE, KeyCode.CJK_SPACE -> "Space"
        KeyCode.ENTER -> if (text.isBlank() || text == "enter") "Return" else text
        KeyCode.TAB -> "Tab"
        KeyCode.VIEW_CHARACTERS -> "Letters"
        KeyCode.VIEW_SYMBOLS -> "Symbols"
        KeyCode.VIEW_SYMBOLS2 -> "More symbols"
        KeyCode.VIEW_NUMERIC, KeyCode.VIEW_NUMERIC_ADVANCED -> "Numeric"
        KeyCode.VIEW_PHONE, KeyCode.VIEW_PHONE2 -> "Phone"
        KeyCode.CTRL, KeyCode.CTRL_LOCK -> "Control"
        KeyCode.ALT, KeyCode.ALT_LOCK -> "Alt"
        KeyCode.FN, KeyCode.FN_LOCK -> "Function"
        KeyCode.ESCAPE -> "Escape"
        else -> if (code < 0 && code != KeyCode.MULTIPLE_CODE_POINTS && code != KeyCode.URI_COMPONENT_TLD) action(code, text)
            else symbols[text] ?: text
    }
    fun allActions(): Map<Int, String> = actions
}
