package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.gif.GifSearchKeys.Route
import com.jtn0123.audiowhisper.gif.GifSearchKeys.SwipeDelete
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.gestures.keyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class GifSearchKeysTest {
    private fun route(key: TextKeyData) = GifSearchKeys.route(key.code, key.type)

    /**
     * What each swipe action's key does while the search box is open. The `when` has no
     * else, so a new SwipeAction can't compile until someone decides where it goes.
     */
    private fun expected(action: SwipeAction): Route? = when (action) {
        // The touch code carries these out itself; see the swipeDelete tests.
        SwipeAction.NO_ACTION, SwipeAction.DELETE_CHARACTER,
        SwipeAction.DELETE_CHARACTERS_PRECISELY, SwipeAction.DELETE_WORDS_PRECISELY,
        SwipeAction.SELECT_CHARACTERS_PRECISELY, SwipeAction.SELECT_WORDS_PRECISELY -> null
        SwipeAction.DELETE_WORD, SwipeAction.INSERT_SPACE -> Route.SEARCH
        SwipeAction.MOVE_CURSOR_UP, SwipeAction.MOVE_CURSOR_DOWN,
        SwipeAction.MOVE_CURSOR_LEFT, SwipeAction.MOVE_CURSOR_RIGHT,
        SwipeAction.MOVE_CURSOR_START_OF_LINE, SwipeAction.MOVE_CURSOR_END_OF_LINE,
        SwipeAction.MOVE_CURSOR_START_OF_PAGE, SwipeAction.MOVE_CURSOR_END_OF_PAGE,
        SwipeAction.UNDO, SwipeAction.REDO -> Route.IGNORE
        SwipeAction.CYCLE_TO_PREVIOUS_KEYBOARD_MODE, SwipeAction.CYCLE_TO_NEXT_KEYBOARD_MODE,
        SwipeAction.HIDE_KEYBOARD, SwipeAction.SHIFT,
        SwipeAction.SHOW_INPUT_METHOD_PICKER, SwipeAction.SHOW_SUBTYPE_PICKER,
        SwipeAction.SWITCH_TO_PREV_SUBTYPE, SwipeAction.SWITCH_TO_NEXT_SUBTYPE,
        SwipeAction.SWITCH_TO_CLIPBOARD_CONTEXT, SwipeAction.SWITCH_TO_MEDIA_CONTEXT,
        SwipeAction.SWITCH_TO_PREV_KEYBOARD, SwipeAction.TOGGLE_SMARTBAR_VISIBILITY -> Route.KEYBOARD
    }

    @Test
    fun `no swipe action reaches the app's text while searching`() {
        for (mode in KeyboardMode.entries) {
            for (action in SwipeAction.entries) {
                assertEquals(expected(action), action.keyData(mode)?.let(::route), "$action in $mode")
            }
        }
    }

    @Test
    fun `typing keys edit the search`() {
        assertEquals(Route.SEARCH, GifSearchKeys.route('a'.code, KeyType.CHARACTER))
        assertEquals(Route.SEARCH, GifSearchKeys.route('7'.code, KeyType.NUMERIC))
        assertEquals(Route.SEARCH, GifSearchKeys.route(KeyCode.MULTIPLE_CODE_POINTS, KeyType.CHARACTER))
        for (key in listOf(TextKeyData.DELETE, TextKeyData.DELETE_WORD, TextKeyData.SPACE)) {
            assertEquals(Route.SEARCH, route(key), key.label)
        }
        assertEquals(Route.SEARCH, GifSearchKeys.route(KeyCode.ENTER, KeyType.ENTER_EDITING), "enter searches")
    }

    @Test
    fun `keys that would edit the app do nothing`() {
        val appKeys = listOf(
            TextKeyData.ARROW_LEFT, TextKeyData.MOVE_END_OF_PAGE, TextKeyData.UNDO, TextKeyData.REDO,
            TextKeyData.CLIPBOARD_PASTE, TextKeyData.CLIPBOARD_CUT, TextKeyData.CLIPBOARD_SELECT_ALL,
            TextKeyData.FORWARD_DELETE, TextKeyData.VOICE_INPUT, TextKeyData.TOGGLE_ACTIONS_OVERFLOW,
        )
        for (key in appKeys) assertEquals(Route.IGNORE, route(key), key.label)
        assertEquals(Route.IGNORE, GifSearchKeys.route(KeyCode.TAB, KeyType.CHARACTER))
        assertEquals(Route.IGNORE, GifSearchKeys.route(-12345, KeyType.FUNCTION), "an unknown key")
    }

    @Test
    fun `keyboard keys still work`() {
        for (key in listOf(TextKeyData.SHIFT, TextKeyData.VIEW_SYMBOLS, TextKeyData.IME_UI_MODE_MEDIA, TextKeyData.IME_HIDE_UI)) {
            assertEquals(Route.KEYBOARD, route(key), key.label)
        }
    }

    @Test
    fun `a delete swipe takes back as much as the app's field would select`() {
        val chars = SwipeAction.DELETE_CHARACTERS_PRECISELY
        assertEquals(SwipeDelete(0, words = false), GifSearchKeys.swipeDelete(chars, -1))
        assertEquals(SwipeDelete(3, words = false), GifSearchKeys.swipeDelete(chars, -4))
        assertEquals(SwipeDelete(0, words = false), GifSearchKeys.swipeDelete(chars, 2), "past where it began")
        assertEquals(SwipeDelete(1, words = true), GifSearchKeys.swipeDelete(SwipeAction.DELETE_WORDS_PRECISELY, -5))
        assertEquals(SwipeDelete(0, words = false), GifSearchKeys.swipeDelete(SwipeAction.SELECT_CHARACTERS_PRECISELY, -6))
        assertNull(GifSearchKeys.swipeDelete(SwipeAction.DELETE_WORD, -6), "not a precise delete")
    }
}
