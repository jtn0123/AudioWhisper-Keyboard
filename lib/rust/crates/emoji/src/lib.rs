//! Emoji data and suggestions, as FlorisBoard's `EmojiData`, `Emoji` and
//! `EmojiSuggestionProvider` had them:
//!
//! - an emoji file (`assets/ime/media/emoji/*.txt`) parsed into its emoji, the sets of each one's
//!   skin-tone variants by category, and the emoji to show for each skin tone ([`data`]);
//! - which skin tone and hair style an emoji has ([`tones`]);
//! - which file a locale reads ([`asset`]);
//! - what typed text asks for emoji (`:smi`), and the emoji whose names and keywords it matches,
//!   best first ([`suggest`]).
//!
//! Kotlin still reads the files and the settings and builds its `Emoji` objects; Rust is handed a
//! file's text once and answers with where each piece of it is, and a query with which emoji.
//!
//! Text is UTF-16 units (`&[u16]`), as a Kotlin `String` is, judged by Kotlin's character rules
//! (the `chars` crate).

#![forbid(unsafe_code)]

pub mod asset;
pub mod data;
pub mod suggest;
pub mod tones;

#[cfg(test)]
pub(crate) fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

pub mod history;
