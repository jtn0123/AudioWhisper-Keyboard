//! The keyboard's subtypes, ported from FlorisBoard's `SubtypeManager`: the stored list decoded
//! and repaired, the active subtype, the language key's next and previous, a switch by id, and
//! what each subtype names (its layouts, popup mapping, currency set, composer, NLP providers and
//! the key graph's cache key), in [`session`]. [`calls`] reads and writes `wire::ime_subtype`'s
//! rows.
//!
//! The rules themselves (the repair, the active choice, the cycle) are `settings::subtype`'s, which
//! the settings screens' edits use too. Android keeps what only it can do: reading and writing the
//! settings, naming a locale tag as `java.util.Locale` does, and loading the currency sets.

pub mod calls;
pub mod session;

#[cfg(test)]
mod tests;

/// UTF-16 text, as Kotlin's strings are compared.
pub type Text = Vec<u16>;
