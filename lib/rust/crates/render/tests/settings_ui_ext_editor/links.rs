//! A settings link arriving while an extension edit is open: the main activity is `singleTask`,
//! so a link from the keyboard or another app reaches the screen the editor is on.
use super::common::*;

const HOME: &str = "settings/home";
const THEMES: &str = "ext/list/EXT_THEME";
const SETUP: &str = "setup";

/// The editor with its first component deleted, its effects cleared.
fn changed() -> (App, Text) {
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, COMP_ID + 3));
    a.effects.clear();
    (a, text)
}

/// Improvement: a link to home replaced the whole back stack, so the editor was gone with its
/// draft unsaved and its workspace left open; the link now lands over the editor.
#[test]
fn a_link_to_home_mid_edit_lands_over_the_editor() {
    let (mut a, mut text) = changed();
    a.navigate(HOME);
    assert_eq!(a.stack, [HOME, EDIT, HOME]);
    assert!(
        last(&a, effect::CLOSE).is_none(),
        "the workspace stays open"
    );
    a.scene(&mut text);
    a.back();
    assert_eq!(top(&a), EDIT);
    assert_eq!(theme_ids(&a), ["night"]);
    assert!(a.extensions.editor.changed);
    a.back();
    assert!(a.unsaved_open(), "the change is still one to lose");
}

/// Setup is a root as home is, but never on the stack below the editor: a link to it lands over
/// the editor too, rather than replacing the stack as one root does another.
#[test]
fn a_link_to_setup_mid_edit_lands_over_the_editor() {
    let (mut a, mut text) = changed();
    a.navigate(SETUP);
    assert_eq!(a.stack, [HOME, EDIT, SETUP]);
    assert!(
        last(&a, effect::CLOSE).is_none(),
        "the workspace stays open"
    );
    a.scene(&mut text);
    a.back();
    assert_eq!(top(&a), EDIT);
    assert_eq!(theme_ids(&a), ["night"]);
    assert!(a.extensions.editor.changed);
}

#[test]
fn a_link_to_a_screen_below_the_editor_lands_over_it() {
    let (mut a, mut text) = fresh();
    a.navigate(THEMES);
    a.navigate(EDIT);
    a.set_ext_editor(&ready(MINE, MANIFEST, false)).unwrap();
    a.scene(&mut text);
    a.navigate(THEMES);
    assert_eq!(a.stack, [HOME, THEMES, EDIT, THEMES]);
    a.back();
    assert_eq!(top(&a), EDIT);
    assert_eq!(a.extensions.editor.workspace, WS);
}

#[test]
fn a_sheet_and_the_editor_itself_navigate_as_before() {
    let (mut a, mut text) = changed();
    assert!(tap(&mut a, &mut text, MANAGE_ID));
    assert_eq!(a.stack, [HOME, EDIT, META]);
    a.navigate(EDIT);
    assert_eq!(
        a.stack,
        [HOME, EDIT],
        "back to the editor, the sheet closed"
    );
    assert_eq!(theme_ids(&a), ["night"]);
    // The meta data opens afresh from the draft.
    assert!(tap(&mut a, &mut text, MANAGE_ID));
    assert_eq!(
        a.extensions.editor.meta_form.as_ref().unwrap().fields[0],
        MINE
    );
}

#[test]
fn another_editors_route_goes_back_to_the_open_edit() {
    let (mut a, _) = changed();
    a.navigate(HOME);
    a.navigate(CREATE);
    assert_eq!(a.stack, [HOME, EDIT]);
    assert!(
        last(&a, effect::OPEN).is_none(),
        "no second workspace is opened"
    );
    assert_eq!(a.extensions.editor.workspace, WS);
    assert_eq!(theme_ids(&a), ["night"]);
    a.navigate("ext/edit/other.themes");
    assert_eq!(a.stack, [HOME, EDIT]);
}

#[test]
fn with_no_workspace_open_a_link_replaces_the_stack_as_before() {
    let (mut a, _) = asked(EDIT, MINE);
    a.navigate(HOME);
    assert_eq!(a.stack, [HOME]);
    let (mut b, mut text) = changed();
    assert!(confirm(&mut b, &mut text));
    b.file_report(&done_report(job::EXT_SAVE, result::OK, WS, ""))
        .unwrap();
    b.navigate(THEMES);
    b.navigate(HOME);
    assert_eq!(b.stack, [HOME]);
}

#[test]
fn a_restored_edit_is_kept_before_kotlin_takes_it_up() {
    let (a, _) = changed();
    let mut b = App {
        config: a.config,
        strings: a.strings.clone(),
        ..App::default()
    };
    b.stack = a.stack.clone();
    b.scrolls = a.scrolls.clone();
    b.restore_ext_editor(&a.ext_editor_saved()).unwrap();
    b.extensions_restored();
    b.navigate(HOME);
    assert_eq!(b.stack, [HOME, EDIT, HOME]);
}
