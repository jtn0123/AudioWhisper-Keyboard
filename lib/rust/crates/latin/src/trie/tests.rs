//! Differential tests of the version 2 trie against LatinIME's C++, through `latin-oracle`. Each test
//! asks the same question of the bundled dictionary both ways and asserts the answers agree. Random
//! inputs come from seeded SplitMix64 streams, so a failure reproduces from its seed. Assertion
//! messages carry indices and numbers only: a dictionary word is never printed.

mod context;
mod nodes;
mod robust;
mod units;

use std::sync::{Arc, OnceLock};

use latin_oracle::hooks::trie as cpp;

use super::dictionary::Dictionary;
use super::interface::WordListener;
use super::jni::get_words;
use crate::defines::{MAX_WORD_LENGTH, NOT_A_WORD_ID};

/// The C++ side's dictionary, as `latin-oracle` opens it.
pub type CppDictionary = latin_oracle::Dictionary;

/// The bundled dictionary, opened on both sides, with the C++'s word list as the reference.
pub struct Pair {
    pub ours: Dictionary,
    pub theirs: CppDictionary,
    pub words: Vec<Vec<i32>>,
}

/// The bundled dictionary, loaded once for the whole test binary.
pub fn pair() -> &'static Pair {
    static PAIR: OnceLock<Pair> = OnceLock::new();
    PAIR.get_or_init(|| {
        let file: Arc<[u8]> = std::fs::read(latin_oracle::bundled_dictionary_path())
            .expect("the bundled dictionary")
            .into();
        let theirs = CppDictionary::bundled();
        let words = cpp::words(&theirs);
        Pair {
            ours: Dictionary::open(file).expect("a version 2 dictionary"),
            theirs,
            words,
        }
    })
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

/// Asserts the attributes of `word_id` after `prev` agree, with `label` naming the case.
pub fn check_attributes(pair: &Pair, word_id: i32, prev: &[i32], label: &str) {
    let ours = pair
        .ours
        .policy()
        .get_word_attributes_in_context(prev, word_id, None);
    let theirs = cpp::word_attributes(&pair.theirs, None, prev, word_id);
    assert_eq!(
        ours.probability(),
        theirs.probability,
        "{label}: probability"
    );
    assert_eq!(
        ours.is_blacklisted(),
        theirs.is_blacklisted,
        "{label}: blacklisted"
    );
    assert_eq!(
        ours.is_not_a_word(),
        theirs.is_not_a_word,
        "{label}: not a word"
    );
    assert_eq!(
        ours.is_possibly_offensive(),
        theirs.is_possibly_offensive,
        "{label}: possibly offensive"
    );
}

/// Asserts the code points of `word_id` agree, up to `MAX_WORD_LENGTH` of them.
pub fn check_code_points(pair: &Pair, word_id: i32, label: &str) {
    let mut out = [0_i32; MAX_WORD_LENGTH];
    let count = pair
        .ours
        .policy()
        .get_code_points_and_return_code_point_count(word_id, MAX_WORD_LENGTH, &mut out);
    let theirs = cpp::code_points(&pair.theirs, word_id, MAX_WORD_LENGTH);
    assert_eq!(count, theirs.len(), "{label}: code point count");
    assert!(out[..count] == theirs[..], "{label}: code points");
}

#[test]
fn word_list_matches_the_cpp_in_content_and_order() {
    let pair = pair();
    let mut collected = Collect::default();
    pair.ours.iterate_words(&mut collected);
    assert_eq!(collected.words.len(), pair.words.len(), "word count");
    for (index, (ours, theirs)) in collected.words.iter().zip(&pair.words).enumerate() {
        assert!(ours == theirs, "word {index} differs");
    }
}

#[test]
fn joined_word_text_matches_the_cpp_get_words() {
    let pair = pair();
    let ours = get_words(&pair.ours);
    let theirs = pair.theirs.words();
    assert_eq!(ours.len(), theirs.len(), "UTF-16 length");
    assert!(ours == theirs, "UTF-16 text differs");
}

#[test]
fn every_word_has_the_same_id_probability_attributes_and_code_points() {
    let pair = pair();
    for (index, word) in pair.words.iter().enumerate() {
        let label = format!("word {index}");
        let id = pair.ours.policy().get_word_id(word, false);
        assert_eq!(
            id,
            cpp::word_id(&pair.theirs, word, false),
            "{label}: word id"
        );
        assert_eq!(
            pair.ours.probability(word),
            pair.theirs.probability(word),
            "{label}: probability"
        );
        check_attributes(pair, id, &[], &label);
        check_code_points(pair, id, &label);
    }
}

#[test]
fn random_words_match_the_cpp_in_id_probability_and_attributes() {
    let pair = pair();
    let mut random = Random(0x5EED_0001);
    for round in 0..40_000 {
        let word = random_word(&mut random, &pair.words);
        let force_lower = random.chance(2);
        let label = format!("round {round}");
        let id = pair.ours.policy().get_word_id(&word, force_lower);
        assert_eq!(
            id,
            cpp::word_id(&pair.theirs, &word, force_lower),
            "{label}: word id (length {}, lower {force_lower})",
            word.len()
        );
        assert_eq!(
            pair.ours.probability(&word),
            pair.theirs.probability(&word),
            "{label}: probability (length {})",
            word.len()
        );
        if id != NOT_A_WORD_ID {
            check_attributes(pair, id, &[], &label);
            check_code_points(pair, id, &label);
        }
    }
}

#[test]
fn random_ids_and_missing_ids_match_the_cpp_code_points() {
    let pair = pair();
    let mut random = Random(0x1D5);
    for round in 0..5_000 {
        let id = if random.chance(10) {
            NOT_A_WORD_ID
        } else {
            let word = &pair.words[random.below(pair.words.len())];
            pair.ours.policy().get_word_id(word, false)
        };
        check_code_points(pair, id, &format!("round {round}"));
    }
}
