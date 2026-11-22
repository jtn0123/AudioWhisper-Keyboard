//! The golden tapes the tests record their answers on (`latin-golden`), and the comparison with
//! LatinIME's C++ the goldens were written from, when it is built (`--features oracle`).

pub use latin_golden::Tape;

/// The tape for `tests/golden/{name}.txt`.
pub fn tape(name: &str) -> Tape {
    Tape::open(env!("CARGO_MANIFEST_DIR"), name)
}

/// Records the answer `ours` on `tape`. With the C++ built, first asserts it is `theirs`, the
/// C++'s answer to the same question, with an optional message.
macro_rules! same {
    ($tape:expr, $ours:expr, $theirs:expr $(, $($message:tt)+)?) => {{
        let ours = $ours;
        #[cfg(feature = "oracle")]
        assert_eq!(ours, $theirs $(, $($message)+)?);
        $tape.record(stringify!($ours), &ours);
    }};
}
pub(crate) use same;

/// A generated question as the C++ oracle takes it.
#[cfg(feature = "oracle")]
pub fn oracle_query(query: &latin_golden::queries::Query) -> latin_oracle::Query {
    latin_oracle::Query {
        code_points: query.code_points.clone(),
        points: query.points.clone(),
        block_offensive_words: query.block_offensive_words,
        previous: query.previous.clone(),
        beginning_of_sentence: query.beginning_of_sentence.clone(),
    }
}
