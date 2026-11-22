//! The draft across a saved state: unsaved rules and meta, and an open rule, meta or property
//! editor with what it holds, come back once Kotlin reports the component again.
use super::common::*;
use render::settings_ui::{
    App,
    catalogue::{languages::Lang, theme_editor::Part},
    theme_editor::{
        META, PROPERTY_ADD, PROPERTY_EDIT, RULE_EDIT,
        paint::{ADD_ID, rule_id},
    },
};
use wire::settings_ui::theme_editor::effect;

/// A fresh app given `a`'s saved stack and theme editor part, as recreation restores them.
fn recreated(a: &App) -> App {
    let saved = a.theme_editor_saved();
    let mut b = App {
        config: a.config,
        strings: a.strings.clone(),
        ..App::default()
    };
    b.restore_theme_editor(&saved).unwrap();
    let draft = b.theme_editor.restored.take();
    b.stack = a.stack.clone();
    b.scrolls = a.scrolls.clone();
    b.theme_editor_restored(draft);
    b
}

#[test]
fn unsaved_rules_and_an_open_rule_editor_survive_recreation() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, rule_id(2, 65)));
    assert!(tap(&mut a, &mut text, rule_id(3, 2)));
    type_into(&mut a, &mut text, Part::Attribute(0), "10, 13");
    assert!(tap(&mut a, &mut text, 400_071));
    let (rules, form) = (a.theme_editor.rules.clone(), a.theme_editor.form.clone());
    let mut b = recreated(&a);
    assert_eq!(b.stack.last().unwrap(), RULE_EDIT);
    let load = vec![effect::LOAD.to_string(), WORKSPACE.into(), "0".into()];
    assert_eq!(b.effects, [load]);
    // Kotlin's report brings the stylesheet as the workspace has it; the draft replaces it.
    b.set_theme_editor(&loaded(SHEET)).unwrap();
    assert_eq!(b.theme_editor.rules, rules);
    assert_eq!(b.theme_editor.form, form);
    assert!(b.theme_editor.changed());
    assert!(last(&b, effect::PREVIEW).is_some());
    // Back on the rules, the draft still asks before it is dropped.
    b.back();
    assert!(b.stack.last().unwrap().starts_with("settings/theme/edit/"));
    assert!(b.theme_holds_back());
}

#[test]
fn the_meta_editor_and_the_meta_draft_survive_recreation() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, 400_013));
    type_into(&mut a, &mut text, Part::MetaField(0), "Dusk");
    type_into(&mut a, &mut text, Part::MetaField(2), "dusk");
    type_into(&mut a, &mut text, Part::MetaField(3), "sheets/d.json");
    let palette = part_id(&a, Part::MetaChoice(0));
    assert!(tap(&mut a, &mut text, palette));
    assert!(tap(&mut a, &mut text, 3000));
    let form = a.theme_editor.meta_form.clone();
    let mut b = recreated(&a);
    b.set_theme_editor(&loaded(SHEET)).unwrap();
    assert_eq!(b.stack.last().unwrap(), META);
    assert_eq!(b.theme_editor.meta_form, form);
    assert_eq!(form.unwrap().flags[0], "TonalSpot");
    assert_eq!(b.theme_editor.meta.label, "Sunny");
    assert!(!b.theme_editor.changed());
    // Applied, the draft meta comes back over the report's.
    let apply = lang_id(&b, Lang::Save);
    assert!(tap(&mut b, &mut text, apply));
    let meta = b.theme_editor.meta.clone();
    let mut c = recreated(&b);
    c.set_theme_editor(&loaded(SHEET)).unwrap();
    assert_eq!(c.theme_editor.meta, meta);
    assert_eq!(
        (meta.id.as_str(), meta.path.as_str()),
        ("dusk", "sheets/d.json")
    );
    assert!(c.theme_editor.changed());
}

#[test]
fn an_editor_saved_before_its_stylesheet_loaded_comes_back_on_the_rules_only() {
    let mut a = opened();
    a.stack.push(RULE_EDIT.into());
    a.scrolls.push(0.0);
    assert_eq!(a.theme_editor_saved(), "");
    let mut b = recreated(&a);
    assert_eq!(b.stack, [route(0)]);
    assert_eq!(b.scrolls.len(), 1);
    b.set_theme_editor(&loaded(SHEET)).unwrap();
    let mut text = text();
    b.scene(&mut text);
    assert!(control(&b, ADD_ID).is_some());
    assert!(!b.theme_editor.changed());
}

#[test]
fn a_draft_for_another_component_or_a_malformed_part_is_not_used() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, rule_id(2, 65)));
    let saved = a.theme_editor_saved();
    let mut b = App::default();
    b.restore_theme_editor(&saved).unwrap();
    let draft = b.theme_editor.restored.take();
    b.stack = vec![route(1)];
    b.scrolls = vec![0.0];
    b.theme_editor_restored(draft);
    assert!(b.theme_editor.restored.is_none());
    let short = saved.rsplit_once('\u{1e}').unwrap().0;
    for broken in [short, "x", &saved.replace("Sunny", "Sunny\u{1f}")] {
        let mut c = App::default();
        assert!(c.restore_theme_editor(broken).is_err(), "{}", broken.len());
        assert!(c.theme_editor.restored.is_none());
    }
}

#[test]
fn an_open_property_editor_survives_recreation_with_what_it_holds() {
    let (mut a, mut text) = app(SHEET);
    super::props::named(&mut a);
    assert!(tap(&mut a, &mut text, rule_id(3, 4)));
    type_into(&mut a, &mut text, Part::PropSearch, "border");
    super::props::choose(&mut a, &mut text, "Border width", "Size (dp)");
    type_into(&mut a, &mut text, Part::PropValue, "x");
    super::props::apply(&mut a, &mut text);
    let prop = a.theme_editor.prop.clone();
    assert!(
        prop.as_ref()
            .is_some_and(|f| f.checked && f.search == "border")
    );
    let mut b = recreated(&a);
    assert_eq!(b.stack.last().unwrap(), PROPERTY_ADD);
    b.set_theme_editor(&loaded(SHEET)).unwrap();
    assert_eq!(b.theme_editor.prop, prop);
    b.scene(&mut text);
    let value = part_id(&b, Part::PropValue);
    assert_eq!(
        control(&b, value).unwrap().summary,
        "ext__validation__enter_valid_number"
    );
    type_into(&mut b, &mut text, Part::PropValue, "4");
    super::props::apply(&mut b, &mut text);
    assert!(super::props::writes(&b, "border-width", "4dp"));
    // Editing, with the encoder changed and a refusal shown.
    super::props::edit(&mut b, &mut text, 2, 0);
    super::props::pick(&mut b, &mut text, Part::PropEncoder, "Solid color");
    type_into(&mut b, &mut text, Part::PropValue, "#123456");
    let prop = b.theme_editor.prop.clone();
    let mut c = recreated(&b);
    c.set_theme_editor(&loaded(SHEET)).unwrap();
    assert_eq!(c.stack.last().unwrap(), PROPERTY_EDIT);
    assert_eq!(c.theme_editor.prop, prop);
    // The preview shows the form's value once the draft is back.
    let shown = last(&c, effect::PREVIEW).unwrap();
    assert!(shown[3].contains("\"background\": \"rgba(18,52,86,1)\""));
}

#[test]
fn a_property_editor_saved_without_its_form_comes_back_on_the_rules() {
    let (mut a, _) = app(SHEET);
    a.stack.push(PROPERTY_EDIT.into());
    a.scrolls.push(0.0);
    let mut b = recreated(&a);
    assert_eq!(b.stack, [route(0)]);
    b.set_theme_editor(&loaded(SHEET)).unwrap();
    assert!(b.theme_editor.prop.is_none());
    // A form naming an encoder or a message that does not exist is malformed.
    let (mut a, mut text) = app(SHEET);
    super::props::edit(&mut a, &mut text, 2, 0);
    let saved = a.theme_editor_saved();
    let mut b = App::default();
    let broken = saved.replace("\u{1f}27\u{1f}", "\u{1f}99\u{1f}");
    assert_ne!(broken, saved);
    assert!(b.restore_theme_editor(&broken).is_err());
}
