//! The keyboard's own stored data, which Rust reads, changes and writes: the clipboard history
//! with its media files' records ([`clips`]), and the user dictionary ([`words`]).
//!
//! Each store is one file in the format [`codec`] describes, replaced whole on every change by
//! [`disk::save`]: written to a temporary file, synced, renamed over the old one and the
//! directory synced, so a crash at any moment leaves the old file or the new one, never a mix.
//! [`store::Store`] holds a store's value and refuses to overwrite a file it could not read.
//! The Room databases the keyboard kept before are read once by Kotlin and handed over through
//! [`legacy`], which imports them exactly once and deletes them after.
#![forbid(unsafe_code)]

pub mod clip_codec;
pub mod clipboard_policy;
pub mod clips;
pub mod codec;
pub mod disk;
pub mod legacy;
pub mod store;
pub mod word_codec;
pub mod words;

#[cfg(test)]
mod tests;
