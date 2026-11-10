//! The search against LatinIME's C++: seeded queries from the oracle's generator, words typed and
//! deleted letter by letter on one session per side, predictions, offensive blocking, capital
//! letters, missing coordinates, German umlaut digraphs, the options and the weight; and hostile
//! input that must not panic (the Rust side alone). Every suggestion is compared field by field,
//! the weight by bits.

mod cases;
mod digraphs;
mod into;
mod options;
mod output;
mod robustness;
mod sessions;
mod space;
mod support;
