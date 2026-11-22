//! The search against LatinIME's C++ answers, kept as golden files (`crate::golden`): seeded
//! queries, words typed and deleted letter by letter on one session, predictions, offensive
//! blocking, capital letters, missing coordinates, German umlaut digraphs, the options and the
//! weight; and hostile input that must not panic. Every suggestion is recorded field by field,
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
