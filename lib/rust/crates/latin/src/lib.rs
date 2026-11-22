//! The typing decoder, ported from LatinIME's C++ (`lib/latinime/src/main/jni`): the bundled
//! dictionary read from its bytes, and the search that turns taps and the words before them into
//! suggestions.
//!
//! Each module is one area of the C++ and names the files it ports. The port keeps the C++'s
//! types (`i32` code points and positions, `f32` scores) and order of operations, so its answers
//! are the same to the bit; every module is tested against the C++'s own answers, kept as golden
//! files in `tests/golden` (see `docs/phase6-oracle-goldens.md`).
//!
//! - [`defines`]: the shared constants and enums (`defines.h`).
//! - [`chars`]: lower case and base letters (`utils/char_utils`).
//! - [`bytes`]: integers, code points and the format version (`dictionary/utils/byte_array_utils`,
//!   `format_utils`).
//! - [`header`]: the dictionary's header and its attributes (`dictionary/header`).
//! - [`trie`]: the version 2 trie: words, probabilities, word pairs and the word list
//!   (`dictionary/structure`, the rest of `dictionary/utils`, `dictionary/property`).
//! - [`layout`]: the keyboard and the taps on it (`suggest/core/layout`).
//! - [`search`]: the search over the trie and its scoring (`suggest/core`, `suggest/policyimpl`).

#![forbid(unsafe_code)]

pub mod bytes;
pub mod chars;
pub mod defines;
#[cfg(test)]
mod golden;
pub mod header;
pub mod layout;
pub mod search;
pub mod trie;
