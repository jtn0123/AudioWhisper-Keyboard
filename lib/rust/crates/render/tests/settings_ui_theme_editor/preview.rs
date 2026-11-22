//! Show keyboard on the property editor: it brings the keyboard, which shows the draft, up over
//! the screen, and Back hides it before leaving.
use super::{common::*, props::*};
use render::settings_ui::theme_editor::{PROPERTY_EDIT, property_paint::KEYBOARD_ID};

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
