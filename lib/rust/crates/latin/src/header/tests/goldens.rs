//! The port's `HeaderPolicy` and header readers against LatinIME's answers (the goldens), on
//! each dictionary (the AOSP one and the one the app ships) and on seeded random headers.

use super::*;
use crate::bytes::{FormatVersion, detect_format_version};
use crate::golden::{Tape, record, tape};

/// Every format version, the file's numbers and the port's.
const VERSIONS: [FormatVersion; 7] = [
    FormatVersion::Version2,
    FormatVersion::Version201,
    FormatVersion::Version202,
    FormatVersion::Version4OnlyForTesting,
    FormatVersion::Version402,
    FormatVersion::Version403,
    FormatVersion::Unknown,
];

/// Every ngram type, the four the header holds counts for.
const ALL_NGRAM_TYPES: [NgramType; 4] = [
    NgramType::Unigram,
    NgramType::Bigram,
    NgramType::Trigram,
    NgramType::Quadgram,
];

/// Attribute keys the C++ reads, and some it does not.
const REAL_KEYS: &[&str] = &[
    "MULTIPLE_WORDS_DEMOTION_RATE",
    "REQUIRES_GERMAN_UMLAUT_PROCESSING",
    "USES_FORGETTING_CURVE",
    "date",
    "LAST_DECAYED_TIME",
    "UNIGRAM_COUNT",
    "BIGRAM_COUNT",
    "TRIGRAM_COUNT",
    "QUADGRAM_COUNT",
    "MAX_UNIGRAM_ENTRY_COUNT",
    "MAX_BIGRAM_ENTRY_COUNT",
    "MAX_TRIGRAM_ENTRY_COUNT",
    "MAX_QUADGRAM_ENTRY_COUNT",
    "EXTENDED_REGION_SIZE",
    "HAS_HISTORICAL_INFO",
    "locale",
    "FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID",
    "codePointTable",
    "dictionary",
    "version",
    "FORGETTING_CURVE_OCCURRENCES_TO_LEVEL_UP",
    "FORGETTING_CURVE_DURATION_TO_LEVEL_DOWN_IN_SECONDS",
];

/// Values that are numbers in some ways and not in others.
const INTEGER_TEXTS: &[&str] = &[
    "",
    "-",
    "-0",
    "0",
    "007",
    "+1",
    "1-",
    "--1",
    "12a",
    " 1",
    "-1",
    "2147483647",
    "-2147483648",
    "2147483648",
    "-2147483649",
    "99999999999999999999",
    "1.5",
    "0x10",
];

/// Values that are not numbers, for random attributes.
const ODD_VALUES: &[&str] = &[
    "-", "+1", "--1", "1-", "0x10", "-0", "007", " 5", "1.5", "true",
];

/// SplitMix64, seeded, so a failure reproduces.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len() as u64) as usize]
    }
}

/// A random key of ASCII letters, digits, `_` and `-`.
fn random_ascii(random: &mut Random) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
    let length = random.below(13);
    (0..length)
        .map(|_| char::from(random.pick(ALPHABET)))
        .collect()
}

/// Code points, a fifth of them one-byte, and the rest across the three-byte range.
fn random_code_points(random: &mut Random, max_length: u64) -> Vec<i32> {
    let length = random.below(max_length + 1);
    (0..length)
        .map(|_| match random.below(8) {
            0 => random.below(0x20) as i32,
            1 => random.below(0x20_0000) as i32,
            2 => i32::from(b'0') + random.below(10) as i32,
            3 => i32::from(b'-'),
            _ => 0x20 + random.below(0xE0) as i32,
        })
        .collect()
}

/// A number of up to 12 digits, sometimes negative: many of them overflow an int.
fn number_text(random: &mut Random) -> Vec<i32> {
    let magnitude = 10_u64.pow(random.below(12) as u32 + 1);
    let value = random.next() % magnitude;
    let sign = if random.chance(30) { "-" } else { "" };
    text(&format!("{sign}{value}"))
}

fn random_value(random: &mut Random) -> Vec<i32> {
    match random.below(20) {
        0..=7 => number_text(random),
        8 => text(random.pick(ODD_VALUES)),
        9 | 10 => Vec::new(),
        11..=16 => random_code_points(random, 12),
        17 | 18 => random_code_points(random, 60),
        _ => random_code_points(random, 2100),
    }
}

/// A header with random attributes: real keys mostly, others now and then, and values that are
/// numbers, junk, or code points. Some lack their terminator and run into the next attribute, and
/// some declare a size that ends before their last attribute does. Bytes follow the declared size,
/// as they do in a real file.
fn random_header(random: &mut Random) -> Vec<u8> {
    let mut attributes = Vec::new();
    for _ in 0..random.below(10) {
        let key = if random.chance(3) {
            random_code_points(random, 300)
        } else if random.chance(75) {
            text(random.pick(REAL_KEYS))
        } else {
            random_code_points(random, 12)
        };
        attributes.extend(encode(&key, true));
        let value = random_value(random);
        attributes.extend(encode(&value, !random.chance(10)));
    }
    let declared = if random.chance(20) {
        12 + random.below(attributes.len() as u64 + 1) as usize
    } else {
        12 + attributes.len()
    };
    let mut buffer = vec![0x9B, 0xC1, 0x3A, 0xFE];
    buffer.extend((random.next() as u16).to_be_bytes());
    buffer.extend((random.next() as u16).to_be_bytes());
    buffer.extend((declared as u32).to_be_bytes());
    buffer.extend(attributes);
    let tail = random.below(40);
    buffer.extend((0..tail).map(|_| random.below(256) as u8));
    buffer
}

/// Bytes with the shapes a header is made of, but nothing else: a 0x1F, a small byte, or any.
fn random_buffer(random: &mut Random) -> Vec<u8> {
    let length = random.below(300) as usize;
    let mut buffer: Vec<u8> = (0..length)
        .map(|_| match random.below(10) {
            0 => 0x1F,
            1 | 2 => random.below(0x20) as u8,
            _ => random.below(256) as u8,
        })
        .collect();
    if length >= 12 {
        let declared = random.below(length as u64 + 1) as u32;
        buffer[8..12].copy_from_slice(&declared.to_be_bytes());
    }
    buffer
}

/// Records every getter of the port's `HeaderPolicy` built from `buffer`, and the attribute map
/// in full. Each read is recorded over every output size it can be given.
fn record_policy(tape: &mut Tape, buffer: &[u8], version: FormatVersion, now: i32) {
    let ours = HeaderPolicy::with_current_time(buffer, version, now);
    record!(
        tape,
        (
            ours.size(),
            ours.format_version_number(),
            ours.is_valid(),
            ours.multi_word_cost_multiplier().to_bits(),
            ours.is_decaying_dict(),
            ours.requires_german_umlaut_processing(),
            ours.date(),
            ours.last_decayed_time()
        )
    );
    for ngram_type in ALL_NGRAM_TYPES {
        record!(
            tape,
            (
                ours.ngram_counts().ngram_count(ngram_type),
                ours.max_ngram_counts().ngram_count(ngram_type)
            )
        );
    }
    record!(
        tape,
        (
            ours.extended_region_size(),
            ours.has_historical_info_of_words(),
            ours.should_boost_exact_matches(),
            ours.forgetting_curve_probability_values_table_id(),
            ours.supports_beginning_of_sentence()
        )
    );
    record!(tape, ours.code_point_table().map(<[i32]>::to_vec));
    record!(tape, ours.locale().to_vec());
    record!(
        tape,
        ours.attribute_map()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>()
    );
    const QUERY_KEYS: [&str; 10] = [
        "dictionary",
        "version",
        "date",
        "locale",
        "codePointTable",
        "MULTIPLE_WORDS_DEMOTION_RATE",
        "UNIGRAM_COUNT",
        "EXTENDED_REGION_SIZE",
        "no such key",
        "",
    ];
    for key in QUERY_KEYS {
        for size in [0, 1, 2, 3, 4, 9, 40, 2100] {
            let read = |read: &dyn Fn(&mut [i32])| {
                let mut out = vec![UNTOUCHED; size];
                read(&mut out);
                out
            };
            record!(
                tape,
                read(&|out| ours.read_header_value_or_question_mark(key, out))
            );
        }
    }
}

/// Records the static readers on `buffer`'s attribute map: the fetch, the
/// fixed fields, and the readers for real keys and random ones.
fn record_attributes(tape: &mut Tape, buffer: &[u8], random: &mut Random) {
    let ours = fetch_all_header_attributes(buffer);
    record!(
        tape,
        ours.iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>()
    );
    record!(
        tape,
        (get_header_size(buffer), i32::from(get_flags(buffer)))
    );
    record!(tape, read_code_point_table(&ours).map(<[i32]>::to_vec));
    let mut keys: Vec<String> = REAL_KEYS.iter().map(|key| (*key).to_string()).collect();
    keys.extend((0..20).map(|_| random_ascii(random)));
    for key in keys {
        let default = random.pick(&[0, -1, 42, i32::MIN, i32::MAX]);
        let default_bool = random.chance(50);
        record!(
            tape,
            (
                read_int_attribute_value(&ours, &key, default),
                read_bool_attribute_value(&ours, &key, default_bool),
                read_code_point_vector_attribute_value(&ours, &key)
            )
        );
    }
}

#[test]
fn each_dictionary_header_matches_the_goldens() {
    for (name, path) in latin_golden::dictionaries() {
        let mut tape = tape(&format!("header.each_dictionary.{name}"));
        let buffer = std::fs::read(path).expect("a dictionary");
        let version = detect_format_version(&buffer);
        assert_eq!(version, FormatVersion::Version202);
        record_policy(&mut tape, &buffer, version, 1_700_000_000);
        record_attributes(&mut tape, &buffer, &mut Random(1));
        tape.finish();
    }
    let buffer = std::fs::read(latin_golden::aosp_dictionary_path()).expect("the AOSP dictionary");
    let policy = HeaderPolicy::with_current_time(&buffer, FormatVersion::Version202, 1_700_000_000);
    assert_eq!(policy.size(), 99);
    assert_eq!(policy.locale(), &text("en_US")[..]);
    assert_eq!(policy.date(), 1_414_726_260);
    assert_eq!(policy.code_point_table(), None);
}

#[test]
fn random_headers_match_the_goldens() {
    let mut tape = tape("header.random_headers");
    let mut random = Random(20_261_109);
    for _ in 0..3_000 {
        let buffer = random_header(&mut random);
        let version = random.pick(&VERSIONS[..]);
        let now = random.next() as i32;
        record_policy(&mut tape, &buffer, version, now);
        record_attributes(&mut tape, &buffer, &mut random);
    }
    tape.finish();
}

/// Whether a header size larger than the buffer would make the C++ read past its padding, which is
/// undefined. The port's answer there is tested on its own (`a_declared_size_past_the_buffer...`).
fn declares_past_the_buffer(buffer: &[u8]) -> bool {
    usize::try_from(get_header_size(buffer)).is_ok_and(|size| size > buffer.len())
}

#[test]
fn random_buffers_match_the_goldens() {
    let mut tape = tape("header.random_buffers");
    let mut random = Random(4_242);
    let mut compared = 0;
    for _ in 0..3_000 {
        let buffer = random_buffer(&mut random);
        if declares_past_the_buffer(&buffer) {
            continue;
        }
        compared += 1;
        let version = random.pick(&VERSIONS[..]);
        let now = random.next() as i32;
        record_policy(&mut tape, &buffer, version, now);
        record_attributes(&mut tape, &buffer, &mut random);
    }
    tape.finish();
    assert!(
        compared > 1_000,
        "only {compared} buffers declare a size inside themselves"
    );
}

#[test]
fn integer_and_bool_values_match_the_goldens() {
    const KEYS: &[&str] = &[
        "EXTENDED_REGION_SIZE",
        "MULTIPLE_WORDS_DEMOTION_RATE",
        "USES_FORGETTING_CURVE",
        "HAS_HISTORICAL_INFO",
        "REQUIRES_GERMAN_UMLAUT_PROCESSING",
        "date",
        "LAST_DECAYED_TIME",
        "UNIGRAM_COUNT",
        "MAX_UNIGRAM_ENTRY_COUNT",
        "FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID",
    ];
    let mut tape = tape("header.integer_and_bool_values");
    let mut random = Random(7);
    for key in KEYS {
        for value in INTEGER_TEXTS {
            let buffer = header_of(&[(key, value)]);
            record_policy(&mut tape, &buffer, FormatVersion::Version202, 99);
            record_attributes(&mut tape, &buffer, &mut random);
        }
    }
    tape.finish();
}

#[test]
fn static_readers_on_every_format_version_match_the_goldens() {
    let mut tape = tape("header.every_format_version");
    let mut random = Random(99);
    for version in VERSIONS {
        let buffer = random_header(&mut random);
        record_policy(&mut tape, &buffer, version, 5);
        record_attributes(&mut tape, &buffer, &mut random);
    }
    tape.finish();
}
