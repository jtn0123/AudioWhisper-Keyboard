package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.GifRules
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.KeyType

/**
 * What the keyboard does while the user types into the GIF search box.
 *
 * The app's own text field is still connected behind the panel, so a key or
 * swipe that reached it would edit text the user can't see. Keys are
 * therefore let through only when known to be safe; anything else, including
 * keys added later, does nothing. Which keys those are is decided in Rust
 * (lib/rust/crates/gif/src/keys.rs).
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

    /** Where the key with [code] and [type] goes while the user types into the search field. */
    fun route(code: Int, type: KeyType): Route =
        Route.entries[GifRules.route(code, typing = type == KeyType.CHARACTER || type == KeyType.NUMERIC).ordinal]

    /**
     * A delete-key swipe, [unitsX] key widths from where it started (negative is left), as
     * the same amount the app's field would have selected; null if [action] isn't a precise
     * delete. A precise select deletes nothing: there's nothing to select in the search box.
     */
    fun swipeDelete(action: SwipeAction, unitsX: Int): SwipeDelete? {
        val swipe = when (action) {
            SwipeAction.DELETE_CHARACTERS_PRECISELY -> GifRules.Swipe.DELETE_CHARACTERS
            SwipeAction.DELETE_WORDS_PRECISELY -> GifRules.Swipe.DELETE_WORDS
            SwipeAction.SELECT_CHARACTERS_PRECISELY, SwipeAction.SELECT_WORDS_PRECISELY -> GifRules.Swipe.SELECT
            else -> return null
        }
        return SwipeDelete(GifRules.swipeDelete(swipe, unitsX), words = swipe == GifRules.Swipe.DELETE_WORDS)
    }
}
