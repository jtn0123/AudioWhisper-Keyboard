use super::*;
use crate::code_point::{decomposed_first, from_surrogates, push_utf16};
use crate::tables::Tables;
use crate::text::{
    code_point_at, code_point_before, code_point_count, code_points, first_code_point_len,
    to_int_or_null,
};

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn unit(c: char) -> u16 {
    u16::try_from(u32::from(c)).unwrap()
}

const BOTH: [&Tables; 2] = [&tables::jvm::TABLES, &tables::android::TABLES];

fn strictly_ascending(values: &[u32]) -> bool {
    values.is_sorted_by(|a, b| a < b)
}

#[test]
fn the_tables_are_well_formed() {
    for tables in BOTH {
        assert_eq!(tables.flag_starts[0], 0);
        assert!(strictly_ascending(tables.flag_starts));
        assert!(*tables.flag_starts.last().unwrap() <= code_point::MAX);
        assert_eq!(tables.flags.len(), tables.flag_starts.len());
        for (from, to) in [
            (tables.upper_from, tables.upper_to.len()),
            (tables.lower_from, tables.lower_to.len()),
            (tables.full_upper_from, tables.full_upper_to.len()),
            (tables.full_lower_from, tables.full_lower_to.len()),
            (tables.decomposed_from, tables.decomposed_to.len()),
        ] {
            assert!(strictly_ascending(from));
            assert_eq!(from.len(), to);
        }
        assert!(strictly_ascending(tables.digit_zeros));
    }
}

/// Every difference between Android's answers and the JVM's is one `tables` explains.
#[test]
fn android_differs_from_the_jvm_only_as_explained() {
    let (jvm, android) = (&tables::jvm::TABLES, &tables::android::TABLES);
    for cp in 0..=code_point::MAX {
        let (j, a) = (jvm.flags(cp), android.flags(cp));
        // Android's case is the JVM's less Other_Uppercase and Other_Lowercase: never more.
        assert_eq!(a & (UPPER | LOWER) & !j, 0, "{cp:04X}");
        // Android's \s is a superset of the JVM's ASCII.
        assert!(a & REGEX_SPACE != 0 || j & REGEX_SPACE == 0, "{cp:04X}");
        let whitespace_differs = (j ^ a) & SPACE != 0;
        assert_eq!(whitespace_differs, cp == 0x180E, "{cp:04X}");
        let rest = LETTER | DIGIT | LETTER_OR_DIGIT | TITLE | CASED | CASE_IGNORABLE;
        assert_eq!(j & rest, a & rest, "{cp:04X}");
    }
    assert_eq!(
        (jvm.upper_from, jvm.upper_to),
        (android.upper_from, android.upper_to)
    );
    assert_eq!(
        (jvm.lower_from, jvm.lower_to),
        (android.lower_from, android.lower_to)
    );
    assert_eq!(jvm.full_upper_from, android.full_upper_from);
    assert_eq!(jvm.full_upper_to, android.full_upper_to);
    assert_eq!(jvm.full_lower_from, android.full_lower_from);
    assert_eq!(jvm.full_lower_to, android.full_lower_to);
    assert_eq!(jvm.digit_zeros, android.digit_zeros);
    assert_eq!(jvm.decomposed_from, android.decomposed_from);
    assert_eq!(jvm.decomposed_to, android.decomposed_to);

    let flag = |tables: &Tables, c: char, flag: u16| tables.flags(u32::from(c)) & flag != 0;
    assert!(flag(jvm, 'ª', LOWER) && !flag(android, 'ª', LOWER));
    assert!(flag(jvm, 'Ⅻ', UPPER) && !flag(android, 'Ⅻ', UPPER));
    assert!(flag(android, 'a', LOWER) && flag(android, 'A', UPPER));
    for u in [0x85, 0xA0, 0x1680, 0x2007, 0x2028, 0x202F, 0x3000] {
        assert!(
            android.flags(u) & REGEX_SPACE != 0 && jvm.flags(u) & REGEX_SPACE == 0,
            "{u:04X}"
        );
    }
    assert_eq!(android.flags(0x1C) & REGEX_SPACE, 0);
    assert_eq!(android.flags(0x180E) & REGEX_SPACE, 0);
}

#[test]
fn regex_space_is_ascii_whitespace_on_the_jvm() {
    let ascii: Vec<u16> = (0..=u16::MAX).filter(|&u| is_regex_space(u)).collect();
    assert_eq!(ascii, [0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x20]);
}

#[test]
fn letters_are_category_l_only() {
    for c in ['a', 'Z', 'ß', 'ǅ', 'ʰ', '中', 'ª'] {
        assert!(is_letter(unit(c)), "{c}");
    }
    // Roman numerals are Nl, which Rust's is_alphabetic counts; Java doesn't.
    for c in ['Ⅻ', '1', '²', '\'', '’', ' ', '\u{0301}'] {
        assert!(!is_letter(unit(c)), "{c}");
    }
}

#[test]
fn digits_are_category_nd_only() {
    for c in ['0', '9', '٣', '９'] {
        assert!(is_digit(unit(c)) && is_letter_or_digit(unit(c)), "{c}");
    }
    for c in ['²', '½', 'Ⅻ', 'a'] {
        assert!(!is_digit(unit(c)), "{c}");
    }
    assert!(is_letter_or_digit(unit('a')));
    assert!(!is_letter_or_digit(unit('Ⅻ')));
}

#[test]
fn on_the_jvm_case_counts_other_uppercase_and_lowercase() {
    assert!(is_upper_case(unit('A')) && is_upper_case(unit('Ⓐ')) && is_upper_case(unit('Ⅻ')));
    assert!(is_lower_case(unit('a')) && is_lower_case(unit('ª')) && is_lower_case(unit('ʰ')));
    // Title case is neither.
    assert!(!is_upper_case(unit('ǅ')) && !is_lower_case(unit('ǅ')) && is_title_case(unit('ǅ')));
    assert!(!is_upper_case(unit('1')) && !is_lower_case(unit('1')) && !is_title_case(unit('A')));
}

#[test]
fn cased_and_case_ignorable_are_unicodes() {
    let flag = |c: char, flag: u16| TABLES.flags(u32::from(c)) & flag != 0;
    // ʰ is a modifier letter, so case-ignorable, and Other_Lowercase, so cased too.
    for c in ['a', 'A', 'ǅ', 'ª', 'Ⓐ', '𐐀', 'ʰ'] {
        assert!(flag(c, CASED), "{c}");
    }
    for c in ['1', '中', '\''] {
        assert!(!flag(c, CASED), "{c}");
    }
    for c in ['\'', '’', '.', ':', '·', '\u{0301}', '\u{00AD}', 'ʰ', '^'] {
        assert!(flag(c, CASE_IGNORABLE), "{c}");
    }
    for c in ['a', ' ', ',', '-', '"', '1'] {
        assert!(!flag(c, CASE_IGNORABLE), "{c}");
    }
}

#[test]
fn whitespace_is_kotlins() {
    for u in [
        0x20, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x1C, 0x1F, 0xA0, 0x1680, 0x2007, 0x202F, 0x3000,
        0x2028, 0x2029,
    ] {
        assert!(is_whitespace(u), "{u:04X}");
    }
    for u in [0x85, 0x200B, 0xFEFF, 0x00, 0x1B, 0x21] {
        assert!(!is_whitespace(u), "{u:04X}");
    }
}

#[test]
fn case_maps_one_unit_to_one() {
    assert_eq!(to_upper(unit('a')), unit('A'));
    assert_eq!(to_upper(unit('ß')), unit('ß'));
    assert_eq!(to_upper(unit('ǆ')), unit('Ǆ'));
    assert_eq!(to_upper(unit('ᾳ')), 0x1FBC);
    assert_eq!(to_upper(unit('ı')), unit('I'));
    assert_eq!(to_lower(unit('A')), unit('a'));
    assert_eq!(to_lower(unit('İ')), unit('i'));
    assert_eq!(to_lower(0x212A), unit('k')); // the Kelvin sign
    assert_eq!(to_lower(unit('Ⅻ')), unit('ⅻ'));
    assert_eq!(to_lower(0xD801), 0xD801); // a surrogate is left alone
    assert_eq!(to_upper(unit('1')), unit('1'));
}

/// So `to_unit`, Java's `(char)` cast, never cuts a BMP unit's case short.
#[test]
fn bmp_case_stays_in_the_bmp() {
    for tables in BOTH {
        for unit in 0..=u32::from(u16::MAX) {
            assert!(
                tables.upper(unit) <= 0xFFFF && tables.lower(unit) <= 0xFFFF,
                "{unit:04X}"
            );
        }
    }
}

#[test]
fn full_case_is_longer_only_where_java_says() {
    assert_eq!(TABLES.full_upper(u32::from('ß')), Some(&utf16("SS")[..]));
    assert_eq!(TABLES.full_upper(u32::from('ŉ')), Some(&utf16("ʼN")[..]));
    assert_eq!(TABLES.full_upper(u32::from('a')), None);
    assert_eq!(TABLES.full_lower_from, [0x0130]);
    assert_eq!(TABLES.full_lower(0x0130), Some(&utf16("i\u{0307}")[..]));
}

#[test]
fn code_points_answer_as_character_does() {
    assert!(code_point::is_digit(0x1D7CE)); // MATHEMATICAL BOLD DIGIT ZERO
    assert!(!code_point::is_digit(u32::from('²')));
    assert_eq!(code_point::digit(0x1D7D7), Some(9));
    assert_eq!(code_point::digit(u32::from('٣')), Some(3));
    assert_eq!(code_point::digit(u32::from('７')), Some(7));
    for c in ['a', '²', '½', '/', ':'] {
        assert_eq!(code_point::digit(u32::from(c)), None, "{c}");
    }
    assert_eq!(code_point::to_upper(0x10428), 0x10400); // Deseret
    assert_eq!(code_point::to_lower(0x10400), 0x10428);
    assert_eq!(code_point::to_upper(u32::from('ß')), u32::from('ß'));
}

#[test]
fn decomposes_to_the_base_letter() {
    for (unit, first) in [
        ('é', 'e'),
        ('ǖ', 'u'),
        ('Å', 'A'),
        ('a', 'a'),
        ('ß', 'ß'),
        ('가', '\u{1100}'),
        ('나', '\u{1102}'),
        ('힣', '\u{1112}'),
        ('\u{FA6C}', '\u{242EE}'),
    ] {
        assert_eq!(decomposed_first(unit as u16), u32::from(first), "{unit}");
    }
}

#[test]
fn surrogates_join_and_split() {
    assert_eq!(from_surrogates(0xD801, 0xDC00), 0x10400);
    for c in ['a', 'é', '中', '𐐀', '😀', '\u{10FFFF}'] {
        let mut out = Vec::new();
        push_utf16(&mut out, u32::from(c));
        assert_eq!(out, utf16(&c.to_string()), "{c}");
    }
}

#[test]
fn reads_code_points_as_java_does() {
    // a, 𐐀, b, then a lone low and a lone high surrogate.
    let lone = [0x61, 0xD801, 0xDC00, 0x62, 0xDC01, 0xD800];
    assert_eq!(code_point_at(&lone, 0), (0x61, 1));
    assert_eq!(code_point_at(&lone, 1), (0x10400, 2));
    assert_eq!(code_point_at(&lone, 2), (0xDC00, 1));
    assert_eq!(code_point_at(&lone, 4), (0xDC01, 1));
    assert_eq!(code_point_at(&lone, 5), (0xD800, 1));
    assert_eq!(code_point_before(&lone, 3), (0x10400, 2));
    assert_eq!(code_point_before(&lone, 2), (0xD801, 1));
    assert_eq!(code_point_before(&lone, 1), (0x61, 1));
    assert_eq!(code_point_before(&lone, 5), (0xDC01, 1));
    let all: Vec<_> = code_points(&lone).collect();
    assert_eq!(
        all,
        [(0, 0x61), (1, 0x10400), (3, 0x62), (4, 0xDC01), (5, 0xD800)]
    );
    assert_eq!(code_point_count(&lone), 5);
    assert_eq!(first_code_point_len(&lone[1..]), 2);
    assert_eq!(first_code_point_len(&lone[2..]), 1);
    assert_eq!(first_code_point_len(&[]), 0);
}

#[test]
fn parses_ints_as_kotlin_does() {
    for (text, value) in [
        ("42", Some(42)),
        ("-42", Some(-42)),
        ("+7", Some(7)),
        ("007", Some(7)),
        ("٣٤", Some(34)),
        ("2147483647", Some(i32::MAX)),
        ("-2147483648", Some(i32::MIN)),
        ("2147483648", None),
        ("-2147483649", None),
        ("99999999999999999999999", None),
        ("", None),
        ("-", None),
        ("+", None),
        ("--1", None),
        ("1a", None),
        (" 1", None),
        ("1 ", None),
        ("0x10", None),
        ("²", None),
    ] {
        assert_eq!(to_int_or_null(&utf16(text)), value, "{text:?}");
    }
}

#[test]
fn trims_kotlins_whitespace() {
    assert_eq!(
        trim(&utf16("\u{A0}\u{1C} hi there\t\u{3000}")),
        utf16("hi there")
    );
    assert_eq!(trim(&utf16("\u{85}hi\u{85}")), utf16("\u{85}hi\u{85}"));
    assert_eq!(trim(&utf16(" \t ")), utf16(""));
    assert_eq!(trim(&[]), [0u16; 0]);
}

#[test]
fn blank_is_empty_or_all_whitespace() {
    assert!(is_blank(&[]));
    assert!(is_blank(&utf16(" \u{A0}\n")));
    assert!(!is_blank(&utf16(" \u{85}")));
}

#[test]
fn compares_lowercase_with_ascii() {
    assert!(lower_equals_ascii(&utf16("New Line"), "new line"));
    assert!(lower_equals_ascii(
        &utf16("QUESTION MAR\u{212A}"),
        "question mark"
    ));
    assert!(!lower_equals_ascii(&utf16("İf"), "if"));
    assert!(!lower_equals_ascii(&utf16("new  line"), "new line"));
    assert!(!lower_equals_ascii(&utf16("newline"), "new line"));
    assert!(lower_equals_ascii(&[], ""));
}
