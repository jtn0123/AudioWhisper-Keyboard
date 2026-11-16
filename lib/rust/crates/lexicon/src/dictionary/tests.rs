use super::*;
use crate::count::count;
use crate::v202::{self, code_points, string};

#[test]
fn steps_match_the_keyboards_encoding() {
    // The keyboard's value for each step lands back on that step. (Above a unigram of about 225 a
    // step is under a point wide and AOSP's encoding, made for whole values, comes out a step
    // low; such words are few, and the encoding is the keyboard's.)
    for unigram in [0_u8, 90, 180, 220] {
        for s in 0..=MAX_STEP {
            assert_eq!(step(unigram, combined(unigram, s)), s, "{unigram} {s}");
        }
        assert_eq!(step(unigram, f64::from(unigram)), 0);
        assert_eq!(step(unigram, 999.0), MAX_STEP);
    }
}

fn entry(word: &str, probability: u8) -> Entry {
    Entry {
        word: code_points(word),
        probability,
        not_a_word: false,
        possibly_offensive: false,
        shortcuts: Vec::new(),
        bigrams: Vec::new(),
    }
}

fn old() -> Dictionary {
    let mut entries = vec![
        entry("the", 200),
        entry("cat", 150),
        entry("dog", 180),
        entry("sat", 100),
        entry("damn", 120),
        entry("xyz", 50),
        entry("ran", 90),
        entry("zebra", 170),
    ];
    entries[4].possibly_offensive = true;
    entries[5].not_a_word = true;
    entries[5].shortcuts = vec![(code_points("xyzzy"), 15)];
    entries[0].bigrams = vec![(7, 0)];
    Dictionary {
        flags: 0,
        attributes: vec![
            ("date".to_string(), "1414726260".to_string()),
            ("version".to_string(), "54".to_string()),
        ],
        entries,
    }
}

fn corpus() -> Vec<&'static str> {
    let mut lines = Vec::new();
    for (line, times) in [
        ("the cat sat", 30),
        ("the dog ran", 10),
        ("the cat ran", 5),
        ("the damn cat", 20),
        ("the xyz", 5),
        ("dog sat", 1),
    ] {
        lines.extend(std::iter::repeat_n(line, times));
    }
    lines
}

fn build(next: usize) -> Dictionary {
    let options = Options {
        next,
        min_pair: 2,
        date: 1_794_787_200,
    };
    let old = old();
    rebuild(&old, &count(&old, &corpus(), 3), options).0
}

fn next_words(d: &Dictionary, word: &str) -> Vec<String> {
    let i = d
        .entries
        .iter()
        .position(|e| string(&e.word) == word)
        .unwrap();
    d.entries[i]
        .bigrams
        .iter()
        .map(|&(t, _)| string(&d.entries[t].word))
        .collect()
}

#[test]
fn hands_out_the_old_values_by_corpus_frequency() {
    let new = build(32);
    let values: Vec<(String, u8)> = new
        .entries
        .iter()
        .map(|e| (string(&e.word), e.probability))
        .collect();
    let expected = [
        ("the", 200),
        ("cat", 180),
        ("dog", 100),
        ("sat", 170),
        ("damn", 120),
        ("xyz", 50),
        ("ran", 150),
        // Never written: below every word that was.
        ("zebra", 90),
    ];
    let expected: Vec<(String, u8)> = expected.iter().map(|&(w, v)| (w.to_string(), v)).collect();
    assert_eq!(values, expected);
    // Words, flags and shortcuts are the old ones.
    assert!(new.entries[4].possibly_offensive && new.entries[5].not_a_word);
    assert_eq!(new.entries[5].shortcuts, old().entries[5].shortcuts);
    assert_eq!(new.attributes[0].1, "1794787200");
    assert_eq!(new.attributes[1].1, "55");
}

#[test]
fn stores_the_most_frequent_followers_but_never_a_flagged_word() {
    let new = build(32);
    // "the damn" and "the xyz" are written, but flagged words are never offered.
    assert_eq!(next_words(&new, "the"), ["cat", "dog"]);
    assert_eq!(next_words(&build(1), "the"), ["cat"]);
    // A flagged word may come before others.
    assert_eq!(next_words(&new, "damn"), ["cat"]);
    // "dog ran" ten times, but "dog sat" once: too few to trust.
    assert_eq!(next_words(&new, "dog"), ["ran"]);
    // After "the", "cat" is likelier than on its own: a step above its value.
    let the = &new.entries[0];
    assert!(the.bigrams[0].1 > 0);
}

#[test]
fn the_rebuilt_dictionary_reads_back() {
    let new = build(32);
    let back = v202::read(&v202::write(&new).unwrap()).unwrap();
    assert_eq!(back.entries.len(), new.entries.len());
    assert_eq!(next_words(&back, "the"), ["cat", "dog"]);
}
