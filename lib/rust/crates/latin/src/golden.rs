//! The golden tapes the tests record their answers on (`latin-golden`): LatinIME's C++ answers
//! to the same questions, written when the C++ was still here (docs/phase6-oracle-goldens.md).

pub use latin_golden::Tape;

/// The tape for `tests/golden/{name}.txt`.
pub fn tape(name: &str) -> Tape {
    Tape::open(env!("CARGO_MANIFEST_DIR"), name)
}

/// Records `answer` on `tape`, labelled with its expression, which checks it against the golden.
macro_rules! record {
    ($tape:expr, $answer:expr) => {{
        let answer = $answer;
        $tape.record(stringify!($answer), &answer);
    }};
}
pub(crate) use record;
