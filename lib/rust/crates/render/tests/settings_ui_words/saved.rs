//! The user dictionary screens in the saved state: the draft, the search, the delete to undo and
//! a transfer survive a recreated activity; anything malformed is refused and changes nothing.
use crate::common::*;
use render::settings_ui::{
    App,
    items::{Deleted, Source, item_id},
    words::{FLORIS, FLORIS_EDIT, FLORIS_WORDS, transfer::Transfer},
};
use wire::settings_ui::words::{dictionary, effect};

#[test]
fn the_draft_search_undo_and_transfer_come_back() {
    let route = format!("{FLORIS_WORDS}/en_US");
    let (mut a, mut text) = app(&[FLORIS, &route], &sample());
    a.words.search = "zor".into();
    let slot = a.slot("1");
    assert!(tap(&mut a, &mut text, item_id(slot, 3)));
    a.words.transfer = Some(Transfer {
        dictionary: dictionary::FLORIS.into(),
        import: false,
    });
    let saved = a.words_saved();
    let mut b = App::default();
    b.restore_words(&saved).unwrap();
    assert_eq!(b.words.search, "zor");
    assert_eq!(b.list.deleted, a.list.deleted);
    assert_eq!(b.words.transfer, a.words.transfer);
    assert_eq!(b.words.draft, None);
    // In the editor, a checked draft as typed.
    let (mut a, mut text) = app(&[FLORIS, &route, &format!("{FLORIS_EDIT}/2")], &sample());
    type_into(&mut a, &mut text, 0, "typed");
    a.words.draft.as_mut().unwrap().checked = true;
    let mut b = App::default();
    b.restore_words(&a.words_saved()).unwrap();
    assert_eq!(b.words.draft, a.words.draft);
    // Restored on the editor's route, the app asks for the report again and keeps the draft.
    b.stack = a.stack.clone();
    b.words_restored();
    assert_eq!(
        b.effects,
        [vec![effect::LOAD.to_string(), dictionary::FLORIS.into()]]
    );
    assert_eq!(b.word_field_text(0), "typed");
}

#[test]
fn a_subtypes_undo_is_not_the_words_and_a_malformed_state_changes_nothing() {
    let (mut a, _) = app(&[FLORIS], &sample());
    a.list.deleted = Some(Deleted {
        source: Source::Subtypes,
        index: 0,
        value: "{}".into(),
        title: "Subtype".into(),
    });
    assert!(a.words_saved().contains("\"deleted\":null"));
    let good = App::default().words_saved();
    let mut b = App::default();
    b.words.search = "kept".into();
    for bad in [
        "",
        "[]",
        r#"{"draft":null,"search":1,"deleted":null,"transfer":null}"#,
        r#"{"draft":[["a","b","c"],null,true],"search":"","deleted":null,"transfer":null}"#,
        r#"{"draft":[["a","b","c","d"],null,"yes"],"search":"","deleted":null,"transfer":null}"#,
        r#"{"draft":[["a","b","c","d"],["1"],false],"search":"","deleted":null,"transfer":null}"#,
        r#"{"draft":null,"search":"","deleted":[0,"not a word","t"],"transfer":null}"#,
        r#"{"draft":null,"search":"","deleted":[-1,"[]","t"],"transfer":null}"#,
        r#"{"draft":null,"search":"","deleted":null,"transfer":["FLORIS"]}"#,
    ] {
        assert!(b.restore_words(bad).is_err(), "{bad}");
        assert_eq!(b.words.search, "kept");
    }
    b.restore_words(&good).unwrap();
    assert!(b.words.search.is_empty());
}
