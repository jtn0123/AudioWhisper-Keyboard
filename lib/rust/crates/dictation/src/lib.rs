//! Dictation's text rules: finding spoken commands in a transcribed phrase ([`parse`]), joining
//! phrases transcribed separately ([`join`], [`punctuate`]), and whether a backspace can still
//! take back the phrase typed last ([`reverts_at`]), and what a press of the mic key does
//! ([`mic_key`]).
//!
//! A port of the keyboard's `Spoken`, `Phrases`, `DictationUndo` and `MicKey` that gives exactly the same
//! text; the keyboard's tests compare the two (see docs/rust.md). Text is UTF-16, as Kotlin
//! stores it, and characters are judged by Kotlin's rules (the `chars` crate).

#![forbid(unsafe_code)]

pub mod audio;
pub mod mic_key;
pub mod model_check;
mod phrases;
mod spoken;
mod undo;

pub use phrases::{Typing, join, punctuate};
pub use spoken::{Spoken, parse};
pub use undo::reverts_at;

/// A UTF-16 unit from an ASCII character.
const fn unit(ascii: char) -> u16 {
    ascii as u16
}

#[cfg(test)]
fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

pub mod session;

pub mod segmentation;

mod compose;

pub fn normalize_result(text: &[u16]) -> Vec<u16> {
    chars::trim(text).to_vec()
}
