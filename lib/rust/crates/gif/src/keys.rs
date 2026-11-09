//! Which keys reach the GIF search field, as Kotlin's `GifSearchKeys` had it. The app's own text
//! field is still connected behind the panel, so a key that reached it would edit text the user
//! can't see: keys are let through only when known to be safe.

/// Where a key goes while the user types into the search field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// It edits the search query.
    Search,
    /// It works as usual: it changes the keyboard, not the text.
    Keyboard,
    /// It would edit the app's text (cursor keys, undo, paste, dictation), so it does nothing.
    Ignore,
}

/// FlorisBoard's key codes for space, which also bounds the control characters.
const SPACE: i32 = 32;

/// Keys that edit the query: delete, delete word, space, the CJK space and enter.
const SEARCH_KEYS: [i32; 5] = [-7, -8, SPACE, 12288, 10];

/// Keys that change the keyboard rather than the text: shift and caps lock, the layouts, the
/// panels, showing and hiding, languages and input methods, settings, the smartbar, incognito,
/// auto-correct, the compact layout, character width and kana.
const KEYBOARD_KEYS: [i32; 36] = [
    -11, -13, -201, -202, -203, -204, -205, -206, -207, -211, -212, -213, -231, -232, -225, -226,
    -227, -224, -228, -221, -222, -223, -301, -241, -244, -245, -110, -111, -112, -9701, -9702,
    -9703, -9710, -9711, -9712, -9713,
];

/// Where the key with `code` goes; `typing` says whether it is a character or numeric key.
pub fn route(code: i32, typing: bool) -> Route {
    if SEARCH_KEYS.contains(&code) {
        Route::Search
    } else if KEYBOARD_KEYS.contains(&code) {
        Route::Keyboard
    } else if (1..SPACE).contains(&code) {
        // Control characters such as tab and escape aren't search text.
        Route::Ignore
    } else if typing {
        Route::Search
    } else {
        Route::Ignore
    }
}

/// The delete key's swipe, as far as the search field is concerned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swipe {
    /// Deleting characters one key width at a time.
    DeleteCharacters,
    /// Deleting words two key widths at a time.
    DeleteWords,
    /// Selecting precisely: there is nothing to select in the search field.
    Select,
}

/// How far a delete-key swipe has reached back: `count` characters, or words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwipeDelete {
    pub count: i32,
    pub words: bool,
}

/// A delete-key `swipe`, `units_x` key widths from where it started (negative is left), as the
/// same amount the app's field would have selected. Kotlin's `Int` negation wraps.
pub fn swipe_delete(swipe: Swipe, units_x: i32) -> SwipeDelete {
    match swipe {
        Swipe::DeleteCharacters => SwipeDelete {
            count: units_x.wrapping_neg().wrapping_sub(1).max(0),
            words: false,
        },
        Swipe::DeleteWords => SwipeDelete {
            count: (units_x.wrapping_neg() / 2 - 1).max(0),
            words: true,
        },
        Swipe::Select => SwipeDelete {
            count: 0,
            words: false,
        },
    }
}

#[cfg(test)]
mod tests;
