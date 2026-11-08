use super::{android, jvm};
use crate::tables::{self, Tables};
use crate::{equals_ignore_case, lowercase, starts_with_ignore_case, uppercase};

const JVM: &Tables = &tables::jvm::TABLES;
const ANDROID: &Tables = &tables::android::TABLES;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn lower_on(tables: &Tables, lowercase: fn(&Tables, &[u16]) -> Vec<u16>, text: &str) -> String {
    String::from_utf16(&lowercase(tables, &utf16(text))).unwrap()
}

fn matches_on(
    tables: &Tables,
    region_matches: fn(&Tables, &[u16], &[u16], usize) -> bool,
    a: &[u16],
    b: &[u16],
) -> bool {
    a.len() == b.len() && region_matches(tables, a, b, a.len())
}

#[test]
fn upper_cases_code_point_by_code_point() {
    let upper = |text: &str| String::from_utf16(&uppercase(&utf16(text))).unwrap();
    assert_eq!(upper("straße"), "STRASSE");
    assert_eq!(upper("ǆemal"), "ǄEMAL");
    assert_eq!(upper("ŉ"), "ʼN");
    assert_eq!(upper("𐐨x"), "𐐀X");
    assert_eq!(upper("ﬁ"), "FI");
    // A lone surrogate is left as it is.
    assert_eq!(uppercase(&[0xD801, 0x61]), [0xD801, 0x41]);
    assert_eq!(uppercase(&[]), [0u16; 0]);
}

#[test]
fn both_lower_case_alike_but_for_sigma() {
    for (tables, lowercase) in [
        (JVM, jvm::lowercase as fn(&Tables, &[u16]) -> Vec<u16>),
        (ANDROID, android::lowercase),
    ] {
        let lower = |text| lower_on(tables, lowercase, text);
        assert_eq!(lower("Hello WORLD"), "hello world");
        assert_eq!(lower("İstanbul"), "i\u{0307}stanbul");
        assert_eq!(lower("ΟΔΟΣ"), "οδος");
        assert_eq!(lower("ΟΔΟΣ ΟΔΟΣ."), "οδος οδος.");
        assert_eq!(lower("Σ"), "σ");
        assert_eq!(lower("ΣΑ"), "σα");
        assert_eq!(lower("1Σ"), "1σ");
        assert_eq!(lower("𐐀Σ"), "𐐨ς");
        assert_eq!(lower("ǅ"), "ǆ");
        assert_eq!(lower("\u{212A}"), "k");
        assert_eq!(lowercase(tables, &[0xDC00, 0x41]), [0xDC00, 0x61]);
    }
}

/// Char by char, Android sees only the chars either side of Σ; ICU, and so Android with İ or a
/// surrogate in the text, and the JVM here, look past case-ignorable ones.
#[test]
fn android_ends_a_word_at_sigma_by_the_chars_beside_it() {
    assert_eq!(lower_on(ANDROID, android::lowercase, "ΑΣ'Α"), "ας'α");
    assert_eq!(lower_on(JVM, jvm::lowercase, "ΑΣ'Α"), "ασ'α");
    assert_eq!(lower_on(ANDROID, android::lowercase, "Α'Σ"), "α'σ");
    assert_eq!(lower_on(JVM, jvm::lowercase, "Α'Σ"), "α'ς");
    // İ or a surrogate anywhere sends Android's whole string to ICU.
    assert_eq!(
        lower_on(ANDROID, android::lowercase, "ΑΣ'Α İ"),
        "ασ'α i\u{0307}"
    );
    assert_eq!(lower_on(ANDROID, android::lowercase, "Α'Σ 𐐀"), "α'ς 𐐨");
    // Title case counts as cased for both.
    assert_eq!(lower_on(ANDROID, android::lowercase, "ǅΣ"), "ǆς");
}

#[test]
fn icu_looks_past_marks_either_side() {
    let lower = |text| lower_on(JVM, jvm::lowercase, text);
    assert_eq!(lower("Α\u{0301}Σ"), "α\u{0301}ς");
    assert_eq!(lower("ΑΣ\u{0301}"), "ας\u{0301}");
    assert_eq!(lower("ΑΣ\u{0301}Α"), "ασ\u{0301}α");
    assert_eq!(lower("'Σ'"), "'σ'");
    // A surrogate pair is read as one code point both ways.
    assert_eq!(lower("ΑΣ𐐀"), "ασ𐐨");
    // U+1D167, a nonspacing mark, is looked past; U+1D165, a spacing one, isn't.
    assert_eq!(lower("ΑΣ\u{1D167}"), "ας\u{1D167}");
    assert_eq!(lower("Α\u{1D167}Σ"), "α\u{1D167}ς");
    assert_eq!(lower("ΑΣ\u{1D167}Α"), "ασ\u{1D167}α");
    assert_eq!(lower("Α\u{1D165}Σ"), "α\u{1D165}σ");
}

#[test]
fn both_compare_ignoring_case_alike_within_the_bmp() {
    for (tables, region_matches) in [
        (
            JVM,
            jvm::region_matches as fn(&Tables, &[u16], &[u16], usize) -> bool,
        ),
        (ANDROID, android::region_matches),
    ] {
        let same = |a: &str, b: &str| matches_on(tables, region_matches, &utf16(a), &utf16(b));
        assert!(same("Hello", "hELLO"));
        assert!(same("ǅ", "ǆ") && same("ǅ", "Ǆ"));
        assert!(same("\u{212A}", "k"));
        assert!(same("µ", "μ")); // the micro sign upper-cases to Μ
        assert!(same("ς", "Σ") && same("ς", "σ"));
        assert!(same("ı", "I") && same("ı", "i"));
        assert!(!same("ß", "SS"));
        assert!(!same("a", "b"));
        assert!(same("", ""));
    }
}

/// Android compares char by char, so the halves of a supplementary letter never match another
/// case; the JVM joins them.
#[test]
fn only_the_jvm_compares_surrogate_pairs_ignoring_case() {
    let (upper, lower) = (utf16("a𐐀"), utf16("A𐐨"));
    assert!(matches_on(JVM, jvm::region_matches, &upper, &lower));
    assert!(!matches_on(
        ANDROID,
        android::region_matches,
        &upper,
        &lower
    ));
    for (tables, region_matches) in [
        (
            JVM,
            jvm::region_matches as fn(&Tables, &[u16], &[u16], usize) -> bool,
        ),
        (ANDROID, android::region_matches),
    ] {
        assert!(matches_on(
            tables,
            region_matches,
            &[0xD801, 0x61],
            &[0xD801, 0x41]
        ));
        assert!(!matches_on(
            tables,
            region_matches,
            &[0xD801, 0x61],
            &[0xD801, 0x62]
        ));
        assert!(!matches_on(tables, region_matches, &[0xDC00], &[0xDC28]));
    }
}

#[test]
fn the_public_functions_follow_the_platform() {
    assert_eq!(
        String::from_utf16(&lowercase(&utf16("ΑΣ'Α"))).unwrap(),
        "ασ'α"
    );
    assert!(equals_ignore_case(&utf16("Straße"), &utf16("STRAßE")));
    assert!(!equals_ignore_case(&utf16("Straße"), &utf16("STRASSE")));
    assert!(starts_with_ignore_case(&utf16("Hello"), &utf16("hE")));
    assert!(starts_with_ignore_case(&utf16("Hello"), &utf16("")));
    assert!(!starts_with_ignore_case(&utf16("He"), &utf16("Hello")));
    assert!(!starts_with_ignore_case(&utf16("Hello"), &utf16("Ha")));
}
