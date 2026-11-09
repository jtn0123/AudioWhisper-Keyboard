use super::*;

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn rules(pairs: &[(&str, &str)]) -> Rules {
    Rules::new(
        pairs
            .iter()
            .map(|&(key, value)| (units(key), units(value)))
            .collect(),
    )
}

fn typed(rules: &Rules, preceding: &str, to_insert: &str) -> Option<(i32, String)> {
    rules
        .actions(&units(preceding), &units(to_insert))
        .map(|action| (action.delete, String::from_utf16(&action.text).unwrap()))
}

fn action(delete: i32, text: &str) -> Option<(i32, String)> {
    Some((delete, text.to_string()))
}

#[test]
fn the_first_matching_rule_replaces_its_ending() {
    let telex = rules(&[
        ("uow", "ươ"),
        ("aa", "â"),
        ("as", "á"),
        ("âs", "ấ"),
        ("w", "ư"),
    ]);
    assert_eq!(typed(&telex, "ta", "a"), action(1, "â"));
    assert_eq!(typed(&telex, "tâ", "s"), action(1, "ấ"));
    assert_eq!(typed(&telex, "uo", "w"), action(2, "ươ"));
    assert_eq!(typed(&telex, "x", "w"), action(0, "ư"));
    assert_eq!(typed(&telex, "x", "b"), None);
    // Earlier rules win.
    let first = rules(&[("a", "1"), ("ba", "2")]);
    assert_eq!(typed(&first, "b", "a"), action(0, "1"));
}

#[test]
fn an_upper_case_ending_upper_cases_the_text() {
    let telex = rules(&[("aa", "â"), ("ss", "ß")]);
    assert_eq!(typed(&telex, "A", "a"), action(1, "Â"));
    assert_eq!(typed(&telex, "A", "A"), action(1, "Â"));
    assert_eq!(typed(&telex, "a", "A"), action(1, "â"));
    // ß upper-cases to SS.
    assert_eq!(typed(&telex, "S", "s"), action(1, "SS"));
    // A digit is its own upper case.
    let digits = rules(&[("1a", "x")]);
    assert_eq!(typed(&digits, "1", "a"), action(1, "X"));
}

#[test]
fn case_mapping_can_change_the_length() {
    // İ lower-cases to i and a combining dot: two units, of one typed.
    let dotted = rules(&[("i\u{307}", "y")]);
    assert_eq!(typed(&dotted, "", "İ"), action(1, "Y"));
    // The ending is cut from the text as typed, so here it starts at the x.
    assert_eq!(typed(&dotted, "x", "İ"), action(1, "y"));
    let longer = rules(&[("xi\u{307}", "y")]);
    assert_eq!(typed(&longer, "x", "İ"), action(2, "y"));
}

#[test]
fn an_empty_ending_always_matches() {
    let empty = rules(&[("", "e")]);
    assert_eq!(typed(&empty, "", ""), action(-1, "E"));
    assert_eq!(typed(&empty, "a", "b"), action(-1, "E"));
    assert_eq!(typed(&rules(&[]), "a", "b"), None);
}
