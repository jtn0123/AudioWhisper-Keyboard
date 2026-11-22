//! The base stylesheet: the theme shown before any loads, or when one fails to, and the defaults
//! every element falls back to. FlorisBoard built it in Kotlin as `FlorisImeThemeBaseStyle`; this
//! is that stylesheet as `SnyggStylesheet.toJson` wrote it.

/// The base stylesheet's JSON.
pub const BASE: &str = include_str!("base-style.json");
