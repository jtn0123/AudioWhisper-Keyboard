//! Test-only: what LatinIME's C++ answered, kept as golden files, so the Rust port in `latin`
//! (and the session `suggest` builds on it) is held to the C++ without building it.
//!
//! A test records each of its answers on a [`Tape`], in order; the tape checks them against the
//! test's golden file in `tests/golden/`. The files were written by the same tests while they
//! still compared every answer with the C++ (the `latin-oracle` crate, since deleted): see
//! docs/phase6-oracle-goldens.md for the commit that can write them again.
//!
//! [`queries`] is the seeded question generator those tests drew from, unchanged, so the
//! questions are the ones the C++ answered.

pub mod answer;
mod math;
pub mod queries;
pub mod qwerty;
pub mod tape;

pub use answer::{Answer, Fnv, Suggestions};
pub use tape::Tape;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Each dictionary's fingerprint (its bytes hashed as an [`Answer`]) when its golden files were
/// written, in the order of [`dictionaries`]. A dictionary that has changed since cannot be held
/// to the answers the C++ gave on the old one.
const FINGERPRINTS: [u64; 2] = [0xEE25_32F8_6E66_A00C, 0xF473_EA4E_015E_BB65];

/// The dictionary the app ships.
pub fn shipped_dictionary_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../app/src/main/assets/dicts/main_en-US.dict")
}

/// Google's AOSP dictionary, which the app shipped until the corpus rebuild (docs/decisions.md,
/// "Word statistics from a public corpus"), kept as a test fixture.
pub fn aosp_dictionary_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../latin/tests/fixtures/aosp_main_en-US.dict")
}

/// Both dictionaries the C++ answered on, each with the name its golden files carry. The first
/// call checks neither has changed since the golden files were written.
pub fn dictionaries() -> [(&'static str, PathBuf); 2] {
    static CHECKED: OnceLock<()> = OnceLock::new();
    let dictionaries = [
        ("aosp", aosp_dictionary_path()),
        ("shipped", shipped_dictionary_path()),
    ];
    CHECKED.get_or_init(|| {
        for ((name, path), expected) in dictionaries.iter().zip(FINGERPRINTS) {
            check_fingerprint(name, path, expected);
        }
    });
    dictionaries
}

/// Fails, saying what to do, unless the dictionary at `path` has the fingerprint `expected`.
fn check_fingerprint(name: &str, path: &Path, expected: u64) {
    let bytes = std::fs::read(path).expect("the dictionary");
    let mut fnv = Fnv::default();
    bytes.as_slice().feed(&mut fnv);
    let actual = fnv.finish();
    assert!(
        actual == expected,
        "the {name} dictionary {} has changed since its golden files were written (fingerprint \
         {actual:016x}, not {expected:016x}): write the *.{name}.txt goldens again as \
         docs/phase6-oracle-goldens.md says",
        path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_dictionaries_are_the_ones_the_goldens_were_written_on() {
        let [(aosp, _), (shipped, _)] = dictionaries();
        assert_eq!((aosp, shipped), ("aosp", "shipped"));
    }

    #[test]
    #[should_panic(expected = "has changed since its golden files were written")]
    fn a_changed_dictionary_is_named_with_what_to_do() {
        check_fingerprint("aosp", &aosp_dictionary_path(), FINGERPRINTS[1]);
    }
}
