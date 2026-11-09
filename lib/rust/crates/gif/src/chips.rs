//! The quick searches under the GIF search field, as Kotlin's `GifSearch.chips` had them.

use chars::equals_ignore_case;

use crate::utf16;

/// GIPHY's trending list, the chip shown first.
pub const TRENDING: &str = "Trending";

/// The GIFs sent before: a chip that needs no GIPHY call.
pub const RECENTS: &str = "Recents";

/// The quick searches, [`TRENDING`] first.
pub const CATEGORIES: [&str; 14] = [
    TRENDING,
    "Haha",
    "Yes",
    "No",
    "Thank you",
    "Love",
    "Wow",
    "Sad",
    "Hi",
    "Bye",
    "Congrats",
    "Dance",
    "Facepalm",
    "OK",
];

/// The chips to offer: [`RECENTS`] once a GIF was sent (`has_recents`), then [`CATEGORIES`]. The
/// word just typed (`typed_word`) goes first, unless it is one of them, ignoring case, which then
/// moves first instead ("congrats" brings Congrats forward).
pub fn chips(typed_word: Option<&[u16]>, has_recents: bool) -> Vec<Vec<u16>> {
    let recents = has_recents.then_some(RECENTS);
    let mut rest: Vec<Vec<u16>> = recents.into_iter().chain(CATEGORIES).map(utf16).collect();
    let Some(typed) = typed_word else {
        return rest;
    };
    let first = match rest.iter().position(|it| equals_ignore_case(it, typed)) {
        Some(same) => rest.remove(same),
        None => typed.to_vec(),
    };
    rest.insert(0, first);
    rest
}

#[cfg(test)]
mod tests;
