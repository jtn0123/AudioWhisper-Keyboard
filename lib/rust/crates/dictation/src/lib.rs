//! Dictation's text rules: finding spoken commands in a transcribed phrase ([`parse`]), joining
//! phrases transcribed separately ([`join`], [`punctuate`]), and whether a backspace can still
//! take back the phrase typed last ([`reverts_at`]).
//!
//! A port of the keyboard's `Spoken`, `Phrases` and `DictationUndo` that gives exactly the same
//! text; the keyboard's tests compare the two (see docs/rust.md). Text is UTF-16, as Kotlin
//! stores it, and characters are judged by Kotlin's rules (the `chars` crate).

#![forbid(unsafe_code)]

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
