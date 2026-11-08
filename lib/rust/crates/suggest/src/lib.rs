//! The keyboard's typing suggestions, moved from the app's Kotlin `suggest` package piece by
//! piece; each module names the Kotlin it replaces. The LatinIME decoder they rank the words of
//! stays C++ behind Kotlin, which passes its results in.
//!
//! Text is UTF-16 (`&[u16]`), as Kotlin's `String` is, and is judged as Kotlin judges it, through
//! `chars`. Code points are `i32` where Kotlin's are an `Int` that may be anything (a key's code),
//! and a Kotlin `Float` is an `f32` computed in the same order.

#![forbid(unsafe_code)]

pub mod auto_correction;
pub mod glide_ranker;
pub mod java;
pub mod kept_words;
pub mod learned_words;
mod lines;
mod ordered;
pub mod personal_words;
pub mod previous_words;
pub mod qwerty;
pub mod raw;
pub mod swipe_words;
pub mod tap;
pub mod undo;
pub mod word_case;
pub mod word_characters;
