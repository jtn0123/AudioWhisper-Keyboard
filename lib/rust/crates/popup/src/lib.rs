//! A key's long-press popup, as upstream FlorisBoard's Kotlin had it in `ime/popup`: which keys
//! it holds and in what order for the hint settings ([`order`], from `PopupSet`), and where the
//! extended popup's grid sits and which of its keys a moving finger is on ([`grid`], from
//! `PopupUiController`).
//!
//! Kotlin keeps the keys themselves; Rust is given only how many there are and answers with
//! indices. Floats are `f32`, as Kotlin's `Float`, and every operation happens in Kotlin's order,
//! so the bounds come out the same to the bit; `Int` arithmetic wraps as Kotlin's does.

#![forbid(unsafe_code)]

pub mod grid;
pub mod order;
