//! The GIF panel's rules that don't need Android, as the keyboard's Kotlin had them in its `gif`
//! package: editing the search field ([`search_text`]), the quick searches under it ([`chips`]),
//! which keys reach it ([`keys`]), which searches the result cache keeps ([`cache`]), the order
//! of the GIFs sent before ([`recents`]), what a failed GIPHY call tells the user
//! ([`problem`]), and GIPHY's URLs and answers ([`giphy`]). The panel's session ([`session`])
//! decides every step from them, with the requests and replies of [`reply`], the previews kept
//! in memory ([`previews`]) and the steps of sending a GIF ([`send`]), crossing as [`rows`].
//!
//! Text is UTF-16 units (`&[u16]`), as a Kotlin `String` is, judged by Kotlin's character rules
//! (the `chars` crate).

#![forbid(unsafe_code)]

pub mod cache;
pub mod chips;
pub mod giphy;
pub mod java_float;
pub mod json;
pub mod keys;
pub mod previews;
pub mod problem;
pub mod recents;
pub mod reply;
pub mod rows;
pub mod search_text;
pub mod send;
pub mod session;

/// `text` as UTF-16 units.
pub(crate) fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
