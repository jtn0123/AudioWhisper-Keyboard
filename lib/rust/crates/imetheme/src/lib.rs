//! The keyboard's theme, ported from FlorisBoard's `ThemeManager`: which theme the settings and the
//! system's night mode make active ([`choose`]), what Android loads to show it and which loaded
//! theme is shown ([`session`]), the stylesheet shown when no other is ([`base`]), and the inline
//! autofill chips' colours in it ([`inline`]) and the window's background ([`window`]).
//! [`calls`] reads and writes `wire::ime_theme`'s rows.
//!
//! Android keeps what only it can do: reading the extension archives and stylesheets Rust names,
//! parsing them for the Compose fallback, and reporting the system's night mode and clock.

pub mod base;
pub mod calls;
pub mod choose;
pub mod inline;
pub mod session;
pub mod window;

#[cfg(test)]
mod tests;

/// UTF-16 text, as Kotlin's strings are compared.
pub type Text = Vec<u16>;
