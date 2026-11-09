use super::*;
use crate::utf16;

const IDS: [&str; 2] = ["recently_used", "smileys"];

fn parse(text: &str) -> EmojiData {
    let ids: Vec<Vec<u16>> = IDS.iter().map(|id| utf16(id)).collect();
    let ids: Vec<&[u16]> = ids.iter().map(Vec::as_slice).collect();
    EmojiData::parse(utf16(text), &ids)
}

fn query(composing: &str, prefix: &str, min_length: i32) -> Option<String> {
    let composing = utf16(composing);
    validate_query(&composing, &utf16(prefix), min_length).map(String::from_utf16_lossy)
}

#[test]
fn a_query_follows_the_prefix_and_is_letters() {
    assert_eq!(query(":smi", ":", 3).as_deref(), Some("smi"));
    assert_eq!(query(":sm", ":", 3), None);
    assert_eq!(query("smi", ":", 3), None);
    assert_eq!(query(":smi1", ":", 3), None);
    assert_eq!(query(":smí", ":", 3), None);
    assert_eq!(query(":smi\n", ":", 3), None);
    assert_eq!(query("smile", "", 3).as_deref(), Some("smile"));
    assert_eq!(query("", "", 0).as_deref(), Some(""));
    assert_eq!(query(":", ":", 0).as_deref(), Some(""));
    assert_eq!(query(":", ":", -5).as_deref(), Some(""));
    // The minimum wraps: i32::MAX plus the colon is negative.
    assert_eq!(query(":a", ":", i32::MAX).as_deref(), Some("a"));
    assert_eq!(query(":a", ":", i32::MAX - 1), None);
}

#[test]
fn contains_ignores_case_as_java_does() {
    let has = |text: &str, other: &str| contains_ignore_case(&utf16(text), &utf16(other));
    assert!(has("Grinning Face", "face"));
    assert!(has("grinning", ""));
    assert!(has("", ""));
    assert!(!has("", "a"));
    assert!(!has("fac", "face"));
    // Dotless ı's upper case is I, and the Kelvin sign's lower case is k.
    assert!(has("ıce", "ICE"));
    assert!(has("\u{212A}iss", "kiss"));
    assert!(has("ſmile", "SMILE"));
}

#[test]
fn java_compare_orders_nan_last_and_zeros() {
    let negative_nan = f64::from_bits(0xfff8_0000_0000_0000);
    assert_eq!(java_compare(negative_nan, f64::INFINITY), Ordering::Greater);
    assert_eq!(java_compare(f64::NAN, negative_nan), Ordering::Equal);
    assert_eq!(java_compare(-0.0, 0.0), Ordering::Less);
    assert_eq!(java_compare(0.3, 0.7), Ordering::Less);
    assert_eq!(java_compare(0.7, 0.7), Ordering::Equal);
}

#[test]
fn the_best_names_come_first_and_ties_keep_their_order() {
    let data = parse(
        "[smileys]\nA;smile;\nB;smiling face;happy\nC;cat;smile|grin\nD;smiley;\nE;grin;\nF;smile;\n",
    );
    let found = suggest(&data, 0, &utf16("smil"), 10);
    // smile 4/5 * .7 = .56 twice (A then F), smiley 4/6 * .7, cat's keyword .3, smiling face
    // 4/12 * .7.
    assert_eq!(found, [0, 5, 3, 2, 1]);
    assert_eq!(suggest(&data, 0, &utf16("smil"), 2), [0, 5]);
    assert_eq!(suggest(&data, 0, &utf16("smil"), 0), Vec::<usize>::new());
    assert_eq!(suggest(&data, 0, &utf16("zzz"), 5), Vec::<usize>::new());
    let score = score(&data, &data.entries[1], &utf16("FACE"));
    assert_eq!(score, 4.0 / 12.0 * 0.7);
}

#[test]
fn an_empty_name_takes_a_place_and_gives_nothing() {
    // Root has no names: an empty query scores NaN for each, which sorts first and is dropped.
    let data = parse("[smileys]\nA;;\nB;b;\nC;;x\n");
    assert!(score(&data, &data.entries[0], &[]).is_nan());
    assert_eq!(suggest(&data, 0, &[], 2), Vec::<usize>::new());
    assert_eq!(suggest(&data, 0, &[], 3), [1]);
    assert_eq!(suggest(&data, 0, &utf16("x"), 4), [2]);
}

#[test]
fn each_tone_has_its_own_list() {
    let data = parse("[smileys]\n👋;wave;\n\t👋🏻;wave;\n\t👋🏿;wave;\n");
    assert_eq!(suggest(&data, 0, &utf16("wave"), 4), [0]);
    assert_eq!(suggest(&data, 1, &utf16("wave"), 4), [0]);
    assert_eq!(data.by_skin_tone[1], [1]);
    assert_eq!(suggest(&data, 2, &utf16("wave"), 4), Vec::<usize>::new());
}
