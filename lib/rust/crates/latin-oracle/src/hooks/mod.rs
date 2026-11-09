//! Hooks below the JNI methods: each module calls into one area of the C++ (through
//! `cpp/hooks/<area>.cpp`) so that area of the Rust port can be compared with it on its own,
//! before the whole decoder is.

pub mod bytes;
pub mod chars;
pub mod header;
pub mod layout;
pub mod search;
pub mod trie;
