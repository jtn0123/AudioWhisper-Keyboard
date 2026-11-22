//! Differential tests: the port against LatinIME's C++ `HeaderPolicy` and header readers, on
//! each dictionary (the AOSP one and the one the app ships) and on seeded random headers.

use super::*;
use crate::bytes::{FormatVersion, detect_format_version};
use latin_oracle::hooks::header as cpp;

/// Every format version, the file's numbers and the port's.
#[cfg(not(target_os = "android"))]
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
#[cfg(not(target_os = "android"))]
const ALL_NGRAM_TYPES: [NgramType; 4] = [
    NgramType::Unigram,
    NgramType::Bigram,
    NgramType::Trigram,
    NgramType::Quadgram,
];

/// Attribute keys the C++ reads, and some it does not.
#[cfg(not(target_os = "android"))]
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
#[cfg(not(target_os = "android"))]
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
#[cfg(not(target_os = "android"))]
const ODD_VALUES: &[&str] = &[
    "-", "+1", "--1", "1-", "0x10", "-0", "007", " 5", "1.5", "true",
];

/// SplitMix64, seeded, so a failure reproduces.
#[cfg(not(target_os = "android"))]
struct Random(u64);

#[cfg(not(target_os = "android"))]
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
#[cfg(not(target_os = "android"))]
fn random_ascii(random: &mut Random) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
    let length = random.below(13);
    (0..length)
        .map(|_| char::from(random.pick(ALPHABET)))
        .collect()
}

/// Code points, a fifth of them one-byte, and the rest across the three-byte range.
#[cfg(not(target_os = "android"))]
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
#[cfg(not(target_os = "android"))]
fn number_text(random: &mut Random) -> Vec<i32> {
    let magnitude = 10_u64.pow(random.below(12) as u32 + 1);
    let value = random.next() % magnitude;
    let sign = if random.chance(30) { "-" } else { "" };
    text(&format!("{sign}{value}"))
}

#[cfg(not(target_os = "android"))]
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
#[cfg(not(target_os = "android"))]
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
#[cfg(not(target_os = "android"))]
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

/// Compares every getter of the port with the C++ `HeaderPolicy` built from the same bytes, and
/// the attribute map, in full. Each read is compared over every output size it can be given.
#[cfg(not(target_os = "android"))]
fn assert_same_policy(buffer: &[u8], version: FormatVersion, now: i32) {
    let ours = HeaderPolicy::with_current_time(buffer, version, now);
    let theirs = cpp::Policy::new(buffer, version as i32, now);
    let context = || format!("{version:?} at {now}: {buffer:02x?}");
    assert_eq!(ours.size(), theirs.size(), "size: {}", context());
    assert_eq!(
        ours.format_version_number(),
        theirs.format_version_number(),
        "{}",
        context()
    );
    assert_eq!(ours.is_valid(), theirs.is_valid(), "{}", context());
    assert_eq!(
        ours.multi_word_cost_multiplier().to_bits(),
        theirs.multi_word_cost_multiplier().to_bits(),
        "{}",
        context()
    );
    assert_eq!(
        ours.is_decaying_dict(),
        theirs.is_decaying_dict(),
        "{}",
        context()
    );
    assert_eq!(
        ours.requires_german_umlaut_processing(),
        theirs.requires_german_umlaut_processing(),
        "{}",
        context()
    );
    assert_eq!(ours.date(), theirs.date(), "{}", context());
    assert_eq!(
        ours.last_decayed_time(),
        theirs.last_decayed_time(),
        "{}",
        context()
    );
    for ngram_type in ALL_NGRAM_TYPES {
        let count = ours.ngram_counts().ngram_count(ngram_type);
        assert_eq!(
            count,
            theirs.ngram_count(ngram_type as i32, false),
            "{}",
            context()
        );
        let max = ours.max_ngram_counts().ngram_count(ngram_type);
        assert_eq!(
            max,
            theirs.ngram_count(ngram_type as i32, true),
            "{}",
            context()
        );
    }
    assert_eq!(
        ours.extended_region_size(),
        theirs.extended_region_size(),
        "{}",
        context()
    );
    assert_eq!(
        ours.has_historical_info_of_words(),
        theirs.has_historical_info_of_words(),
        "{}",
        context()
    );
    assert_eq!(
        ours.should_boost_exact_matches(),
        theirs.should_boost_exact_matches(),
        "{}",
        context()
    );
    assert_eq!(
        ours.forgetting_curve_probability_values_table_id(),
        theirs.forgetting_curve_table_id(),
        "{}",
        context()
    );
    assert_eq!(
        ours.supports_beginning_of_sentence(),
        theirs.supports_beginning_of_sentence(),
        "{}",
        context()
    );
    assert_eq!(
        ours.code_point_table().map(<[i32]>::to_vec),
        theirs.code_point_table(),
        "{}",
        context()
    );
    assert_eq!(ours.locale().to_vec(), theirs.locale(), "{}", context());
    let our_attributes: Vec<_> = ours
        .attribute_map()
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    assert_eq!(our_attributes, theirs.attributes(), "{}", context());
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
            let mut ours_out = vec![UNTOUCHED; size];
            ours.read_header_value_or_question_mark(key, &mut ours_out);
            let mut theirs_out = vec![UNTOUCHED; size];
            theirs.read_value_or_question_mark(key, &mut theirs_out);
            assert_eq!(ours_out, theirs_out, "{key:?} in {size}: {}", context());
        }
    }
}

/// Compares the static readers on `buffer`'s attribute map with the C++ ones: the fetch, the
/// fixed fields, and the readers for real keys and random ones.
#[cfg(not(target_os = "android"))]
fn assert_same_attributes(buffer: &[u8], random: &mut Random) {
    let ours = fetch_all_header_attributes(buffer);
    let theirs = cpp::Attributes::fetch(buffer);
    let context = || format!("{buffer:02x?}");
    let our_entries: Vec<_> = ours
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    assert_eq!(our_entries, theirs.entries(), "{}", context());
    assert_eq!(
        get_header_size(buffer),
        cpp::get_header_size(buffer),
        "{}",
        context()
    );
    assert_eq!(
        i32::from(get_flags(buffer)),
        cpp::get_flags(buffer),
        "{}",
        context()
    );
    assert_eq!(
        read_code_point_table(&ours).map(<[i32]>::to_vec),
        theirs.code_point_table(),
        "{}",
        context()
    );
    let mut keys: Vec<String> = REAL_KEYS.iter().map(|key| (*key).to_string()).collect();
    keys.extend((0..20).map(|_| random_ascii(random)));
    for key in keys {
        let default = random.pick(&[0, -1, 42, i32::MIN, i32::MAX]);
        assert_eq!(
            read_int_attribute_value(&ours, &key, default),
            theirs.read_int(&key, default),
            "{key:?}: {}",
            context()
        );
        let default_bool = random.chance(50);
        assert_eq!(
            read_bool_attribute_value(&ours, &key, default_bool),
            theirs.read_bool(&key, default_bool),
            "{key:?}: {}",
            context()
        );
        assert_eq!(
            read_code_point_vector_attribute_value(&ours, &key),
            theirs.read_code_point_vector(&key),
            "{key:?}: {}",
            context()
        );
    }
}

#[cfg(not(target_os = "android"))]
#[test]
fn each_dictionary_header_matches_the_cpp() {
    for path in latin_oracle::dictionary_paths() {
        let buffer = std::fs::read(path).expect("a dictionary");
        let version = detect_format_version(&buffer);
        assert_eq!(version, FormatVersion::Version202);
        assert_same_policy(&buffer, version, 1_700_000_000);
        assert_same_attributes(&buffer, &mut Random(1));
    }
    let buffer = std::fs::read(latin_oracle::aosp_dictionary_path()).expect("the AOSP dictionary");
    let policy = HeaderPolicy::with_current_time(&buffer, FormatVersion::Version202, 1_700_000_000);
    assert_eq!(policy.size(), 99);
    assert_eq!(policy.locale(), &text("en_US")[..]);
    assert_eq!(policy.date(), 1_414_726_260);
    assert_eq!(policy.code_point_table(), None);
}

#[cfg(not(target_os = "android"))]
#[test]
fn random_headers_match_the_cpp() {
    let mut random = Random(20_261_109);
    for _ in 0..3_000 {
        let buffer = random_header(&mut random);
        let version = random.pick(&VERSIONS[..]);
        let now = random.next() as i32;
        assert_same_policy(&buffer, version, now);
        assert_same_attributes(&buffer, &mut random);
    }
}

/// Whether a header size larger than the buffer would make the C++ read past its padding, which is
/// undefined. The port's answer there is tested on its own (`a_declared_size_past_the_buffer...`).
#[cfg(not(target_os = "android"))]
fn declares_past_the_buffer(buffer: &[u8]) -> bool {
    usize::try_from(get_header_size(buffer)).is_ok_and(|size| size > buffer.len())
}

#[cfg(not(target_os = "android"))]
#[test]
fn random_buffers_match_the_cpp() {
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
        assert_same_policy(&buffer, version, now);
        assert_same_attributes(&buffer, &mut random);
    }
    assert!(
        compared > 1_000,
        "only {compared} buffers declare a size inside themselves"
    );
}

#[cfg(not(target_os = "android"))]
#[test]
fn integer_and_bool_values_match_the_cpp() {
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
    let mut random = Random(7);
    for key in KEYS {
        for value in INTEGER_TEXTS {
            let buffer = header_of(&[(key, value)]);
            assert_same_policy(&buffer, FormatVersion::Version202, 99);
            assert_same_attributes(&buffer, &mut random);
        }
    }
}

#[cfg(not(target_os = "android"))]
#[test]
fn static_readers_match_the_cpp_on_every_format_version() {
    let mut random = Random(99);
    for version in VERSIONS {
        let buffer = random_header(&mut random);
        assert_same_policy(&buffer, version, 5);
        assert_same_attributes(&buffer, &mut random);
    }
}
