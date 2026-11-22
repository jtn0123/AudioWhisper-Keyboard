//! The rounds of questions a computation asks, and the files a graph reads once.

use crate::assemble::Reader;
use keys::codes;

use crate::facts::{Facts, Question, ask};
use crate::{Files, Graph, Request};

use super::assets::answer;

/// A code that isn't a code point: Android's `UCharacter.toString` gives null for it, and the
/// answer adapter throws, as main's `AutoLetterState` did.
const NOT_A_CODE_POINT: &str = "-2147483648";

/// Computes `graph` for `facts`, failing on any question about [`NOT_A_CODE_POINT`].
fn compute(graph: &mut Graph, facts: &Facts) -> Vec<Question> {
    let mut asked = Vec::new();
    for _ in 0..10 {
        let questions = graph.compute(facts);
        if questions.is_empty() {
            return asked;
        }
        for question in &questions {
            let value = String::from_utf16_lossy(&question.value);
            assert_ne!(
                value, NOT_A_CODE_POINT,
                "asked {question:?}, which main never asks"
            );
        }
        let answers: Vec<Vec<u16>> = questions.iter().map(answer).collect();
        graph.answer(&answers).expect("answers");
        asked.extend(questions);
    }
    panic!("the graph kept asking");
}

#[test]
fn a_label_waiting_on_its_case_looks_nothing_up() {
    // Two letters whose label lowercases to "abc", and a mapping with a set under "ABC" holding
    // a key no casing survives. Main looks up "abc" and never reaches it; a lookup made with the
    // label standing in for its answer would, the second letter's question being already open.
    let layout = r#"[[{"$":"auto_text_key","code":65,"label":"ABC"},
        {"$":"auto_text_key","code":65,"label":"ABC"}]]"#;
    let mapping = format!(
        r#"{{"all":{{"ABC":{{"relevant":[{{"$":"auto_text_key","code":{NOT_A_CODE_POINT}}}]}}}}}}"#
    );
    let mut graph = Graph::build(&Request {
        layouts: Files {
            main: Some(layout),
            ..Files::default()
        },
        mapping: Some(&mapping),
        ..Request::default()
    });
    let asked = compute(&mut graph, &Facts::default());
    assert!(!asked.is_empty());
    let label = String::from_utf16_lossy(&graph.keys()[1].data.label);
    assert_eq!(label, "abc");
    assert!(
        graph.keys()[1]
            .popups
            .keys(Facts::default().hints)
            .is_empty()
    );
}

#[test]
fn a_file_read_twice_is_parsed_once() {
    let layout = r#"[[{"code":97,"label":"a","popup":{"main":{"code":98}}}]]"#.to_string();
    let mapping = r#"{"all":{"a":{"main":{"code":98,"popup":{"main":{"code":99}}}}}}"#.to_string();
    let (layout_again, mapping_again) = (layout.clone(), mapping.clone());
    let mut reader = Reader::default();
    let rows = reader.layout(&layout);
    let mapped = reader.mapping(&mapping);
    let popups = reader.arena.popups.len();
    assert_eq!(popups, 3);
    assert_eq!(reader.layout(&layout_again), rows);
    assert_eq!(reader.mapping(&mapping_again), mapped);
    assert_eq!(reader.arena.popups.len(), popups);
}

#[test]
fn only_codes_from_the_first_mark_up_are_asked_about() {
    let layout = r#"[[{"code":97,"label":"a"},{"code":769,"label":"\u0301"}]]"#;
    let mut graph = Graph::build(&Request {
        layouts: Files {
            main: Some(layout),
            ..Files::default()
        },
        ..Request::default()
    });
    let asked = compute(&mut graph, &Facts::default());
    let marks: Vec<String> = asked
        .iter()
        .filter(|question| question.ask == ask::MARK)
        .map(|question| String::from_utf16_lossy(&question.value))
        .collect();
    assert_eq!(marks, ["769"]);
}

#[test]
fn an_action_is_answered_from_the_last_facts() {
    let mut graph = Graph::build(&Request::default());
    let facts = Facts {
        pastable: true,
        ..Facts::default()
    };
    compute(&mut graph, &facts);
    let label: Vec<u16> = "a".encode_utf16().collect();
    let letter = graph.action(0, 97, &label);
    assert_eq!((letter.enabled, letter.label), (true, Some(label)));
    assert!(graph.action(0, codes::CLIPBOARD_PASTE, &[]).enabled);
    compute(&mut graph, &Facts::default());
    assert!(!graph.action(0, codes::CLIPBOARD_PASTE, &[]).enabled);
}
