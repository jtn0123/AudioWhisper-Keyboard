//! The keyboard's key graph: what FlorisBoard's `TextKeyboard` held. Layout and popup-mapping
//! files are read ([`parse`]) and merged into rows of keys ([`Graph::build`]); each key is then
//! computed for the keyboard's state and the facts Android reports ([`Graph::compute`]): its
//! data, whether it shows and is enabled, its flex, popup keys, label, hint and icon; and laid
//! out at a size ([`Graph::layout`]).
//!
//! Android keeps only what it alone knows: it reads the files, reports the facts
//! ([`facts::Facts`]) and answers the questions of locale casing and ICU's mark predicate
//! ([`facts::Question`]), which a computation hands back the first time it needs them.

#![forbid(unsafe_code)]

mod assemble;
pub mod calls;
pub mod compute;
mod context;
pub mod data;
pub mod facts;
mod graph;
pub mod node;
pub mod parse;

pub use graph::{Action, Files, Graph, Key, Request};

#[cfg(test)]
mod tests;
