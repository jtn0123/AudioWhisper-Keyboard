//! The user dictionary's changes and lookups, as its Room table made them.

use crate::words::{Taken, Word, Words};

use super::golden::units;

fn word(id: i64, text: &str, locale: Option<&str>) -> Word {
    Word {
        id,
        word: units(text),
        frequency: 100,
        locale: locale.map(units),
        shortcut: None,
    }
}

fn ids(words: &[&Word]) -> Vec<i64> {
    words.iter().map(|word| word.id).collect()
}

#[test]
fn an_id_of_zero_takes_the_next_and_others_are_kept() {
    let mut words = Words::default();
    assert_eq!(words.insert(word(0, "a", None)), Ok(1));
    assert_eq!(words.insert(word(10, "b", None)), Ok(10));
    assert_eq!(words.insert(word(5, "c", None)), Ok(5));
    assert_eq!(words.insert(word(0, "d", None)), Ok(11));
    assert_eq!(words.insert(word(5, "e", None)), Err(Taken(5)));
    assert_eq!(
        words.insert(word(i64::MAX, "f", None)),
        Err(Taken(i64::MAX))
    );
    assert_eq!(
        words.all().iter().map(|w| w.id).collect::<Vec<_>>(),
        [1, 5, 10, 11]
    );
    words.delete(11);
    assert_eq!(
        words.insert(word(0, "g", None)),
        Ok(12),
        "an id is never given twice"
    );
}

#[test]
fn exact_lookups_match_the_word_and_the_locale_exactly() {
    let mut words = Words::default();
    for (text, locale) in [
        ("w", None),
        ("w", Some("en")),
        ("w", Some("")),
        ("W", None),
        ("w", Some("en")),
    ] {
        words.insert(word(0, text, locale)).expect("an id");
    }
    let en = units("en");
    assert_eq!(ids(&words.exact(&units("w"), None)), [1]);
    assert_eq!(ids(&words.exact(&units("w"), Some(&en))), [2, 5]);
    assert_eq!(ids(&words.exact(&units("w"), Some(&[]))), [3]);
    assert!(words.exact(&units("x"), None).is_empty());
}

#[test]
fn updates_and_deletes_go_by_id() {
    let mut words = Words::default();
    words.insert(word(0, "a", None)).expect("an id");
    words.update(Word {
        frequency: 7,
        ..word(1, "b", Some("fr"))
    });
    assert_eq!(words.all()[0].frequency, 7);
    assert_eq!(words.all()[0].word, units("b"));
    let before = words.clone();
    words.update(word(9, "c", None));
    words.delete(9);
    assert!(words == before, "unknown ids change nothing");
    words.delete(1);
    assert!(words.all().is_empty());
}
