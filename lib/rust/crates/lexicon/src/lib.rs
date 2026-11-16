//! Host-only tools that build the keyboard's word statistics from a public corpus. Nothing here
//! ships in the app; `scripts/build-language-model.sh` runs them, and docs/rust.md ("The typing
//! scoreboard") explains the whole pipeline.

#![forbid(unsafe_code)]

pub mod corpus;
pub mod count;
pub mod dictionary;
pub mod text;
pub mod v202;
