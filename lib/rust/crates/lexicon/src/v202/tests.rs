use super::*;
use std::collections::BTreeMap;

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

/// Prefixes of other words, single-child runs that become one multi-character node, characters of
/// one and three bytes, flags, shortcuts, and next words pointing both ways.
fn sample() -> Dictionary {
    let mut entries = vec![
        entry("a", 200),
        entry("ab", 3),
        entry("abacus", 90),
        entry("café", 120),
        entry("cafés", 60),
        entry("damn", 100),
        entry("hello", 180),
        entry("help", 170),
        entry("xyzzy", 1),
        entry("ŝi", 40),
        entry("日本", 255),
    ];
    entries[5].possibly_offensive = true;
    entries[8].not_a_word = true;
    entries[8].shortcuts = vec![(code_points("plugh"), 15), (code_points("日"), 3)];
    entries[6].shortcuts = vec![(code_points("hi"), 4)];
    entries[0].bigrams = vec![(10, 15), (6, 0), (2, 7)];
    entries[7].bigrams = vec![(0, 3), (6, 14)];
    entries[10].bigrams = vec![(9, 1)];
    Dictionary {
        flags: 0,
        attributes: vec![
            ("dictionary".to_string(), "main:en_us".to_string()),
            ("description".to_string(), "Test (ŝ)".to_string()),
            ("version".to_string(), "55".to_string()),
        ],
        entries,
    }
}

/// Each word with what the dictionary says of it, next words by spelling: the reader returns words
/// in file order, so indexes differ from the writer's input.
type Words = BTreeMap<String, (u8, bool, bool, Vec<(Vec<u32>, u8)>, Vec<(String, u8)>)>;

fn words(dictionary: &Dictionary) -> Words {
    let spell = |i: usize| string(&dictionary.entries[i].word);
    dictionary
        .entries
        .iter()
        .map(|e| {
            let bigrams = e.bigrams.iter().map(|&(t, p)| (spell(t), p)).collect();
            let about = (e.probability, e.not_a_word, e.possibly_offensive);
            (
                string(&e.word),
                (about.0, about.1, about.2, e.shortcuts.clone(), bigrams),
            )
        })
        .collect()
}

#[test]
fn writes_the_header_the_keyboard_reads() {
    let bytes = write(&sample()).unwrap();
    assert_eq!(bytes[..4], [0x9B, 0xC1, 0x3A, 0xFE]);
    assert_eq!(bytes[4..8], [0, 202, 0, 0]);
    let size = u32::from_be_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let back = read(&bytes).unwrap();
    assert_eq!(back.attribute("description"), Some("Test (ŝ)"));
    assert_eq!(back.attribute("version"), Some("55"));
    assert_eq!(back.attribute("locale"), None);
    assert_eq!(back.attributes, sample().attributes);
    // The first node array starts right after the header: its size, then the root's children.
    assert_eq!(usize::from(bytes[size]), 7);
    let mut flagged = sample();
    flagged.flags = 0x0102;
    assert_eq!(read(&write(&flagged).unwrap()).unwrap().flags, 0x0102);
}

#[test]
fn reads_back_every_word_flag_shortcut_and_next_word() {
    let dictionary = sample();
    let back = read(&write(&dictionary).unwrap()).unwrap();
    assert_eq!(back.entries.len(), dictionary.entries.len());
    let found = words(&back);
    assert_eq!(found, words(&dictionary));
    assert!(found["damn"].2 && !found["damn"].1);
    assert!(found["xyzzy"].1 && !found["xyzzy"].2);
    assert_eq!(found["xyzzy"].3[1], (code_points("日"), 3));
    let after_a: Vec<(&str, u8)> = found["a"].4.iter().map(|(w, p)| (w.as_str(), *p)).collect();
    assert_eq!(after_a, [("日本", 15), ("hello", 0), ("abacus", 7)]);
    // Read again and written again, it is the same file.
    assert_eq!(write(&back).unwrap(), write(&dictionary).unwrap());
}

#[test]
fn takes_words_in_any_order() {
    let mut shuffled = sample();
    shuffled.entries.reverse();
    let n = shuffled.entries.len();
    for e in &mut shuffled.entries {
        for (target, _) in &mut e.bigrams {
            *target = n - 1 - *target;
        }
    }
    assert_eq!(write(&shuffled).unwrap(), write(&sample()).unwrap());
}

#[test]
fn turns_away_what_the_format_cannot_hold() {
    let mut twice = sample();
    twice.entries.push(entry("help", 1));
    assert!(write(&twice).is_err());
    let mut empty = sample();
    empty.entries.push(entry("", 1));
    assert!(write(&empty).is_err());
    let mut long = sample();
    long.entries
        .push(entry(&"a".repeat(MAX_WORD_LENGTH + 1), 1));
    assert!(write(&long).is_err());
    let mut dangling = sample();
    dangling.entries[1].bigrams.push((99, 1));
    assert!(write(&dangling).is_err());
    let bytes = write(&sample()).unwrap();
    assert!(read(&bytes[1..]).is_err());
    assert!(read(&bytes[..bytes.len() - 1]).is_err());
}

/// The AOSP dictionary the app shipped before, rewritten from what was read, comes out byte for
/// byte: the strongest check that this is the keyboard's format, offsets and order included.
#[test]
fn rewrites_the_aosp_dictionary_byte_for_byte() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../latin/tests/fixtures/aosp_main_en-US.dict");
    let bytes = std::fs::read(path).unwrap();
    let dictionary = read(&bytes).unwrap();
    assert_eq!(dictionary.entries.len(), 160_715);
    let bigrams: usize = dictionary.entries.iter().map(|e| e.bigrams.len()).sum();
    assert_eq!(bigrams, 481_875);
    assert_eq!(dictionary.attribute("version"), Some("54"));
    assert!(write(&dictionary).unwrap() == bytes);
}
