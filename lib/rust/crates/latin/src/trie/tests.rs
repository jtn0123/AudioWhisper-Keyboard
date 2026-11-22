//! Differential tests of the version 2 trie against LatinIME's C++, kept as golden files of its
//! answers (`crate::golden`). Each test asks the same question of each dictionary (Google's AOSP
//! one and the one the app ships) and checks the answers against the C++'s. Random
//! inputs come from seeded SplitMix64 streams, so a failure reproduces from its seed. Assertion
//! messages carry indices and numbers only: a dictionary word is never printed.

mod context;
mod nodes;
mod robust;
mod units;

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

#[cfg(feature = "oracle")]
use latin_oracle::hooks::trie as cpp;

use super::dictionary::Dictionary;
use super::interface::{NgramListener, WordListener};
use super::jni::get_words;
use crate::defines::{MAX_WORD_LENGTH, NOT_A_WORD_ID};
use crate::golden::{Tape, same, tape};

/// A dictionary, opened by the Rust (and by the C++ when it is built), with its words.
pub struct Pair {
    pub ours: Dictionary,
    #[cfg(feature = "oracle")]
    pub theirs: latin_oracle::Dictionary,
    pub words: Vec<Vec<i32>>,
    pub path: PathBuf,
    /// Which dictionary, for the golden files' names.
    pub name: &'static str,
}

impl Pair {
    /// The tape for `test` on this dictionary.
    pub fn tape(&self, test: &str) -> Tape {
        tape(&format!("trie.{test}.{}", self.name))
    }
}

fn open((name, path): (&'static str, PathBuf)) -> Pair {
    let file: Arc<[u8]> = std::fs::read(&path).expect("the dictionary").into();
    let ours = Dictionary::open(file).expect("a version 2 dictionary");
    Pair {
        words: words_of(&ours),
        ours,
        #[cfg(feature = "oracle")]
        theirs: latin_oracle::Dictionary::at(&path),
        path,
        name,
    }
}

/// Google's AOSP dictionary and the one the app ships, each loaded once for the whole test
/// binary.
pub fn pairs() -> [&'static Pair; 2] {
    static PAIRS: OnceLock<[Pair; 2]> = OnceLock::new();
    let [aosp, shipped] = PAIRS.get_or_init(|| latin_golden::dictionaries().map(open));
    [aosp, shipped]
}

/// Every word of `dictionary`, in the order a walk visits them.
fn words_of(dictionary: &Dictionary) -> Vec<Vec<i32>> {
    let mut collected = Collect::default();
    dictionary.iterate_words(&mut collected);
    collected.words
}

/// Collects the bigram entries a walk visits.
#[derive(Default)]
struct Entries(Vec<(i32, i32)>);

impl NgramListener for Entries {
    fn on_visit_entry(&mut self, ngram_probability: i32, target_word_id: i32) {
        self.0.push((ngram_probability, target_word_id));
    }
}

/// The bigram entries after `prev`, as (probability, target word id).
pub fn ngram_entries(pair: &Pair, prev: &[i32]) -> Vec<(i32, i32)> {
    let mut entries = Entries::default();
    pair.ours.policy().iterate_ngram_entries(prev, &mut entries);
    entries.0
}

/// The code points of `word_id`, up to `MAX_WORD_LENGTH` of them.
pub fn code_points(pair: &Pair, word_id: i32) -> Vec<i32> {
    let mut out = [0_i32; MAX_WORD_LENGTH];
    let count = pair
        .ours
        .policy()
        .get_code_points_and_return_code_point_count(word_id, MAX_WORD_LENGTH, &mut out);
    out[..count].to_vec()
}

/// SplitMix64, as `bytes/tests.rs` uses it.
pub struct Random(pub u64);

impl Random {
    /// The next 64 random bits.
    pub fn draw(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number below `bound`.
    pub fn below(&mut self, bound: usize) -> usize {
        (self.draw() % bound as u64) as usize
    }

    /// True one time in `one_in`.
    pub fn chance(&mut self, one_in: usize) -> bool {
        self.below(one_in) == 0
    }

    /// A random code point, mostly ASCII letters, with some Latin-1 and the odd code point.
    fn letter(&mut self) -> i32 {
        match self.below(20) {
            0 => i32::from(b'\''),
            1 => 0xE9,
            _ => i32::from(b'a') + self.below(26) as i32,
        }
    }
}

/// Code points no dictionary word is made of, but which the C++ must still answer for.
const ODD: [i32; 12] = [
    0,
    1,
    0x1F,
    0x20,
    0x2019,
    0xD800,
    0xDFFF,
    0x10000,
    0x10FFFF,
    0x110000,
    -1,
    i32::MIN,
];

/// A word of a kind the dictionary lookup can meet: a dictionary word, a typo, a case variant, an
/// empty or odd word, or one longer than `MAX_WORD_LENGTH`.
pub fn random_word(random: &mut Random, real: &[Vec<i32>]) -> Vec<i32> {
    let base = &real[random.below(real.len())];
    match random.below(16) {
        0 => Vec::new(),
        1 | 2 => (0..1 + random.below(12)).map(|_| random.letter()).collect(),
        3 => (0..1 + random.below(3))
            .map(|_| ODD[random.below(ODD.len())])
            .collect(),
        4 => long_word(random, base),
        5 | 6 => typo(random, base),
        7 => map_first(base, to_upper),
        8 => base.iter().map(|&it| to_upper(it)).collect(),
        9 => base.iter().map(|&it| to_lower(it)).collect(),
        10 => {
            let mut word = base.clone();
            word.push(random.letter());
            word
        }
        11 => base[..random.below(base.len() + 1)].to_vec(),
        _ => base.clone(),
    }
}

/// `base` repeated until it is longer than `MAX_WORD_LENGTH`, by a random amount.
fn long_word(random: &mut Random, base: &[i32]) -> Vec<i32> {
    let mut word = Vec::new();
    while word.len() <= MAX_WORD_LENGTH {
        word.extend_from_slice(base);
        if base.is_empty() {
            word.push(random.letter());
        }
    }
    word.truncate(MAX_WORD_LENGTH + 1 + random.below(12));
    word
}

/// `base` with one letter replaced, dropped, added or swapped with its neighbour.
fn typo(random: &mut Random, base: &[i32]) -> Vec<i32> {
    let mut word = base.to_vec();
    if word.is_empty() {
        return vec![random.letter()];
    }
    let at = random.below(word.len());
    match random.below(4) {
        0 => word[at] = random.letter(),
        1 => {
            word.remove(at);
        }
        2 => word.insert(at, random.letter()),
        _ => {
            if at + 1 < word.len() {
                word.swap(at, at + 1);
            }
        }
    }
    word
}

fn map_first(word: &[i32], change: fn(i32) -> i32) -> Vec<i32> {
    let mut word = word.to_vec();
    if let Some(first) = word.first_mut() {
        *first = change(*first);
    }
    word
}

fn to_upper(code_point: i32) -> i32 {
    if (i32::from(b'a')..=i32::from(b'z')).contains(&code_point) {
        code_point - 32
    } else {
        code_point
    }
}

fn to_lower(code_point: i32) -> i32 {
    if (i32::from(b'A')..=i32::from(b'Z')).contains(&code_point) {
        code_point + 32
    } else {
        code_point
    }
}

/// Collects the words a walk visits.
#[derive(Default)]
struct Collect {
    words: Vec<Vec<i32>>,
}

impl WordListener for Collect {
    fn on_visit_word(&mut self, code_points: &[i32]) {
        self.words.push(code_points.to_vec());
    }
}

/// Checks the attributes of `word_id` after `prev`.
pub fn check_attributes(tape: &mut Tape, pair: &Pair, word_id: i32, prev: &[i32]) {
    let ours = pair
        .ours
        .policy()
        .get_word_attributes_in_context(prev, word_id, None);
    #[cfg(feature = "oracle")]
    let theirs = cpp::word_attributes(&pair.theirs, None, prev, word_id);
    same!(
        tape,
        (
            ours.probability(),
            ours.is_blacklisted(),
            ours.is_not_a_word(),
            ours.is_possibly_offensive()
        ),
        (
            theirs.probability,
            theirs.is_blacklisted,
            theirs.is_not_a_word,
            theirs.is_possibly_offensive
        ),
        "answer {}: probability, blacklisted, not a word, possibly offensive",
        tape.count()
    );
}

/// Checks the code points of `word_id`, up to `MAX_WORD_LENGTH` of them.
pub fn check_code_points(tape: &mut Tape, pair: &Pair, word_id: i32) {
    same!(
        tape,
        code_points(pair, word_id),
        cpp::code_points(&pair.theirs, word_id, MAX_WORD_LENGTH),
        "answer {}: code points of word {word_id}",
        tape.count()
    );
}

#[test]
fn word_list_matches_the_goldens_in_content_and_order() {
    for pair in pairs() {
        #[cfg(feature = "oracle")]
        assert!(
            pair.words == cpp::words(&pair.theirs),
            "the word lists differ"
        );
        let mut tape = pair.tape("word_list");
        for word in &pair.words {
            tape.record("word", word);
        }
        tape.finish();
    }
}

#[test]
fn joined_word_text_matches_the_goldens() {
    for pair in pairs() {
        let mut tape = pair.tape("joined_word_text");
        same!(
            tape,
            get_words(&pair.ours),
            pair.theirs.words(),
            "UTF-16 text"
        );
        tape.finish();
    }
}

#[test]
fn every_word_matches_the_goldens_in_id_probability_attributes_and_code_points() {
    for pair in pairs() {
        let mut tape = pair.tape("every_word");
        for word in &pair.words {
            let id = pair.ours.policy().get_word_id(word, false);
            same!(
                tape,
                (id, pair.ours.probability(word)),
                (
                    cpp::word_id(&pair.theirs, word, false),
                    pair.theirs.probability(word)
                ),
                "answer {}: word id and probability",
                tape.count()
            );
            check_attributes(&mut tape, pair, id, &[]);
            check_code_points(&mut tape, pair, id);
        }
        tape.finish();
    }
}

#[test]
fn random_words_match_the_goldens_in_id_probability_and_attributes() {
    for pair in pairs() {
        let mut tape = pair.tape("random_words");
        let mut random = Random(0x5EED_0001);
        for _ in 0..40_000 {
            let word = random_word(&mut random, &pair.words);
            let force_lower = random.chance(2);
            let id = pair.ours.policy().get_word_id(&word, force_lower);
            same!(
                tape,
                (id, pair.ours.probability(&word)),
                (
                    cpp::word_id(&pair.theirs, &word, force_lower),
                    pair.theirs.probability(&word)
                ),
                "answer {}: word id and probability (length {}, lower {force_lower})",
                tape.count(),
                word.len()
            );
            if id != NOT_A_WORD_ID {
                check_attributes(&mut tape, pair, id, &[]);
                check_code_points(&mut tape, pair, id);
            }
        }
        tape.finish();
    }
}

#[test]
fn random_ids_and_missing_ids_match_the_goldens_code_points() {
    for pair in pairs() {
        let mut tape = pair.tape("random_ids");
        let mut random = Random(0x1D5);
        for _ in 0..5_000 {
            let id = if random.chance(10) {
                NOT_A_WORD_ID
            } else {
                let word = &pair.words[random.below(pair.words.len())];
                pair.ours.policy().get_word_id(word, false)
            };
            check_code_points(&mut tape, pair, id);
        }
        tape.finish();
    }
}
