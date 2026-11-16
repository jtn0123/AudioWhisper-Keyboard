//! The reader on small files made here by hand. `crates/lexicon` tests the builder's files.

use super::format::{self, HEADER_LEN, Header, Unigram, pack};
use super::*;

const STEP: f32 = 0.1;

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// A model of "a", "b", "c" (ids 0-2; the sentence start is 3) with bigrams a→b, a→c and
/// start→a, and the trigram (start, a)→c.
fn bytes() -> Vec<u8> {
    let words = ["a", "b", "c"];
    let header = Header {
        words: 3,
        units: 3,
        bigrams: 3,
        contexts: 1,
        trigrams: 1,
        step: STEP,
        backoff_step: STEP,
        scale_a: 240.0,
        scale_b: 11.0,
    };
    let mut out = header.to_bytes().to_vec();
    let push = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes());
    for start in 0..=3 {
        push(&mut out, start);
    }
    for w in words {
        out.extend_from_slice(&utf16(w)[0].to_le_bytes());
    }
    out.resize(out.len().next_multiple_of(4), 0);
    let unigram = |p, backoff, frequency| Unigram {
        probability: p,
        backoff,
        frequency,
        flags: 0,
    };
    for u in [
        unigram(10, 5, 9),
        unigram(20, 0, 21),
        unigram(30, 0, 29),
        unigram(255, 7, 255),
    ] {
        out.extend_from_slice(&u.to_bytes());
    }
    // a: bigrams 0-1; b, c: none; the sentence start: bigram 2.
    for start in [0, 2, 2, 2, 3] {
        push(&mut out, start);
    }
    for (word, q) in [(1, 12), (2, 15), (0, 4)] {
        push(&mut out, pack(word, q));
    }
    out.extend_from_slice(&[0, 0, 6, 0]);
    // Bigram 2 (start, a) has trigram c.
    for v in [2, 0, 1, pack(2, 8)] {
        push(&mut out, v);
    }
    out
}

fn model() -> Model {
    Model::open(bytes()).expect("a valid model")
}

fn near(a: Option<f32>, b: f32) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-5)
}

#[test]
fn reads_the_header_and_words() {
    let m = model();
    assert_eq!(m.words(), 3);
    assert_eq!(m.header().scale_b, 11.0);
    assert_eq!(m.word(&utf16("b")), Some(1));
    assert_eq!(m.word(&utf16("B")), Some(1));
    assert_eq!(m.word(&utf16("d")), None);
    assert_eq!(m.spelling(2), Some(utf16("c")));
    assert_eq!(m.spelling(3), None);
}

#[test]
fn backs_off_through_the_stored_ngrams() {
    let m = model();
    let after = |newest, older| Context { newest, older };
    // A stored bigram, then a missing one (a's backoff, then c's unigram).
    assert!(near(m.ln_probability(after(Some(0), None), 1), -1.2));
    assert!(near(m.ln_probability(after(Some(1), None), 2), -3.0));
    assert!(near(m.ln_probability(after(Some(0), None), 0), -0.5 - 1.0));
    // A stored trigram, then (start, a) backing off to a's bigram with its backoff.
    let start_a = after(Some(0), Some(3));
    assert!(near(m.ln_probability(start_a, 2), -0.8));
    assert!(near(m.ln_probability(start_a, 1), -0.6 - 1.2));
    // No context: the unigram. The sentence start is never a word.
    assert!(near(m.ln_probability(Context::default(), 1), -2.0));
    assert_eq!(m.ln_probability(Context::default(), 3), None);
    assert!(near(m.lift(start_a, 2), -0.8 + 2.9));
    assert_eq!(m.lift(Context::default(), 2), None);
}

#[test]
fn predicts_the_likeliest_words_first() {
    let m = model();
    let words = |p: Vec<(u32, f32)>| p.into_iter().map(|(id, _)| id).collect::<Vec<_>>();
    let start_a = Context {
        newest: Some(0),
        older: Some(3),
    };
    // c by its trigram (-0.8), b by a's bigram (-1.8), then a by a's backoff (-0.6-0.5-1.0).
    assert_eq!(words(m.predict(start_a, 5)), vec![2, 1, 0]);
    assert_eq!(words(m.predict(start_a, 1)), vec![2]);
    assert_eq!(words(m.predict(Context::default(), 5)), vec![0, 1, 2]);
    let start = Context {
        newest: Some(3),
        older: None,
    };
    assert_eq!(words(m.predict(start, 2)), vec![0, 1]);
}

#[test]
fn a_damaged_file_does_not_open() {
    let good = bytes();
    assert!(Model::open(good[..good.len() - 4].to_vec()).is_none());
    let mut longer = good.clone();
    longer.extend_from_slice(&[0; 4]);
    assert!(Model::open(longer).is_none());
    let mut magic = good.clone();
    magic[0] = b'X';
    assert!(Model::open(magic).is_none());
    let mut version = good.clone();
    version[4] = 9;
    assert!(Model::open(version).is_none());
    assert!(Model::open(good[..HEADER_LEN - 1].to_vec()).is_none());
    // A trigram count the starts don't end on.
    let mut count = good.clone();
    count[24] = 2;
    count.extend_from_slice(&[0; 4]);
    assert!(Model::open(count).is_none());
}

#[test]
fn damaged_entries_give_no_answer_rather_than_a_panic() {
    let mut damaged = bytes();
    let layout = format::Layout::of(&Header::read(&damaged).unwrap()).unwrap();
    // Point word a's bigrams past the end of the section.
    damaged[layout.bigram_starts + 4..layout.bigram_starts + 8]
        .copy_from_slice(&900u32.to_le_bytes());
    damaged[layout.spelling_starts + 4..layout.spelling_starts + 8]
        .copy_from_slice(&900u32.to_le_bytes());
    let m = Model::open(damaged).unwrap();
    let a = Context {
        newest: Some(0),
        older: None,
    };
    assert_eq!(m.spelling(0), None);
    assert!(m.ln_probability(a, 1).is_some());
    assert!(m.predict(a, 3).len() <= 3);
}

#[test]
fn packs_and_quantises() {
    assert_eq!(format::unpack(pack(123_456, 200)), (123_456, 200));
    assert_eq!(format::quantise(-1.04, 0.1), 10);
    assert_eq!(format::quantise(-100.0, 0.1), 255);
    assert_eq!(format::quantise(0.5, 0.1), 0);
    assert_eq!(format::ln(10, 0.1), -1.0);
}
