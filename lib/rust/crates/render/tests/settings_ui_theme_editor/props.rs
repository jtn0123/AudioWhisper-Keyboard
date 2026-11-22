//! What the property editor tests share: opening the screen from a rule's Add or a property,
//! choosing in its dropdowns by label, Apply, what the stylesheet writes, and the focus order.
use super::common::*;
use render::settings_ui::{
    App,
    catalogue::{languages::Lang, theme_editor::Part},
    theme_editor::{PROPERTY_ADD, PROPERTY_EDIT, paint::rule_id},
};
use render::text::Text;

/// The labels the tests read in place of their string keys.
pub const PLACEHOLDER: &str = "- select -";
pub fn named(a: &mut App) {
    for (k, v) in [
        ("general__select_dropdown_value_placeholder", PLACEHOLDER),
        ("snygg__property_name__font_size", "Text size"),
        ("snygg__property_name__border_width", "Border width"),
        ("snygg__property_value__solid_color", "Solid color"),
        ("snygg__property_value__dp_size", "Size (dp)"),
        ("snygg__property_value__defined_var", "Variable"),
    ] {
        a.strings.insert(k.into(), v.into());
    }
}
/// The stylesheet with labels for the tests, painted.
pub fn named_app() -> (App, Text) {
    let (mut a, mut text) = app(SHEET);
    named(&mut a);
    a.scene(&mut text);
    (a, text)
}
/// Rule `r`'s Add property: the screen adding.
pub fn add_to(a: &mut App, text: &mut Text, r: usize) {
    assert!(tap(a, text, rule_id(r, 4)), "rule {r} has no Add");
    assert_eq!(a.stack.last().unwrap(), PROPERTY_ADD);
}
/// Property `n` of rule `r` as the list counts them across sets: the screen editing it.
pub fn edit(a: &mut App, text: &mut Text, r: usize, n: usize) {
    assert!(tap(a, text, rule_id(r, 64 + 2 * n)), "rule {r} has no {n}");
    assert_eq!(a.stack.last().unwrap(), PROPERTY_EDIT);
}
/// The labels dropdown `part` lists, closed again.
pub fn listed(a: &mut App, text: &mut Text, part: Part) -> Vec<String> {
    let id = part_id(a, part);
    assert!(tap(a, text, id), "{part:?} does not open");
    let labels = choices(a);
    a.back();
    a.scene(text);
    labels
}
/// Dropdown `part` opened and its entry labelled `label` tapped.
pub fn pick(a: &mut App, text: &mut Text, part: Part, label: &str) {
    let id = part_id(a, part);
    assert!(tap(a, text, id), "{part:?} does not open");
    let labels = choices(a);
    let at = labels.iter().position(|l| l == label);
    let at = at.unwrap_or_else(|| panic!("{label} is not in {labels:?}"));
    assert!(tap(a, text, 3000 + at as i32));
    assert!(a.modal.is_none());
}
/// The name and the encoder chosen by label.
pub fn choose(a: &mut App, text: &mut Text, name: &str, encoder: &str) {
    pick(a, text, Part::PropName, name);
    pick(a, text, Part::PropEncoder, encoder);
}
/// The bottom bar's Add or Apply.
pub fn apply(a: &mut App, text: &mut Text) {
    let id = lang_id(a, Lang::Save);
    assert!(tap(a, text, id));
}
/// Whether the stylesheet writes property `name` as `value`.
pub fn writes(a: &App, name: &str, value: &str) -> bool {
    let json = a.theme_editor.json().unwrap();
    json.contains(&format!("\"{name}\": \"{value}\""))
}
/// The static colour's picker opened from the value row, `typed` entered in its field and the
/// field's Apply changes tapped: whether the picker took it.
pub fn picker_typed(a: &mut App, text: &mut Text, typed: &str) -> bool {
    assert!(tap(a, text, part_id(a, Part::PropValue)), "no picker");
    assert!(a.color_picker().is_some_and(|p| p.alpha.is_some()));
    assert!(tap(a, text, 7140), "no Edit");
    assert!(tap(a, text, 7000), "no field");
    a.editor.text = typed.encode_utf16().collect();
    a.editor.anchor = a.editor.text.len();
    a.editor.focus = a.editor.text.len();
    a.sync_field();
    assert!(tap(a, text, 7141), "no Apply changes");
    !a.color_picker().unwrap().error
}
/// `typed` picked as the static colour: entered in the picker, then OK.
pub fn pick_color(a: &mut App, text: &mut Text, typed: &str) {
    assert!(picker_typed(a, text, typed), "the picker refused {typed}");
    assert!(tap(a, text, 7), "no OK");
    assert!(a.modal.is_none());
}
/// The screen's field `part`: its text and its message.
pub fn field(a: &App, part: Part) -> (String, String) {
    let c = control(a, part_id(a, part)).unwrap();
    let text = a.line_text(c.id).unwrap_or_default();
    (text, c.summary)
}
/// The focus order from the first focusable control round to it.
pub fn focus_order(a: &mut App) -> Vec<i32> {
    a.focus = -1;
    a.focus_next(false);
    let first = a.focus;
    let mut out = vec![first];
    loop {
        a.focus_next(false);
        if a.focus == first {
            return out;
        }
        out.push(a.focus);
    }
}
