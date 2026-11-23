//! The GIF panel's rules (the `gif` crate) for `GifNative`, whose generated entry points call
//! these or the crate directly.

use gif::keys::{self, Route, Swipe};
use gif::problem::{self, Failure};
use gif::{chips, search_text};

use crate::jni::{Result, illegal};

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// The longest search, in UTF-16 units (`search_text::MAX_LENGTH`).
pub fn max_length() -> i32 {
    search_text::MAX_LENGTH as i32
}

/// The quick searches, trending first (`chips::CATEGORIES`).
pub fn categories() -> Vec<Vec<u16>> {
    chips::CATEGORIES.map(utf16).to_vec()
}

/// The chip for the GIFs sent before (`chips::RECENTS`).
pub fn recents() -> Vec<u16> {
    utf16(chips::RECENTS)
}

/// `query` without its last `count` characters (`search_text::delete_last`).
pub fn delete_last(query: &[u16], count: i32) -> Vec<u16> {
    search_text::delete_last(query, count).to_vec()
}

/// `query` without its last `count` words (`search_text::delete_words`).
pub fn delete_words(query: &[u16], count: i32) -> Vec<u16> {
    search_text::delete_words(query, count).to_vec()
}

/// The word just typed at the end of `text`, or null (`search_text::last_word`).
pub fn last_word(text: Option<&[u16]>) -> Option<Vec<u16>> {
    text.and_then(search_text::last_word).map(<[u16]>::to_vec)
}

/// Where a key goes while typing into the search (`keys::route`), as `GifRules.Route` orders it.
pub fn route(code: i32, typing: bool) -> i32 {
    match keys::route(code, typing) {
        Route::Search => 0,
        Route::Keyboard => 1,
        Route::Ignore => 2,
    }
}

/// How far a delete-key swipe reaches back (`keys::swipe_delete`): `swipe` is 0 to delete
/// characters, 1 words and 2 to select; the count comes back.
pub fn swipe_delete(swipe: i32, units_x: i32) -> Result<i32> {
    let swipe = match swipe {
        0 => Swipe::DeleteCharacters,
        1 => Swipe::DeleteWords,
        2 => Swipe::Select,
        _ => return Err(illegal(format!("no swipe {swipe}"))),
    };
    Ok(keys::swipe_delete(swipe, units_x).count)
}

/// What a failed GIPHY call tells the user (`problem::problem`): `failure` is 0 for an
/// unreadable answer, 1 for an HTTP `status` and 2 for no answer.
pub fn problem(failure: i32, status: i32) -> Result<Vec<u16>> {
    let failure = match failure {
        0 => Failure::Unreadable,
        1 => Failure::Status(status),
        2 => Failure::Unreachable,
        _ => return Err(illegal(format!("no failure {failure}"))),
    };
    Ok(utf16(problem::problem(failure)))
}
