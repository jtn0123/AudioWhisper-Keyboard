//! One test's answers, in order, checked against its golden file as they come.
//!
//! A golden file is plain text: the answer count, the digest after the last answer, the digest
//! after each power-of-two count of answers (`at`), and a few answers in full (`sample`), each
//! with its hash and the expression that made it. The digest chains the answers' FNV-1a hashes
//! ([`Fnv`]), so it pins every answer and their order; the checkpoints say how far a different
//! run got before it differed, and the samples what an answer looked like.
//!
//! The files hold LatinIME's C++ answers, so this tree only checks them. They are written from
//! the generator commit, where every answer was compared with the C++ as it was written
//! (docs/phase6-oracle-goldens.md); `LATIN_GOLDEN=write` here fails rather than record the
//! port's own answers under LatinIME's name.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::answer::{Answer, Fnv};

/// The longest sample text a golden file keeps.
const SAMPLE_TEXT: usize = 120;
/// The longest expression a sample is labelled with.
const SAMPLE_LABEL: usize = 80;

/// What a golden file holds.
#[derive(Debug, Default)]
struct Golden {
    count: u64,
    digest: u64,
    checkpoints: BTreeMap<u64, u64>,
    /// Hash and line, by answer index.
    samples: BTreeMap<u64, (u64, String)>,
}

/// A test's run of answers. Every answer goes through [`Tape::record`]; [`Tape::finish`] ends it.
#[derive(Debug)]
pub struct Tape {
    name: String,
    path: PathBuf,
    /// The golden file's contents when checking, `None` when writing.
    golden: Option<Golden>,
    count: u64,
    digest: Fnv,
    written: Golden,
    finished: bool,
}

/// Whether the answer at `index` is kept in full: the first four, and the last before each
/// checkpoint.
fn sampled(index: u64) -> bool {
    index < 4 || (index + 1).is_power_of_two()
}

impl Tape {
    /// The tape for `tests/golden/{name}.txt` under `directory` (a crate's directory).
    pub fn open(directory: &str, name: &str) -> Self {
        let path = Path::new(directory)
            .join("tests/golden")
            .join(format!("{name}.txt"));
        assert!(
            std::env::var_os("LATIN_GOLDEN").is_none(),
            "the golden files hold LatinIME's C++ answers: write them from the generator commit \
             8549be045 (docs/phase6-oracle-goldens.md), not from the port"
        );
        Self::at(path, name, false)
    }

    /// The tape for the golden file at `path`, which it checks, or writes (only the tape's own
    /// tests write here; see [`Tape::open`]).
    fn at(path: PathBuf, name: &str, writing: bool) -> Self {
        let golden = (!writing).then(|| {
            let text = std::fs::read_to_string(&path);
            assert!(
                text.is_ok(),
                "no golden file {}: see docs/phase6-oracle-goldens.md",
                path.display()
            );
            parse(&text.unwrap_or_default())
        });
        Self {
            name: name.to_owned(),
            path,
            golden,
            count: 0,
            digest: Fnv::default(),
            written: Golden::default(),
            finished: false,
        }
    }

    /// Records the next answer, made by the expression `label`, and checks it as far as the
    /// golden file can say yet.
    pub fn record<A: Answer + ?Sized>(&mut self, label: &str, answer: &A) {
        let mut fnv = Fnv::default();
        answer.feed(&mut fnv);
        let hash = fnv.finish();
        self.digest.word(hash);
        let index = self.count;
        self.count += 1;
        if let Some(golden) = &self.golden {
            assert!(
                index < golden.count,
                "{}: more answers than the golden {}",
                self.name,
                golden.count
            );
        }
        if sampled(index) {
            self.sample(index, hash, label, answer);
        }
        if self.count.is_power_of_two() {
            self.checkpoint();
        }
    }

    fn sample<A: Answer + ?Sized>(&mut self, index: u64, hash: u64, label: &str, answer: &A) {
        let mut text = String::new();
        answer.render(&mut text);
        if let Some((cut, _)) = text.char_indices().nth(SAMPLE_TEXT) {
            text.truncate(cut);
            text.push_str("...");
        }
        let label: String = label.split_whitespace().collect::<Vec<_>>().join(" ");
        let label: String = label.chars().take(SAMPLE_LABEL).collect();
        let text: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let line = format!("{text} <- {label}");
        match &self.golden {
            Some(golden) => {
                let (expected, golden_line) = &golden.samples[&index];
                assert!(
                    *expected == hash,
                    "{}: answer {index} is {line}; LatinIME's was {golden_line}",
                    self.name
                );
            }
            None => {
                self.written.samples.insert(index, (hash, line));
            }
        }
    }

    fn checkpoint(&mut self) {
        let (count, digest) = (self.count, self.digest.finish());
        match &self.golden {
            Some(golden) => {
                let first = count / 2;
                assert!(
                    golden.checkpoints.get(&count) == Some(&digest),
                    "{}: answers {first} to {} differ from LatinIME's in {}",
                    self.name,
                    count - 1,
                    self.path.display()
                );
            }
            None => {
                self.written.checkpoints.insert(count, digest);
            }
        }
    }

    /// How many answers have been recorded, which is the index of the next.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Ends the run: checks the answer count and the final digest, or writes the golden file.
    pub fn finish(mut self) {
        self.finished = true;
        let digest = self.digest.finish();
        let Some(golden) = &self.golden else {
            self.written.count = self.count;
            self.written.digest = digest;
            std::fs::create_dir_all(self.path.parent().expect("a directory"))
                .expect("the golden directory");
            std::fs::write(&self.path, render(&self.name, &self.written)).expect("a golden file");
            return;
        };
        assert!(
            self.count == golden.count,
            "{}: {} answers, the golden has {}",
            self.name,
            self.count,
            golden.count
        );
        assert!(
            digest == golden.digest,
            "{}: the answers differ from LatinIME's in {}",
            self.name,
            self.path.display()
        );
    }
}

impl Drop for Tape {
    fn drop(&mut self) {
        assert!(
            self.finished || std::thread::panicking(),
            "{}: the tape was never finished",
            self.name
        );
    }
}

fn render(name: &str, golden: &Golden) -> String {
    let mut text = format!(
        "# {name}: LatinIME's C++ answers to this test's questions (docs/phase6-oracle-goldens.md).\n\
         # Answers, the digest after the last, after each power of two (at N), and some in full.\n\
         count {}\ndigest {:016x}\n",
        golden.count, golden.digest
    );
    for (count, digest) in &golden.checkpoints {
        let _ = writeln!(text, "at {count} {digest:016x}");
    }
    for (index, (hash, line)) in &golden.samples {
        let _ = writeln!(text, "sample {index} {hash:016x} {line}");
    }
    text
}

fn parse(text: &str) -> Golden {
    let mut golden = Golden::default();
    let hex =
        |field: Option<&str>| u64::from_str_radix(field.expect("a hex field"), 16).expect("hex");
    let number = |field: Option<&str>| field.expect("a number").parse::<u64>().expect("a number");
    for line in text.lines().filter(|it| !it.starts_with('#')) {
        let mut fields = line.splitn(4, ' ');
        match fields.next() {
            Some("count") => golden.count = number(fields.next()),
            Some("digest") => golden.digest = hex(fields.next()),
            Some("at") => {
                let count = number(fields.next());
                golden.checkpoints.insert(count, hex(fields.next()));
            }
            Some("sample") => {
                let index = number(fields.next());
                let hash = hex(fields.next());
                golden
                    .samples
                    .insert(index, (hash, fields.next().unwrap_or_default().to_owned()));
            }
            other => panic!("a golden line starts with count, digest, at or sample, not {other:?}"),
        }
    }
    golden
}

#[cfg(test)]
mod tests;
