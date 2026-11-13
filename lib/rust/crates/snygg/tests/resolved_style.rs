//! Frozen results exported by ReferenceSnyggTheme through the prototype's query path.
//! The JVM test checks this fixture too; changes to the Rust resolver cannot rewrite it.
use snygg::{
    grammar::{self, Rule},
    resolve::{CompiledTheme, Properties, Query},
    text,
    value::{Encoder, Unit, Value},
};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}
fn ask(theme: &CompiledTheme, name: &str, selector: usize, parent: &Properties) -> Properties {
    let attrs = vec![
        (text("mode"), text("characters")),
        (text("shiftstate"), text("unshifted")),
        (text("code"), text("97")),
    ];
    theme.resolve(
        &Query {
            element: &text(name),
            attributes: &attrs,
            selector,
        },
        parent,
        1.0,
    )
}
#[test]
fn bundled_light_and_dark_equal_the_kotlin_prototype_handoff() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("renderer-expected.json")).unwrap();
    for (path, expected) in fixture.as_object().unwrap() {
        let theme =
            CompiledTheme::from_json(&std::fs::read_to_string(root().join(path)).unwrap()).unwrap();
        let initial = vec![(text("font-size"), Value::Dimension(Unit::Sp, 14.0))];
        let parent = ask(&theme, "window", 0, &ask(&theme, "root", 0, &initial));
        for (label, name, selector) in [
            ("key", "key", 0),
            ("pressed-key", "key", 1),
            ("candidate", "smartbar-candidate-word", 0),
            ("popup", "key-popup-box", 0),
            ("focus", "key-popup-element", 2),
            ("window", "window", 0),
            ("hint", "key-hint", 0),
        ] {
            let properties = ask(&theme, name, selector, &parent);
            let actual: serde_json::Map<_, _> = properties
                .into_iter()
                .map(|(k, v)| {
                    (
                        String::from_utf16(&k).unwrap(),
                        serde_json::Value::String(
                            String::from_utf16(&v.encoder().serialize(&v).unwrap()).unwrap(),
                        ),
                    )
                })
                .collect();
            assert_eq!(
                serde_json::Value::Object(actual),
                expected[label],
                "{path} {label}"
            );
        }
    }
}
#[test]
fn every_bundled_theme_decodes_and_resolves_without_java() {
    let mut count = 0;
    for family in ["org.florisboard.themes", "org.florisboard.themes.my"] {
        let directory = root()
            .join("app/src/main/assets/ime/theme")
            .join(family)
            .join("stylesheets");
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let theme = CompiledTheme::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
            assert!(!ask(&theme, "key", 1, &vec![]).is_empty());
            count += 1;
        }
    }
    assert_eq!(count, 12);
}
#[test]
fn native_editor_owns_copies_conflicts_and_ordered_property_edits() {
    use snygg::{
        editor::{Editor, RuleEdit},
        resolve::{PropertySets, Stylesheet},
    };
    let rule = |s: &str| grammar::parse(&text(s)).unwrap().unwrap();
    let a = rule("key");
    let b = rule("key:pressed");
    let mut editor = Editor::new(Stylesheet {
        rules: vec![(
            a.clone(),
            PropertySets::Single(vec![(text("foreground"), Encoder::Color.default_value())]),
        )],
    });
    assert!(editor.copy_rule(&a, b.clone()));
    let PropertySets::Single(props) = &mut editor.sheet.rules[1].1 else {
        panic!("single set");
    };
    Editor::set_property(
        props,
        text("foreground"),
        Some(Value::Keyword(Encoder::Inherit)),
    );
    assert_ne!(editor.sheet.rules[0].1, editor.sheet.rules[1].1);
    assert_eq!(editor.rename(&a, b.clone(), true), RuleEdit::Duplicate);
    assert_eq!(
        editor.rename(&rule("absent"), rule("z"), false),
        RuleEdit::Missing
    );
    assert_eq!(
        editor.rename(&rule("absent"), Rule::Defines, true),
        RuleEdit::Add
    );
    assert!(matches!(editor.build().rules[0].0, Rule::Defines));
    assert!(editor.remove(&b).is_some());
}
