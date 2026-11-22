//! The live preview while the property editor is open: each value the stylesheet can hold is
//! shown on the keyboard before Apply, Cancel and Back show the draft without it, and Show
//! keyboard brings the keyboard up over the screen.
use super::{common::*, props::*};
use render::settings_ui::{
    catalogue::{languages::Lang, theme_editor::Part},
    theme_editor::{PROPERTY_EDIT, property_paint::KEYBOARD_ID},
};
use wire::settings_ui::theme_editor::effect;

/// The stylesheet the last preview carried, and the effects cleared.
fn previewed(a: &mut render::settings_ui::App) -> Option<String> {
    let json = last(a, effect::PREVIEW).map(|p| p[3].clone());
    a.effects.clear();
    json
}

#[test]
fn each_value_that_can_be_written_is_previewed_before_apply() {
    let (mut a, mut text) = named_app();
    let draft = a.theme_editor.json().unwrap();
    edit(&mut a, &mut text, 2, 0);
    a.effects.clear();
    pick(&mut a, &mut text, Part::PropEncoder, "Solid color");
    let shown = previewed(&mut a).unwrap();
    assert!(
        shown.contains("\"background\": \"rgba(0,0,0,1)\""),
        "{shown}"
    );
    assert!(shown.contains("--accent"), "the rest of the draft is there");
    assert_eq!(
        a.theme_editor.json().unwrap(),
        draft,
        "the draft is not changed"
    );
    // The picker takes nothing the stylesheet cannot hold, and its Cancel previews nothing.
    assert!(!picker_typed(&mut a, &mut text, "#00ff0"));
    assert!(tap(&mut a, &mut text, 6));
    assert_eq!(previewed(&mut a), None, "nothing before OK");
    pick_color(&mut a, &mut text, "#00ff00");
    let packet = last(&a, effect::PREVIEW).unwrap();
    assert_eq!(packet[1..3], [WORKSPACE, "0"]);
    assert!(packet[3].contains("\"background\": \"rgba(0,255,0,1)\""));
    a.effects.clear();
    // Cancel: the draft again.
    let cancel = lang_id(&a, Lang::Cancel);
    assert!(tap(&mut a, &mut text, cancel));
    assert_eq!(previewed(&mut a).unwrap(), draft);
}

#[test]
fn back_previews_the_draft_and_apply_previews_what_it_applied() {
    let (mut a, mut text) = named_app();
    let draft = a.theme_editor.json().unwrap();
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, "Border width", "Size (dp)");
    let shown = previewed(&mut a).unwrap();
    assert!(shown.contains("\"border-width\": \"0dp\""));
    a.back();
    assert_eq!(previewed(&mut a).unwrap(), draft);
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, "Border width", "Size (dp)");
    type_into(&mut a, &mut text, Part::PropValue, "3");
    a.effects.clear();
    apply(&mut a, &mut text);
    let applied = previewed(&mut a).unwrap();
    assert_eq!(applied, a.theme_editor.json().unwrap());
    assert!(applied.contains("\"border-width\": \"3dp\""));
}

#[test]
fn show_keyboard_focuses_the_hidden_editor_and_back_hides_it_first() {
    let (mut a, mut text) = named_app();
    edit(&mut a, &mut text, 2, 0);
    let row = control(&a, KEYBOARD_ID).unwrap();
    assert_eq!(row.label, "settings__theme_editor__show_keyboard");
    assert!(tap(&mut a, &mut text, KEYBOARD_ID));
    assert!(a.editor.focused);
    assert!(a.effects.contains(&vec!["editor".to_string()]));
    a.back();
    assert!(!a.editor.focused);
    assert_eq!(a.stack.last().unwrap(), PROPERTY_EDIT);
    a.back();
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
}
