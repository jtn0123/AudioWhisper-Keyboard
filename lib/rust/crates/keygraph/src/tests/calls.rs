//! The graph's calls as rows: what Android sends and reads back.

use wire::key_graph::{
    self as names, action, action_answer, build, data, facts, file, hints, info, key, question,
};

use crate::Graph;
use crate::calls::{self, Error};
use crate::facts::Question;

use super::assets::answer;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn text(rows: &[Vec<u16>]) -> Vec<String> {
    rows.iter()
        .map(|row| String::from_utf16_lossy(row))
        .collect()
}

/// A letters keyboard of one file: an auto-text key, a multi-text key with a popup, and a key
/// of a currency slot.
fn graph() -> Graph {
    let layout = r#"[[{"$":"auto_text_key","code":97,"label":"a"},
        {"$":"multi_text_key","codePoints":[100,122],"label":"dz","popup":{"main":{"code":98,"label":"b"}}},
        {"code":-801,"label":"$"}]]"#;
    let mut row = vec![Vec::new(); build::SIZE];
    row[build::MODE] = utf16("0");
    row[build::NUMBER_HINTS] = utf16("0");
    row[build::LOADED] = utf16(&file::MAIN.to_string());
    row[build::MAIN] = utf16(layout);
    calls::build(&row).expect("a graph")
}

fn facts(slots: &[(&str, &str)]) -> Vec<Vec<u16>> {
    let mut row = vec![utf16("0"); facts::SIZE];
    row[facts::LANGUAGE] = utf16("English");
    row[facts::DEFAULT_LANGUAGE] = utf16("English (US)");
    row[facts::SLOTS] = utf16(&slots.len().to_string());
    for (code, label) in slots {
        row.extend([utf16(code), utf16(label)]);
    }
    row
}

fn call(graph: &mut Graph, op: &str, fields: &[Vec<u16>]) -> Vec<Vec<u16>> {
    calls::call(graph, &utf16(op), fields).expect("an answer")
}

/// Computes for `row`, answering each round's questions: how many rounds asked.
fn compute(graph: &mut Graph, row: &[Vec<u16>]) -> usize {
    for round in 0..10 {
        let rows = call(graph, names::COMPUTE, row);
        if rows.is_empty() {
            return round;
        }
        assert_eq!(rows.len() % question::SIZE, 0);
        let answers: Vec<Vec<u16>> = rows
            .chunks(question::SIZE)
            .map(|q| {
                answer(&Question {
                    ask: String::from_utf16_lossy(&q[question::ASK]).parse().unwrap(),
                    locale: String::from_utf16_lossy(&q[question::LOCALE])
                        .parse()
                        .unwrap(),
                    value: q[question::VALUE].clone(),
                })
            })
            .collect();
        call(graph, names::ANSWER, &answers);
    }
    panic!("the graph kept asking");
}

#[test]
fn a_computed_keyboard_reads_back_row_by_row() {
    let mut graph = graph();
    assert!(compute(&mut graph, &facts(&[("36", "$")])) > 0);
    let info = text(&call(&mut graph, names::INFO, &[]));
    assert_eq!(info[info::MODE], "0");
    assert_eq!(
        (info[info::ROWS].as_str(), info[info::LISTED].as_str()),
        ("1", "3")
    );
    let keys = call(&mut graph, names::KEYS, &[]);
    assert_eq!(keys.len(), 3 * key::SIZE);
    let field = |at: usize, field: usize| String::from_utf16_lossy(&keys[at * key::SIZE + field]);
    assert_eq!(
        (field(0, key::CODE), field(0, key::SHOWN)),
        ("97".into(), "a".into())
    );
    assert_eq!(
        (field(0, key::VISIBLE), field(0, key::LABELED)),
        ("1".into(), "1".into())
    );
    assert_eq!(
        (field(1, key::POPUP), field(1, key::MULTI)),
        ("0".into(), "0".into())
    );
    assert_eq!(
        (field(2, key::CODE), field(2, key::BASE)),
        ("36".into(), "-801".into())
    );
    let multi = field(1, key::MULTI);
    assert_eq!(
        text(&call(&mut graph, names::CODE_POINTS, &[utf16(&multi)])),
        ["100", "122"]
    );
}

#[test]
fn popups_read_back_as_data_rows() {
    let mut graph = graph();
    compute(&mut graph, &facts(&[]));
    let config = vec![utf16("1"), utf16("1"), utf16("1")];
    assert_eq!(config.len(), hints::SIZE);
    let flags = text(&call(&mut graph, names::POPUP_FLAGS, &config));
    assert_eq!(flags, ["0", "1", "0"]);
    let mut fields = vec![utf16("1")];
    fields.extend(config);
    let popups = text(&call(&mut graph, names::POPUPS, &fields));
    assert_eq!(popups.len(), 1 + data::SIZE);
    assert_eq!(popups[1 + data::CODE], "98");
    assert_eq!(
        (
            popups[1 + data::POPUP].as_str(),
            popups[1 + data::MULTI].as_str()
        ),
        ("-1", "-1")
    );
}

#[test]
fn actions_read_back_what_the_last_facts_say() {
    let mut graph = graph();
    let mut row = facts(&[]);
    row[facts::PASTABLE] = wire::flag::utf16(true);
    compute(&mut graph, &row);
    // A character key (`KeyType` 0) shows its label; the system key (6) of paste, its icon.
    let ask = |graph: &mut Graph, kind: &str, code: i32, label: &str| {
        let mut asked = vec![Vec::new(); action::SIZE];
        asked[action::TYPE] = utf16(kind);
        asked[action::CODE] = utf16(&code.to_string());
        asked[action::LABEL] = utf16(label);
        text(&call(graph, names::ACTIONS, &asked))
    };
    let letter = ask(&mut graph, "0", 97, "a");
    assert_eq!(letter.len(), action_answer::SIZE);
    assert_eq!(letter[action_answer::ENABLED], wire::flag::TRUE);
    let shown = &letter[action_answer::ICON..];
    assert_eq!(shown, ["0", wire::flag::TRUE, "a"]);
    let paste = keys::codes::CLIPBOARD_PASTE;
    let pasting = ask(&mut graph, "6", paste, "");
    assert_eq!(pasting[action_answer::ENABLED], wire::flag::TRUE);
    let shown = &pasting[action_answer::ICON..];
    assert_eq!(shown, [paste.to_string().as_str(), wire::flag::FALSE, ""]);
    compute(&mut graph, &facts(&[]));
    let pasting = ask(&mut graph, "6", paste, "");
    assert_eq!(pasting[action_answer::ENABLED], wire::flag::FALSE);
    let cut = vec![utf16("0"), utf16("97")];
    let refused = calls::call(&mut graph, &utf16(names::ACTIONS), &cut).err();
    assert_eq!(refused, Some(Error("an action row has one field each")));
}

#[test]
fn rows_cut_short_or_misread_are_refused() {
    assert_eq!(
        calls::build(&[]).err(),
        Some(Error("a build row has one field each"))
    );
    let mut graph = graph();
    let refused = |graph: &mut Graph, op: &str, fields: &[Vec<u16>]| {
        calls::call(graph, &utf16(op), fields).err()
    };
    assert_eq!(
        refused(&mut graph, "nothing", &[]),
        Some(Error("unknown operation"))
    );
    let short = facts(&[])[..facts::SIZE - 1].to_vec();
    assert_eq!(
        refused(&mut graph, names::COMPUTE, &short),
        Some(Error("a facts row is cut short"))
    );
    let mut unpaired = facts(&[("36", "$")]);
    unpaired.pop();
    let unpaired = refused(&mut graph, names::COMPUTE, &unpaired);
    assert_eq!(unpaired, Some(Error("each slot and resource is a pair")));
    let asked = call(&mut graph, names::COMPUTE, &facts(&[]));
    assert!(!asked.is_empty());
    let wrong = refused(&mut graph, names::ANSWER, &[]);
    assert_eq!(wrong, Some(Error("one answer is needed for each question")));
    assert_eq!(
        calls::layout(&mut graph, &[1.0]).err(),
        Some(Error("a layout takes five numbers"))
    );
    let bad_hints = vec![utf16("9"), utf16("0"), utf16("0")];
    assert_eq!(
        refused(&mut graph, names::POPUP_FLAGS, &bad_hints),
        Some(Error("not a hint mode"))
    );
}

#[test]
fn a_layout_reads_back_ten_numbers_a_key() {
    let mut graph = graph();
    compute(&mut graph, &facts(&[]));
    let numbers = calls::layout(&mut graph, &[300.0, 200.0, 50.0, 2.0, 2.0]).expect("a layout");
    assert_eq!(numbers.len(), 2 + 10 * 3);
    assert_eq!(numbers[2 + 9], 97.0);
    assert_eq!(numbers[2 + 8], 1.0);
}
