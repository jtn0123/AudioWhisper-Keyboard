package com.jtn0123.audiowhisper.gif

import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType

/**
 * What the keyboard does while the user types into the GIF search box.
 *
 * The app's own text field is still connected behind the panel, so a key or
 * swipe that reached it would edit text the user can't see. Keys are
 * therefore let through only when known to be safe; anything else, including
 * keys added later, does nothing.
 */
object GifSearchKeys {
    enum class Route {
        /** Edits the search query. */
        SEARCH,

        /** Works as usual: it changes the keyboard, not the text (shift, layouts, languages, hiding). */
        KEYBOARD,

        /** Would edit the app's text (cursor keys, undo, paste, dictation), so it does nothing. */
        IGNORE,
    }

    /** How far a delete-key swipe has reached back in the query: [count] characters, or words. */
    data class SwipeDelete(val count: Int, val words: Boolean)

    private val SEARCH_KEYS = setOf(
        KeyCode.DELETE, KeyCode.DELETE_WORD, KeyCode.SPACE, KeyCode.CJK_SPACE, KeyCode.ENTER,
    )

    private val KEYBOARD_KEYS = setOf(
        KeyCode.SHIFT, KeyCode.CAPS_LOCK,
        KeyCode.VIEW_CHARACTERS, KeyCode.VIEW_SYMBOLS, KeyCode.VIEW_SYMBOLS2, KeyCode.VIEW_NUMERIC,
        KeyCode.VIEW_NUMERIC_ADVANCED, KeyCode.VIEW_PHONE, KeyCode.VIEW_PHONE2,
        KeyCode.IME_UI_MODE_TEXT, KeyCode.IME_UI_MODE_MEDIA, KeyCode.IME_UI_MODE_CLIPBOARD,
        KeyCode.IME_SHOW_UI, KeyCode.IME_HIDE_UI,
        KeyCode.IME_PREV_SUBTYPE, KeyCode.IME_NEXT_SUBTYPE, KeyCode.LANGUAGE_SWITCH,
        KeyCode.IME_SUBTYPE_PICKER, KeyCode.SHOW_SUBTYPE_PICKER,
        KeyCode.SYSTEM_INPUT_METHOD_PICKER, KeyCode.SYSTEM_PREV_INPUT_METHOD, KeyCode.SYSTEM_NEXT_INPUT_METHOD,
        KeyCode.SETTINGS, KeyCode.TOGGLE_SMARTBAR_VISIBILITY, KeyCode.TOGGLE_INCOGNITO_MODE, KeyCode.TOGGLE_AUTOCORRECT,
        KeyCode.TOGGLE_COMPACT_LAYOUT, KeyCode.COMPACT_LAYOUT_TO_LEFT, KeyCode.COMPACT_LAYOUT_TO_RIGHT,
        KeyCode.CHAR_WIDTH_SWITCHER, KeyCode.CHAR_WIDTH_FULL, KeyCode.CHAR_WIDTH_HALF,
        KeyCode.KANA_SWITCHER, KeyCode.KANA_HIRA, KeyCode.KANA_KATA, KeyCode.KANA_HALF_KATA,
    )

    fun route(code: Int, type: KeyType): Route = when {
        code in SEARCH_KEYS -> Route.SEARCH
        code in KEYBOARD_KEYS -> Route.KEYBOARD
        // Control characters such as tab and escape aren't search text.
        code in 1 until KeyCode.SPACE -> Route.IGNORE
        type == KeyType.CHARACTER || type == KeyType.NUMERIC -> Route.SEARCH
        else -> Route.IGNORE
    }

    /**
     * A delete-key swipe, [unitsX] key widths from where it started (negative is left), as
     * the same amount the app's field would have selected; null if [action] isn't a precise
     * delete. A precise select deletes nothing: there's nothing to select in the search box.
     */
    fun swipeDelete(action: SwipeAction, unitsX: Int): SwipeDelete? = when (action) {
        SwipeAction.DELETE_CHARACTERS_PRECISELY -> SwipeDelete((-unitsX - 1).coerceAtLeast(0), words = false)
        SwipeAction.DELETE_WORDS_PRECISELY -> SwipeDelete((-unitsX / 2 - 1).coerceAtLeast(0), words = true)
        SwipeAction.SELECT_CHARACTERS_PRECISELY, SwipeAction.SELECT_WORDS_PRECISELY -> SwipeDelete(0, words = false)
        else -> null
    }
}
