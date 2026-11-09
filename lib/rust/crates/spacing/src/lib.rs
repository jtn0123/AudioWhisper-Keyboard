//! The spaces the keyboard types by itself, as FlorisBoard's `EditorInstance` decided them:
//!
//! - a **phantom space**, owed after a word picked from the suggestions or swiped, and typed only
//!   if a letter, a digit or an opening mark comes next ([`spaces::phantom_space_before`]);
//! - **auto-spacing** after punctuation (a setting, off by default): a space typed after a full
//!   stop or a comma, and taken back if the next thing typed is punctuation too
//!   ([`spaces::commit_char`]);
//! - the two flags' small state machines ([`state`]), and the punctuation rule that lists which
//!   symbols go on either side of each kind of space ([`rule`]).
//!
//! Kotlin still looks the text up (the characters before the cursor, cut by ICU's grapheme
//! breaks, and the current word), and only when a rule gets far enough to need it: a rule that
//! needs text it wasn't given answers with what it needs ([`spaces::Needs`]), and Kotlin asks
//! again with it.
//!
//! Text is UTF-16 units (`&[u16]`), as a Kotlin `String` is, judged by Kotlin's character rules
//! (the `chars` crate).

#![forbid(unsafe_code)]

pub mod rule;
pub mod spaces;
pub mod state;

/// Whether `units` are exactly the ASCII `text`.
pub(crate) fn is_ascii(units: &[u16], text: &str) -> bool {
    units.iter().copied().eq(text.bytes().map(u16::from))
}
