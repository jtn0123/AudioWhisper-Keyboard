//! The extension screens across a recreated activity: the route, an open delete question or
//! maintainer dialog come back, and the index is asked for again since Kotlin's side is new.
use super::common::*;
use boxes::STORE_ID;
use page::{DELETE_ID, MAINTAINER_ID};

/// A fresh app given `a`'s stack, scrolls and dialog, as `render_settings_state::restore` does.
fn recreated(a: &App) -> App {
    let mut b = App {
        config: a.config,
        strings: a.strings.clone(),
        ..App::default()
    };
    b.stack = a.stack.clone();
    b.scrolls = a.scrolls.clone();
    b.modal = a.modal.clone();
    b.extensions_restored();
    b
}

/// Improvement: FlorisConfirmDeleteDialog was held in a plain `remember`, so turning the phone
/// closed it; here it survives, and still deletes the extension it named.
#[test]
fn the_page_and_its_delete_question_survive_recreation() {
    let (mut a, mut text) = app(extensions::THEMES);
    a.navigate(&format!("ext/view/{MINE}"));
    assert!(tap(&mut a, &mut text, DELETE_ID));
    let mut b = recreated(&a);
    assert_eq!(b.effects, [["extensions"]]);
    b.effects.clear();
    b.set_extensions(&report()).unwrap();
    let items = painted(&mut b, &mut text);
    assert!(items.iter().any(|i| i.label == "Delete \"My themes\"?"));
    assert_eq!(b.title(), "My themes");
    assert!(tap(&mut b, &mut text, 7));
    assert_eq!(b.effects, [["extensionDelete", MINE]]);
    b.extension_done(&["delete".into(), MINE.into(), wire::flag::TRUE.into()])
        .unwrap();
    assert_eq!(b.stack.last().map(String::as_str), Some(extensions::THEMES));
}

#[test]
fn a_maintainers_dialog_survives_recreation() {
    let (mut a, mut text) = app(&format!("ext/view/{FLORIS}"));
    assert!(tap(&mut a, &mut text, MAINTAINER_ID));
    let mut b = recreated(&a);
    b.set_extensions(&report()).unwrap();
    b.scene(&mut text);
    assert_eq!(b.maintainer_dialog(), Some(0));
    assert_eq!(control(&b, 3000).label, "patrick@example.org");
    assert_eq!(control(&b, 3001).label, "https://patrickgold.dev");
}

#[test]
fn a_list_keeps_its_query_and_asks_for_the_index_again() {
    let (a, mut text) = app("ext/list/EXT_THEME?showUpdate=true");
    let mut b = recreated(&a);
    assert_eq!(b.effects, [["extensions"]]);
    b.set_extensions(&report()).unwrap();
    b.scene(&mut text);
    assert!(has(&b, STORE_ID));
    assert_eq!(b.title(), "Theme extensions");
}

#[test]
fn a_recreated_screen_elsewhere_asks_only_once_an_extension_screen_opens() {
    let (mut a, _) = app(extensions::HOME);
    a.back();
    let mut b = recreated(&a);
    assert!(b.effects.is_empty(), "{:?}", b.effects);
    b.navigate(extensions::HOME);
    b.navigate(extensions::THEMES);
    assert_eq!(b.effects, [["extensions"]]);
}
