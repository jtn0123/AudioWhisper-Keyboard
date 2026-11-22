use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::bytes::FormatVersion;

/// The golden tests: LatinIME's answers on each dictionary and on seeded random headers.
mod goldens;

/// What a read leaves in an `out` slot it does not write.
const UNTOUCHED: i32 = -7_777;

/// A string's code points, from its ASCII text.
fn text(s: &str) -> Vec<i32> {
    s.bytes().map(i32::from).collect()
}

/// A string as the header stores it, which `bytes::read_code_point_advance` reads back: a code
/// point from 0x20 to 0xFF is one byte, and any other is three, so it must be below 0x200000. A
/// terminated string ends with the 0x1F that stops the read.
fn encode(code_points: &[i32], terminated: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    for &code_point in code_points {
        if (0x20..=0xFF).contains(&code_point) {
            bytes.push(code_point as u8);
        } else {
            bytes.extend_from_slice(&code_point.to_be_bytes()[1..]);
        }
    }
    if terminated {
        bytes.push(0x1F);
    }
    bytes
}

/// The fixed fields (magic number, version 202, no flags) and the declared header size, then the
/// attributes.
fn header_bytes(declared_size: u32, attributes: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x9B, 0xC1, 0x3A, 0xFE, 0, 202, 0, 0];
    bytes.extend_from_slice(&declared_size.to_be_bytes());
    bytes.extend_from_slice(attributes);
    bytes
}

/// A header whose attributes are `pairs`, declaring the size that ends them.
fn header_of(pairs: &[(&str, &str)]) -> Vec<u8> {
    let mut attributes = Vec::new();
    for (key, value) in pairs {
        attributes.extend(encode(&text(key), true));
        attributes.extend(encode(&text(value), true));
    }
    header_bytes(12 + attributes.len() as u32, &attributes)
}

fn policy_of(pairs: &[(&str, &str)]) -> HeaderPolicy {
    HeaderPolicy::with_current_time(&header_of(pairs), FormatVersion::Version202, 0)
}

fn clock_seconds() -> i32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after 1970")
        .as_secs() as i32
}

#[test]
fn integers_are_read_as_the_cpp_reads_them() {
    // "2147483648" overflows an int, which the C++ leaves undefined. The port wraps, which is what
    // the machine does with the C++'s loop.
    let cases = [
        ("", 0),
        ("-", 0),
        ("-0", 0),
        ("007", 7),
        ("-1", -1),
        ("2147483647", i32::MAX),
        ("-2147483648", i32::MIN),
        ("2147483648", i32::MIN),
        ("+1", 42),
        ("1-", 42),
        ("--1", 42),
        ("12a", 42),
        (" 1", 42),
    ];
    for (value, expected) in cases {
        let attributes =
            fetch_all_header_attributes(&header_of(&[("EXTENDED_REGION_SIZE", value)]));
        let policy = policy_of(&[("EXTENDED_REGION_SIZE", value)]);
        assert_eq!(
            read_int_attribute_value(&attributes, "EXTENDED_REGION_SIZE", 42),
            expected,
            "{value:?}"
        );
        // The policy reads the same attribute with the default 0.
        let policy_expected = read_int_attribute_value(&attributes, "EXTENDED_REGION_SIZE", 0);
        assert_eq!(policy.extended_region_size(), policy_expected, "{value:?}");
    }
    assert_eq!(policy_of(&[]).extended_region_size(), 0);
}

#[test]
fn bools_are_nonzero_numbers_and_an_empty_value_is_zero() {
    let cases = [
        ("0", false),
        ("-0", false),
        ("", false),
        ("-", false),
        ("1", true),
        ("-1", true),
        ("2", true),
        ("yes", true),
    ];
    for (value, expected) in cases {
        let attributes = fetch_all_header_attributes(&header_of(&[(
            "REQUIRES_GERMAN_UMLAUT_PROCESSING",
            value,
        )]));
        assert_eq!(
            read_bool_attribute_value(&attributes, "REQUIRES_GERMAN_UMLAUT_PROCESSING", true),
            expected,
            "{value:?}"
        );
    }
    assert!(read_bool_attribute_value(
        &AttributeMap::new(),
        "missing",
        true
    ));
}

#[test]
fn duplicate_keys_keep_the_first_value() {
    let attributes = fetch_all_header_attributes(&header_of(&[("date", "5"), ("date", "9")]));
    assert_eq!(attributes.len(), 1);
    assert_eq!(read_int_attribute_value(&attributes, "date", 0), 5);
}

#[test]
fn an_empty_code_point_table_is_none() {
    let empty = fetch_all_header_attributes(&header_of(&[("codePointTable", "")]));
    assert_eq!(read_code_point_table(&empty), None);
    assert!(empty.contains_key(&text("codePointTable")));
    let table = fetch_all_header_attributes(&header_of(&[("codePointTable", "ab")]));
    assert_eq!(read_code_point_table(&table), Some(&[97, 98][..]));
    assert_eq!(read_code_point_table(&AttributeMap::new()), None);
}

#[test]
fn a_declared_size_past_the_buffer_stops_at_its_end() {
    let body = header_of(&[("date", "5"), ("locale", "en_US")])[12..].to_vec();
    let exact = header_bytes(12 + body.len() as u32, &body);
    let past_the_end = header_bytes(1_000_000, &body);
    assert_eq!(get_header_size(&past_the_end), 1_000_000);
    assert_eq!(
        fetch_all_header_attributes(&past_the_end),
        fetch_all_header_attributes(&exact)
    );
    // A size of 2^31 or more is negative in the C++'s int, and then there are no attributes.
    let negative = header_bytes(u32::MAX, &body);
    assert_eq!(get_header_size(&negative), -1);
    assert!(fetch_all_header_attributes(&negative).is_empty());
}

#[test]
fn the_multi_word_cost_multiplier_follows_the_demotion_rate() {
    let cases: [(Option<&str>, f32); 6] = [
        (None, 1.0),
        (Some("100"), 1.0),
        (Some("25"), 4.0),
        (Some("3"), 100.0_f32 / 3.0_f32),
        (Some("0"), 10_000_000.0),
        (Some("-3"), 10_000_000.0),
    ];
    for (rate, expected) in cases {
        let policy = match rate {
            Some(rate) => policy_of(&[("MULTIPLE_WORDS_DEMOTION_RATE", rate)]),
            None => policy_of(&[]),
        };
        assert_eq!(
            policy.multi_word_cost_multiplier().to_bits(),
            expected.to_bits(),
            "{rate:?}"
        );
    }
}

#[test]
fn quadgram_counts_are_never_read() {
    let policy = policy_of(&[
        ("UNIGRAM_COUNT", "3"),
        ("QUADGRAM_COUNT", "5"),
        ("MAX_UNIGRAM_ENTRY_COUNT", "9"),
        ("MAX_QUADGRAM_ENTRY_COUNT", "7"),
    ]);
    assert_eq!(policy.ngram_counts().count_array(), &[3, 0, 0, 0]);
    assert_eq!(
        policy.max_ngram_counts().count_array(),
        &[9, 30000, 30000, 0]
    );
    let plain = policy_of(&[]);
    assert_eq!(plain.ngram_counts().count_array(), &[0, 0, 0, 0]);
    assert_eq!(
        plain.max_ngram_counts().count_array(),
        &[10000, 30000, 30000, 0]
    );
}

#[test]
fn format_versions_are_numbered_as_the_file_numbers_them() {
    let cases = [
        (FormatVersion::Version2, -1, false),
        (FormatVersion::Version201, -1, false),
        (FormatVersion::Version202, 202, false),
        (FormatVersion::Version4OnlyForTesting, 399, false),
        (FormatVersion::Version402, 402, true),
        (FormatVersion::Version403, 403, true),
        (FormatVersion::Unknown, -1, false),
    ];
    for (version, number, supports_beginning_of_sentence) in cases {
        let policy = HeaderPolicy::with_current_time(&header_of(&[]), version, 0);
        assert_eq!(policy.format_version_number(), number, "{version:?}");
        assert_eq!(
            policy.supports_beginning_of_sentence(),
            supports_beginning_of_sentence,
            "{version:?}"
        );
    }
}

#[test]
fn a_decaying_dictionary_needs_its_historical_info() {
    let decaying = policy_of(&[("USES_FORGETTING_CURVE", "1")]);
    assert!(decaying.is_decaying_dict());
    assert!(!decaying.is_valid());
    assert!(!decaying.should_boost_exact_matches());
    let with_history = policy_of(&[("USES_FORGETTING_CURVE", "1"), ("HAS_HISTORICAL_INFO", "1")]);
    assert!(with_history.is_valid());
    let plain = policy_of(&[]);
    assert!(plain.is_valid());
    assert!(plain.should_boost_exact_matches());
}

#[test]
fn a_missing_date_is_the_given_time() {
    let missing = HeaderPolicy::with_current_time(&header_of(&[]), FormatVersion::Version202, 777);
    assert_eq!((missing.date(), missing.last_decayed_time()), (777, 777));
    let dated = HeaderPolicy::with_current_time(
        &header_of(&[("date", "5"), ("LAST_DECAYED_TIME", "6")]),
        FormatVersion::Version202,
        777,
    );
    assert_eq!((dated.date(), dated.last_decayed_time()), (5, 6));
}

#[test]
fn new_reads_the_clock_for_a_missing_date() {
    let before = clock_seconds();
    let policy = HeaderPolicy::new(&header_of(&[]), FormatVersion::Version202);
    let after = clock_seconds();
    assert!((before..=after).contains(&policy.date()));
}

#[test]
fn a_value_is_written_cut_and_terminated() {
    let policy = policy_of(&[("date", "2026")]);
    let mut out = [UNTOUCHED; 8];
    policy.read_header_value_or_question_mark("date", &mut out[..0]);
    assert_eq!(out, [UNTOUCHED; 8]);
    policy.read_header_value_or_question_mark("date", &mut out[..1]);
    assert_eq!(out[0], 0);
    let mut out = [UNTOUCHED; 8];
    policy.read_header_value_or_question_mark("nope", &mut out[..3]);
    assert_eq!(&out[..4], &[i32::from(b'?'), 0, UNTOUCHED, UNTOUCHED]);
    policy.read_header_value_or_question_mark("date", &mut out[..2]);
    assert_eq!(&out[..3], &[i32::from(b'2'), 0, UNTOUCHED]);
    policy.read_header_value_or_question_mark("date", &mut out[..6]);
    assert_eq!(
        &out[..6],
        &[
            i32::from(b'2'),
            i32::from(b'0'),
            i32::from(b'2'),
            i32::from(b'6'),
            0,
            UNTOUCHED
        ]
    );
}
