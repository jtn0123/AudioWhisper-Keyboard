//! Whether a model's files are all there and whole: dictation's and the read-aloud voice's. The
//! keyboard's `ModelCheck` decided this in Kotlin; Kotlin now only looks at the files, hashes the
//! ones this asks for, and keeps the keys this says to remember.
//!
//! A quick check looks at names and sizes. A full one also needs each file's SHA-256, hashed once
//! and remembered by its path, size and modification time, so a file replaced later is hashed
//! again. A read that failed is never remembered.

use wire::dictation_edges::model_state;

/// Which files are missing or damaged, by index into the expected files; `MISSING` wins.
#[derive(Debug, PartialEq, Eq)]
pub struct State {
    /// A `ModelCheckState` code.
    pub code: i32,
    /// The files that make it missing or damaged, in order.
    pub files: Vec<usize>,
}

impl State {
    fn of(code: i32, files: Vec<usize>) -> State {
        State { code, files }
    }

    /// The state as `ModelCheckRules` answers it: its code, the count of its files, then them.
    pub fn row(&self) -> Vec<i32> {
        [self.code, self.files.len() as i32]
            .into_iter()
            .chain(self.files.iter().map(|&i| i as i32))
            .collect()
    }
}

/// From names and sizes: files not there at all are missing; else files of the wrong size are
/// damaged; else the model is ready as far as a quick look can tell.
pub fn quick(expected: &[i64], present: &[bool], lengths: &[i64]) -> State {
    let missing: Vec<usize> = (0..expected.len()).filter(|&i| !present[i]).collect();
    if !missing.is_empty() {
        return State::of(model_state::MISSING, missing);
    }
    let wrong: Vec<usize> = (0..expected.len())
        .filter(|&i| lengths[i] != expected[i])
        .collect();
    let code = if wrong.is_empty() {
        model_state::READY
    } else {
        model_state::DAMAGED
    };
    State::of(code, wrong)
}

/// What a file that passed is remembered by: its path, size and modification time.
pub fn key(path: &[u16], length: i64, modified: i64) -> Vec<u16> {
    let mut key = path.to_vec();
    key.extend(format!(":{length}:{modified}").encode_utf16());
    key
}

/// The files to hash: those not remembered as passed.
pub fn to_hash(known: &[bool]) -> Vec<usize> {
    (0..known.len()).filter(|&i| !known[i]).collect()
}

/// A full check's verdict, from which files were [`known`](to_hash) and the SHA-256 of each one
/// hashed (`None` where reading it failed): the state, and the files to remember as passed.
pub fn verdict(
    known: &[bool],
    expected: &[Vec<u16>],
    sums: &[Option<Vec<u16>>],
) -> (State, Vec<usize>) {
    let mut damaged = Vec::new();
    let mut passed = Vec::new();
    for i in 0..expected.len() {
        if known[i] {
            continue;
        }
        match &sums[i] {
            Some(sum) if *sum == expected[i] => passed.push(i),
            _ => damaged.push(i),
        }
    }
    let code = if damaged.is_empty() {
        model_state::READY
    } else {
        model_state::DAMAGED
    };
    (State::of(code, damaged), passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    #[test]
    fn missing_files_win_over_wrong_sizes() {
        let state = quick(&[3, 4, 5], &[true, false, true], &[1, 0, 5]);
        assert_eq!(state, State::of(model_state::MISSING, vec![1]));
        assert_eq!(state.row(), vec![model_state::MISSING, 1, 1]);
        assert_eq!(
            quick(&[3, 4], &[true, true], &[3, 5]),
            State::of(model_state::DAMAGED, vec![1])
        );
        assert_eq!(
            quick(&[3, 4], &[true, true], &[3, 4]),
            State::of(model_state::READY, vec![])
        );
    }

    #[test]
    fn keys_are_path_size_and_time() {
        assert_eq!(key(&utf16("/m/a.onnx"), 12, -3), utf16("/m/a.onnx:12:-3"));
    }

    #[test]
    fn only_unknown_files_are_hashed_and_only_passes_remembered() {
        let known = [true, false, false, false];
        assert_eq!(to_hash(&known), vec![1, 2, 3]);
        let expected = [utf16("aa"), utf16("bb"), utf16("cc"), utf16("dd")];
        let sums = [None, Some(utf16("bb")), Some(utf16("CC")), None];
        let (state, passed) = verdict(&known, &expected, &sums);
        assert_eq!(state, State::of(model_state::DAMAGED, vec![2, 3]));
        assert_eq!(passed, vec![1]);
        let (ready, _) = verdict(&[true, false], &expected[..2], &[None, Some(utf16("bb"))]);
        assert_eq!(ready.code, model_state::READY);
    }
}
