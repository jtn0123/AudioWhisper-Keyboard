use super::*;
use crate::utf16;

fn typed(query: &str, text: &str) -> String {
    String::from_utf16_lossy(&type_text(&utf16(query), &utf16(text)))
}

fn deleted(query: &str, count: i32) -> String {
    String::from_utf16_lossy(delete_last(&utf16(query), count))
}

fn words_deleted(query: &str, count: i32) -> String {
    String::from_utf16_lossy(delete_words(&utf16(query), count))
}

fn word(text: &str) -> Option<String> {
    last_word(&utf16(text)).map(String::from_utf16_lossy)
}

#[test]
fn typed_text_is_appended_up_to_the_limit() {
    assert_eq!(typed("cat", "s"), "cats");
    let long = "a".repeat(MAX_LENGTH);
    assert_eq!(typed(&long, "b"), long);
    // An emoji at the limit is cut in half, as Kotlin's take cuts it.
    let units = type_text(&utf16(&"a".repeat(MAX_LENGTH - 1)), &utf16("😀"));
    assert_eq!(units.len(), MAX_LENGTH);
    assert!(is_high_surrogate(units[MAX_LENGTH - 1]));
}

#[test]
fn glided_words_are_spaced_from_the_one_before() {
    let word =
        |query: &str, word: &str| String::from_utf16_lossy(&type_word(&utf16(query), &utf16(word)));
    assert_eq!(word("", "happy"), "happy");
    assert_eq!(word("happy", "dance"), "happy dance");
    assert_eq!(word("happy ", "dance"), "happy dance");
    // Only a plain space counts as one already there.
    assert_eq!(word("happy\t", "dance"), "happy\t dance");
}

#[test]
fn delete_takes_a_character_or_a_whole_emoji() {
    assert_eq!(deleted("cat", 1), "ca");
    assert_eq!(deleted("cat 😀", 1), "cat ");
    assert_eq!(deleted("", 1), "");
    assert_eq!(deleted("cat 😀", 3), "ca");
    assert_eq!(deleted("cat", 0), "cat");
    assert_eq!(deleted("cat", -4), "cat");
    assert_eq!(deleted("cat", 9), "");
    // A lone half of an emoji goes on its own.
    assert_eq!(delete_last(&[0x61, 0xDC00], 1), [0x61]);
    assert_eq!(delete_last(&[0xD83D, 0x61], 1), [0xD83D]);
}

#[test]
fn deleting_words_keeps_the_space_before_them() {
    assert_eq!(words_deleted("happy dance", 1), "happy ");
    assert_eq!(words_deleted("happy dance  ", 1), "happy ");
    assert_eq!(words_deleted("happy dance", 2), "");
    assert_eq!(words_deleted("", 1), "");
    assert_eq!(words_deleted("happy dance", 0), "happy dance");
    assert_eq!(words_deleted("one\u{00A0}two", 1), "one\u{00A0}");
}

#[test]
fn the_word_just_typed_is_offered() {
    assert_eq!(word("Big congrats! ").as_deref(), Some("congrats"));
    assert_eq!(word("Congrats").as_deref(), Some("Congrats"));
    assert_eq!(word("I don't").as_deref(), Some("don't"));
    assert_eq!(word("at the café.").as_deref(), Some("café"));
    assert_eq!(word("'quoted'").as_deref(), Some("quoted"));
    assert_eq!(
        word(&"x".repeat(30)).as_deref(),
        Some("x".repeat(30).as_str())
    );
}

#[test]
fn nothing_is_offered_without_a_word_at_the_end() {
    for text in [
        "", "   ", "a", "call 555", "see 😀", "''a''", "it's-", "ok:)",
    ] {
        assert_eq!(word(text), None, "{text:?}");
    }
    assert_eq!(word(&"x".repeat(31)), None);
}
