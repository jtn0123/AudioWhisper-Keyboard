use super::*;

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn typed(preceding: &str, to_insert: &str, sticky: bool) -> Option<(i32, String)> {
    actions(&units(preceding), &units(to_insert), sticky)
        .map(|action| (action.delete, String::from_utf16(&action.text).unwrap()))
}

fn action(delete: i32, text: &str) -> Option<(i32, String)> {
    Some((delete, text.to_string()))
}

#[test]
fn marks_apply_and_come_off_again() {
    assert_eq!(typed("か", "\u{3099}", false), action(1, "が"));
    assert_eq!(typed("か", "゛", false), action(1, "が"));
    assert_eq!(typed("カ", "ﾞ", false), action(1, "ガ"));
    assert_eq!(typed("が", "゛", false), action(1, "か"));
    assert_eq!(typed("は", "゜", false), action(1, "ぱ"));
    assert_eq!(typed("ぱ", "ﾟ", false), action(1, "は"));
    // A voiced kana half-voiced, and back.
    assert_eq!(typed("び", "゜", false), action(1, "ぴ"));
    assert_eq!(typed("ぴ", "゛", false), action(1, "び"));
}

#[test]
fn the_small_key_makes_kana_small_and_back() {
    assert_eq!(typed("つ", "〓", false), action(1, "っ"));
    assert_eq!(typed("っ", "〓", false), action(1, "つ"));
    assert_eq!(typed("ゐ", "〓", false), action(1, "𛅐"));
    assert_eq!(typed("ン", "〓", false), action(1, "𛅧"));
    // A voiced kana is made small from its base.
    assert_eq!(typed("づ", "〓", false), action(1, "っ"));
    // The small ゐ has no way back.
    assert_eq!(typed("𛅐", "〓", false), action(0, ""));
}

#[test]
fn sticky_keeps_marks_on() {
    assert_eq!(typed("が", "゛", true), action(1, "が"));
    assert_eq!(typed("っ", "〓", true), action(1, "っ"));
    assert_eq!(typed("\u{3099}", "\u{3099}", true), action(1, "\u{3099}"));
}

#[test]
fn combining_marks_in_a_row() {
    assert_eq!(typed("\u{3099}", "\u{3099}", false), action(1, ""));
    assert_eq!(typed("\u{3099}", "\u{309A}", false), action(1, "\u{309A}"));
    // Only combining marks combine.
    assert_eq!(typed("゛", "゛", false), action(0, "゛"));
}

#[test]
fn a_mark_with_nothing_to_apply_to() {
    assert_eq!(typed("a", "゛", false), action(0, "゛"));
    assert_eq!(typed("a", "゛x", false), action(0, "゛"));
    assert_eq!(typed("a", "〓", false), action(0, ""));
    assert_eq!(typed("", "〓", false), action(0, ""));
    assert_eq!(typed("", "\u{3099}", false), action(0, ""));
    assert_eq!(typed("", "゛", false), None);
}

#[test]
fn anything_else_is_typed_as_it_is() {
    assert_eq!(typed("か", "か", false), None);
    assert_eq!(typed("か", "", false), None);
    assert_eq!(typed("", "", true), None);
}

#[test]
fn every_table_has_each_key_once() {
    fn unique<V>(table: &[(char, V)]) {
        let mut keys: Vec<char> = table.iter().map(|&(key, _)| key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), table.len());
    }
    unique(&DAKU);
    unique(&HANDAKU);
    unique(&SMALL);
    unique(&REVERSE_DAKU);
    unique(&REVERSE_HANDAKU);
    unique(&REVERSE_SMALL);
}
