use super::*;

fn offered(typed: Option<&str>, has_recents: bool) -> Vec<String> {
    let typed = typed.map(utf16);
    let chips = chips(typed.as_deref(), has_recents);
    chips
        .iter()
        .map(|it| String::from_utf16_lossy(it))
        .collect()
}

#[test]
fn the_categories_come_after_recents() {
    assert_eq!(offered(None, false), CATEGORIES);
    let with_recents = offered(None, true);
    assert_eq!(with_recents[0], RECENTS);
    assert_eq!(with_recents[1..], CATEGORIES);
}

#[test]
fn the_word_just_typed_goes_first() {
    let chips = offered(Some("pizza"), true);
    assert_eq!(chips[..3], ["pizza", RECENTS, TRENDING]);
    assert_eq!(chips.len(), CATEGORIES.len() + 2);
}

#[test]
fn a_chip_already_there_moves_first_instead() {
    let chips = offered(Some("congrats"), false);
    assert_eq!(chips[0], "Congrats");
    assert_eq!(chips.len(), CATEGORIES.len());
    assert_eq!(chips.iter().filter(|it| *it == "Congrats").count(), 1);
    assert_eq!(offered(Some("recents"), true)[0], RECENTS);
    // Without any recents, "recents" is just a word.
    assert_eq!(offered(Some("recents"), false)[..2], ["recents", TRENDING]);
}
