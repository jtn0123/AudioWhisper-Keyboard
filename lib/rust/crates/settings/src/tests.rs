use super::*;
#[test]
fn catalogue_is_complete_unique_and_defaults_are_valid() {
    let rows = catalogue::all();
    assert_eq!(rows.len(), 145);
    let mut keys = std::collections::HashSet::new();
    for row in rows {
        assert!(keys.insert(row.key));
        if !row.choices.is_empty() {
            assert!(catalogue::enum_index(&text(row.key), &text(row.default)).unwrap() >= 0);
        }
    }
}
#[test]
fn malformed_packets_are_errors() {
    for op in [
        ops::ENUM_INDEX,
        ops::CLAMP,
        ops::DECODE,
        ops::ENCODE,
        ops::MIGRATE,
        ops::SUBTYPE,
        ops::PRESET,
    ] {
        assert!(call(op, &[]).is_err());
    }
    assert!(call(ops::SUBTYPE, &[text("a"), text("-1")]).is_err());
}
#[test]
fn the_layout_packet_follows_the_layouts() {
    use layouts::*;
    let fields = [
        (CHARACTERS, "characters"),
        (SYMBOLS, "symbols"),
        (SYMBOLS2, "symbols2"),
        (NUMERIC, "numeric"),
        (NUMERIC_ADVANCED, "numericAdvanced"),
        (NUMERIC_ROW, "numericRow"),
        (PHONE, "phone"),
        (PHONE2, "phone2"),
    ];
    assert_eq!(fields.len(), SIZE);
    for (field, name) in fields {
        assert_eq!(codec::LAYOUTS[field].0, name);
    }
}
#[test]
fn subtype_fallback_and_removed_inputs() {
    assert_eq!(subtype::active(&[4, 7], 7, -1), (1, false));
    assert_eq!(subtype::active(&[4, 7], 8, -1), (0, true));
    assert_eq!(subtype::active(&[], 8, -1), (-1, true));
    let mut old = codec::default_subtype();
    old.set(
        "composer",
        Value::string("org.florisboard.composers:kana-unicode"),
    )
    .unwrap();
    assert_eq!(subtype::supported(&old).unwrap(), codec::default_subtype());
}
#[test]
fn utf16_strings_remain_exact() {
    let value = value::object([
        (
            "pinned",
            Value::List(vec![Value::String(vec![0xd800, 0, 34, 92, 0xdfff])]),
        ),
        ("recent", Value::List(vec![])),
    ]);
    let raw = codec::encode("history", &value).unwrap();
    assert_eq!(codec::decode("history", &raw, false).unwrap(), value);
}
#[test]
fn learning_how_the_user_taps_is_on_by_default_like_learning_words() {
    let rows = catalogue::all();
    let row = |key: &str| rows.iter().find(|row| row.key == key).unwrap();
    let aim = row("suggestion__learn_tap_aim");
    assert_eq!((aim.kind, aim.default), ("b", "true"));
    let words = row("suggestion__learn_from_typing");
    assert_eq!((aim.kind, aim.default), (words.kind, words.default));
}
#[test]
fn auto_correction_is_normal_by_default_and_lists_every_strength() {
    let rows = catalogue::all();
    let row = rows
        .iter()
        .find(|row| row.key == "correction__auto_correct")
        .unwrap();
    assert_eq!((row.kind, row.default), ("s", "NORMAL"));
    assert_eq!(row.choices, "OFF|MODEST|NORMAL|AGGRESSIVE");
}
#[test]
fn the_removed_settings_switch_is_dropped_whatever_it_stored() {
    let key = text("other__rust_settings_prototype");
    for raw in ["true", "false", ""] {
        let plan = migration::migrate(&text("b"), &key, &text(raw)).unwrap();
        assert_eq!(plan[0], text("reset"));
    }
    // The keyboard's switch is still a setting and is kept as stored.
    let kept = migration::migrate(
        &text("b"),
        &text("keyboard__rust_renderer_prototype"),
        &text("false"),
    )
    .unwrap();
    assert_ne!(kept[0], text("reset"));
}
