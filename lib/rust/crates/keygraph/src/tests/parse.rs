//! Reading files as kotlinx's lenient config did.

use crate::node::{Arena, Node};
use crate::parse;

fn one(text: &str) -> Option<Node> {
    let mut arena = Arena::default();
    parse::layout(&format!("[[{text}]]"), &mut arena).map(|mut rows| rows[0].remove(0))
}

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[test]
fn an_unknown_or_missing_class_reads_as_a_text_key() {
    for text in [
        r#"{"code": 97, "label": "a"}"#,
        r#"{"$": "kana_selector", "code": 97, "label": "a", "hira": {}}"#,
        r#"{"$": 5, "code": 97, "label": "a"}"#,
        r#"{"$": null, "code": "97", "label": "a"}"#,
    ] {
        let Some(Node::Text(leaf)) = one(text) else {
            panic!("{text} is a text key");
        };
        assert_eq!((leaf.code, leaf.label, leaf.kind), (97, utf16("a"), 0));
    }
}

#[test]
fn fields_take_defaults_and_lenient_literals() {
    let Some(Node::Text(leaf)) = one(r#"{"type": "System_Gui", "label": 5, "groupId": 2}"#) else {
        panic!("a text key");
    };
    assert_eq!(
        (leaf.kind, leaf.code, leaf.label, leaf.group),
        (6, 0, utf16("5"), 2)
    );
}

#[test]
fn what_kotlinx_rejects_fails_the_file() {
    for text in [
        r#"{"type": "letter"}"#,
        r#"{"code": 1.5}"#,
        r#"{"code": 4294967296}"#,
        r#"{"label": null}"#,
        r#"{"$": "case_selector", "lower": {"code": 1}}"#,
        r#"{"$": "case_selector", "lower": {"code": 1}, "upper": null}"#,
        r#"{"$": "char_width_selector", "full": null}"#,
        r#"{"$": {}}"#,
        r#"{"popup": {"relevant": null}}"#,
        r#""a""#,
    ] {
        assert_eq!(one(text), None, "{text}");
    }
    let mut arena = Arena::default();
    assert_eq!(parse::layout("[[{\"code\": 1}], null]", &mut arena), None);
    assert_eq!(parse::layout("not json", &mut arena), None);
    assert_eq!(
        arena,
        Arena::default(),
        "a failed file leaves nothing behind"
    );
}

#[test]
fn selectors_keep_their_children_in_kotlin_order() {
    let node = one(r#"{"$": "variation_selector", "uri": {"code": 47}, "default": {"code": 44}}"#);
    let Some(Node::Select(2, children)) = node else {
        panic!("a variation selector");
    };
    let codes: Vec<Option<i32>> = children
        .iter()
        .map(|child| match child {
            Some(Node::Text(leaf)) => Some(leaf.code),
            _ => None,
        })
        .collect();
    assert_eq!(codes, [Some(44), None, Some(47), None, None]);
    let width = one(r#"{"$": "char_width_selector", "full": null, "half": {"code": 1}}"#);
    assert!(matches!(width, Some(Node::Select(4, children)) if children[0].is_none()));
}

#[test]
fn multi_text_keys_and_popups_go_to_the_arena() {
    let mut arena = Arena::default();
    let text = r#"[[{"$": "multi_text_key", "code": "x", "codePoints": [97, 98], "label": "ab",
        "popup": {"main": {"$": "auto_text_key", "code": 98}, "relevant": [{"code": 99}]}}]]"#;
    let rows = parse::layout(text, &mut arena).expect("reads");
    let Node::Multi(leaf, multi) = &rows[0][0] else {
        panic!("a multi-text key");
    };
    assert_eq!(
        (leaf.code, leaf.popup, *multi),
        (keys::codes::MULTIPLE_CODE_POINTS, Some(0), 0)
    );
    assert_eq!(arena.multis, [vec![97, 98]]);
    assert_eq!(arena.autos, [(98, Vec::new())]);
    assert_eq!(arena.popups[0].relevant.len(), 1);
}

#[test]
fn a_mapping_names_its_variations() {
    let mut arena = Arena::default();
    let mapping = parse::mapping(
        r#"{"all": {"a": {"relevant": [{"code": 228}]}}, "uri": {}}"#,
        &mut arena,
    );
    let mapping = mapping.expect("reads");
    assert_eq!(mapping.get(0, &utf16("a")), Some(0));
    assert!(mapping.variations[4].is_some() && mapping.variations[1].is_none());
    assert_eq!(parse::mapping(r#"{"everything": {}}"#, &mut arena), None);
    assert_eq!(parse::mapping(r#"{"all": {"a": null}}"#, &mut arena), None);
}
