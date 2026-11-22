//! The graph over the keyboard's own layout files, with a stand-in for Android's casing.

use std::path::{Path, PathBuf};

use popup::order::{HintConfig, HintMode};

use crate::facts::{Facts, Question, ask};
use crate::node::Arena;
use crate::{Files, Graph, Request, parse};

fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../app/src/main/assets/ime/keyboard")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(assets().join(path)).expect("an asset")
}

fn layout(kind: &str, name: &str) -> String {
    read(&format!(
        "org.florisboard.layouts/layouts/{kind}/{name}.json"
    ))
}

/// What Android would answer, with Rust's casing standing in for ICU's.
pub fn answer(question: &Question) -> Vec<u16> {
    let text = String::from_utf16_lossy(&question.value);
    let code = || char::from_u32(text.parse().expect("a code")).expect("a char");
    let answer = match question.ask {
        ask::LOWER_CODE => (code().to_lowercase().next().expect("a char") as u32).to_string(),
        ask::UPPER_CODE => (code().to_uppercase().next().expect("a char") as u32).to_string(),
        ask::LOWER_TEXT => text.to_lowercase(),
        ask::UPPER_TEXT => text.to_uppercase(),
        _ => u8::from(('\u{300}'..'\u{370}').contains(&code())).to_string(),
    };
    answer.encode_utf16().collect()
}

/// Computes `graph` for `facts`, answering its questions: how many rounds that took.
pub fn compute(graph: &mut Graph, facts: &Facts) -> usize {
    for round in 0..10 {
        let questions = graph.compute(facts);
        if questions.is_empty() {
            return round;
        }
        let answers: Vec<Vec<u16>> = questions.iter().map(answer).collect();
        graph.answer(&answers).expect("answers");
    }
    panic!("the graph kept asking");
}

fn qwerty(number_hints: bool) -> Graph {
    let (main, modifier, symbols) = (
        layout("characters", "qwerty"),
        layout("charactersMod", "default"),
        layout("symbols", "western"),
    );
    let (symbols_mod, row) = (
        layout("symbolsMod", "default"),
        layout("numericRow", "western_arabic"),
    );
    let mapping = read("org.florisboard.localization/popupMappings/de.json");
    let fallback = read("org.florisboard.localization/popupMappings/default.json");
    Graph::build(&Request {
        mode: 0,
        number_hints,
        layouts: Files {
            main: Some(&main),
            modifier: Some(&modifier),
            ..Files::default()
        },
        symbols: Files {
            main: Some(&symbols),
            modifier: Some(&symbols_mod),
            own_modifier: None,
            extension: Some(&row),
        },
        mapping: Some(&mapping),
        fallback: Some(&fallback),
    })
}

fn hints() -> HintConfig {
    HintConfig {
        symbol: HintMode::HintPriority,
        number: HintMode::HintPriority,
        merge_hint_popups: true,
    }
}

fn label(graph: &Graph, at: usize) -> String {
    String::from_utf16_lossy(graph.keys()[at].label.as_deref().unwrap_or_default())
}

#[test]
fn every_layout_and_mapping_reads() {
    let root = assets().join("org.florisboard.layouts/layouts");
    let mut count = 0;
    for kind in std::fs::read_dir(&root).expect("layouts") {
        for file in std::fs::read_dir(kind.expect("a kind").path()).expect("files") {
            let text = std::fs::read_to_string(file.expect("a file").path()).expect("text");
            assert!(parse::layout(&text, &mut Arena::default()).is_some());
            count += 1;
        }
    }
    assert!(count > 80, "{count} layouts");
    let mappings = assets().join("org.florisboard.localization/popupMappings");
    for file in std::fs::read_dir(mappings).expect("mappings") {
        let text = std::fs::read_to_string(file.expect("a file").path()).expect("text");
        assert!(parse::mapping(&text, &mut Arena::default()).is_some());
    }
}

#[test]
fn letters_case_with_shift_and_hint_their_numbers() {
    let mut graph = qwerty(true);
    let facts = Facts {
        hints: hints(),
        ..Facts::default()
    };
    assert!(
        compute(&mut graph, &facts) > 0,
        "the first computation asks"
    );
    assert_eq!(graph.row_sizes(), [10, 9, 9, 7]);
    assert_eq!(
        (label(&graph, 0), graph.keys()[0].data.code),
        ("q".into(), 113)
    );
    assert_eq!(
        graph.keys()[0].hint.as_deref(),
        Some(&*"1".encode_utf16().collect::<Vec<_>>())
    );
    let shifted = Facts {
        state: 1 << 8,
        ..facts.clone()
    };
    assert!(compute(&mut graph, &shifted) > 0, "upper case is new");
    assert_eq!(compute(&mut graph, &facts), 0, "nothing more to ask");
    compute(&mut graph, &shifted);
    assert_eq!(
        (label(&graph, 0), graph.keys()[0].data.code),
        ("Q".into(), 81)
    );
    let a = &graph.keys()[10];
    assert!(
        a.popups
            .keys(hints())
            .all()
            .any(|key| key.code == 228 || key.code == 196)
    );
}

#[test]
fn a_new_graph_holds_uncomputed_keys_and_lays_them_out() {
    let mut graph = Graph::placeholder();
    assert_eq!((graph.listed(), graph.keys()[0].data.code), (34, 0));
    let desired = graph.layout(1080.0, 800.0, 160.0, [4.0, 4.0]);
    assert!(desired[0] > 0.0);
    assert!(graph.keys()[0].bounds.touch.bottom > 0.0, "laid out");
}
