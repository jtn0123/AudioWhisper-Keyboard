use settings::{Value, codec, text};
use wire::ime_subtype::{self as ops, loaded, subtype, switched};

use crate::calls::{Error, call};
use crate::session::{Load, Session, Switch};

fn subtype_json(id: i64, locale: &str, characters: &str) -> String {
    format!(
        r#"{{"id":{id},"primaryLocale":"{locale}","secondaryLocales":["de_DE"],"composer":"org.florisboard.composers:appender","currencySet":"org.florisboard.currencysets:euro","popupMapping":"org.florisboard.localization:de","layoutMap":{{"characters":"org.florisboard.layouts:{characters}"}}}}"#
    )
}

fn list(items: &[String]) -> Vec<u16> {
    text(&format!("[{}]", items.join(",")))
}

/// A session that names every tag as Android would for these tests: `_` becomes `-`.
fn loaded_session(raw: &[u16]) -> (Session, bool) {
    let mut session = Session::new();
    let Load::Ask(tags) = session.load(raw).unwrap() else {
        panic!("a first load asks for the tags")
    };
    let names: Vec<Vec<u16>> = tags
        .iter()
        .map(|t| t.iter().map(|&u| if u == 95 { 45 } else { u }).collect())
        .collect();
    session.name(&names).unwrap();
    let Load::Loaded { changed } = session.load(raw).unwrap() else {
        panic!("every tag is named")
    };
    (session, changed)
}

#[test]
fn a_blank_list_is_empty_and_the_default_is_active() {
    let (mut session, changed) = loaded_session(&text(" \u{a0}\t"));
    assert!(!changed);
    assert!(session.list().is_empty());
    assert_eq!(
        session.evaluate(5).unwrap(),
        Switch {
            write: true,
            id: -1
        }
    );
    assert_eq!(session.active(), &codec::default_subtype());
    assert_eq!(
        session.evaluate(-1).unwrap(),
        Switch {
            write: false,
            id: -1
        }
    );
}

#[test]
fn a_load_asks_for_each_tag_once_with_the_default_one() {
    let mut session = Session::new();
    let raw = list(&[
        subtype_json(1, "en_US", "qwerty"),
        subtype_json(2, "fr_FR", "azerty"),
    ]);
    let Load::Ask(tags) = session.load(&raw).unwrap() else {
        panic!("asks")
    };
    assert_eq!(
        tags,
        vec![text("en_US"), text("de_DE"), text("fr_FR"), text("en-US")]
    );
    assert_eq!(
        session.name(&[text("x")]),
        Err("an answer for each tag asked")
    );
    let mut fresh = Session::new();
    fresh.load(&raw).unwrap();
    fresh
        .name(&tags.iter().map(|_| text("xx")).collect::<Vec<_>>())
        .unwrap();
    assert_eq!(fresh.load(&raw).unwrap(), Load::Loaded { changed: false });
    assert_eq!(
        fresh.list()[1].get("primaryLocale").unwrap(),
        &Value::string("xx")
    );
}

#[test]
fn a_removed_layout_is_repaired_to_the_default_with_its_id() {
    let raw = list(&[
        subtype_json(7, "en_US", "qwerty"),
        subtype_json(9, "ar", "arabic"),
    ]);
    let (session, changed) = loaded_session(&raw);
    assert!(changed);
    let mut expected = codec::default_subtype();
    expected.set("id", Value::Number(9)).unwrap();
    assert_eq!(session.list()[1], expected);
    assert_eq!(
        session.list()[0].get("primaryLocale").unwrap(),
        &Value::string("en-US")
    );
}

#[test]
fn the_stored_id_else_the_first_is_active() {
    let raw = list(&[
        subtype_json(4, "en_US", "qwerty"),
        subtype_json(7, "fr_FR", "azerty"),
    ]);
    let (mut session, _) = loaded_session(&raw);
    assert_eq!(
        session.evaluate(7).unwrap(),
        Switch {
            write: false,
            id: 7
        }
    );
    assert_eq!(session.evaluate(8).unwrap(), Switch { write: true, id: 4 });
}

#[test]
fn the_language_key_wraps_both_ways_and_falls_back_off_the_list() {
    let raw = list(&[
        subtype_json(1, "en_US", "qwerty"),
        subtype_json(2, "fr_FR", "azerty"),
        subtype_json(3, "de_DE", "qwertz"),
    ]);
    let (mut session, _) = loaded_session(&raw);
    session.evaluate(3).unwrap();
    assert_eq!(session.cycle(false).unwrap().id, 1);
    assert_eq!(session.cycle(true).unwrap().id, 3);
    assert_eq!(session.cycle(true).unwrap().id, 2);
    let (mut other, _) = loaded_session(&list(&[subtype_json(5, "en_US", "qwerty")]));
    assert_eq!(
        other.cycle(false).unwrap(),
        Switch {
            write: true,
            id: -1
        }
    );
    assert_eq!(other.active(), &codec::default_subtype());
}

#[test]
fn a_switch_by_id_takes_the_first_with_it() {
    let raw = list(&[
        subtype_json(1, "en_US", "qwerty"),
        subtype_json(2, "fr_FR", "azerty"),
        subtype_json(2, "de_DE", "qwertz"),
    ]);
    let (mut session, _) = loaded_session(&raw);
    assert_eq!(session.select(9).unwrap(), None);
    assert_eq!(
        session.select(2).unwrap(),
        Some(Switch { write: true, id: 2 })
    );
    assert_eq!(
        session.active().get("primaryLocale").unwrap(),
        &Value::string("fr-FR")
    );
}

#[test]
fn equal_subtypes_share_a_cache_key() {
    let mut session = Session::new();
    let a = codec::default_subtype();
    let mut b = a.clone();
    b.set("id", Value::Number(3)).unwrap();
    assert_eq!(session.key(&a), 0);
    assert_eq!(session.key(&b), 1);
    assert_eq!(session.key(&a.clone()), 0);
}

#[test]
fn a_currency_set_is_found_among_those_reported() {
    let mut session = Session::new();
    assert_eq!(session.currency(&text("a:b")), None);
    session.report(vec![text("x:y"), text("a:b")]);
    assert_eq!(session.currency(&text("a:b")), Some(1));
    assert_eq!(session.currency(&text("a:c")), None);
}

#[test]
fn rows_name_the_layouts_by_layout_type_and_the_secondary_locales_after() {
    let mut session = Session::new();
    let raw = list(&[subtype_json(1, "en_US", "qwertz")]);
    let first = call(&mut session, &text(ops::LOAD), std::slice::from_ref(&raw)).unwrap();
    assert_eq!(first[loaded::ASKS], text("3"));
    let names = first[loaded::SIZE..].to_vec();
    call(&mut session, &text(ops::LOCALES), &names).unwrap();
    let rows = call(&mut session, &text(ops::LOAD), &[raw]).unwrap();
    assert_eq!(rows[loaded::ASKS], text("0"));
    assert_eq!(rows[loaded::CHANGED], text(wire::flag::encode(false)));
    assert_eq!(rows[loaded::COUNT], text("1"));
    let row = &rows[loaded::SIZE..];
    assert_eq!(row.len(), subtype::SIZE + 1);
    assert_eq!(row[subtype::ID], text("1"));
    assert_eq!(row[subtype::KEY], text("0"));
    assert_eq!(
        row[subtype::CHARACTERS],
        text("org.florisboard.layouts:qwertz")
    );
    assert_eq!(
        row[subtype::SYMBOLS2],
        text("org.florisboard.layouts:western")
    );
    assert!(row[subtype::CHARACTERS_MOD].is_empty() && row[subtype::EXTENSION].is_empty());
    assert_eq!(row[subtype::POPUP], text("org.florisboard.localization:de"));
    assert_eq!(row[subtype::SECONDARY], text("1"));
    assert_eq!(row[subtype::SIZE], text("de_DE"));
    call(&mut session, &text(ops::EVALUATE), &[text("1")]).unwrap();
    let next = call(&mut session, &text(ops::NEXT), &[]).unwrap();
    assert_eq!(next[switched::WRITE], text(wire::flag::encode(true)));
    assert_eq!(next[switched::ID], text("1"));
    assert_eq!(next[switched::SIZE + subtype::KEY], text("0"));
    let active = call(&mut session, &text(ops::ACTIVE), &[]).unwrap();
    assert_eq!(active[switched::WRITE], text(wire::flag::encode(false)));
    assert_eq!(active[switched::ID], text("1"));
    assert_eq!(active[switched::SIZE..], next[switched::SIZE..]);
    let missing = call(&mut session, &text(ops::SELECT), &[text("8")]).unwrap();
    assert_eq!(missing.len(), switched::SIZE);
    assert_eq!(missing[switched::ACTIVE], text(wire::flag::encode(false)));
}

#[test]
fn a_repaired_list_is_written_back_as_the_settings_store_it() {
    let mut session = Session::new();
    let raw = list(&[subtype_json(9, "en-US", "arabic")]);
    let tags = call(&mut session, &text(ops::LOAD), std::slice::from_ref(&raw)).unwrap();
    call(&mut session, &text(ops::LOCALES), &tags[loaded::SIZE..]).unwrap();
    let rows = call(&mut session, &text(ops::LOAD), &[raw]).unwrap();
    assert_eq!(rows[loaded::CHANGED], text(wire::flag::encode(true)));
    let mut expected = codec::default_subtype();
    expected.set("id", Value::Number(9)).unwrap();
    assert_eq!(
        rows[loaded::RAW],
        codec::encode("subtypes", &Value::List(vec![expected])).unwrap()
    );
}

#[test]
fn broken_rows_are_refused() {
    let mut session = Session::new();
    assert_eq!(
        call(&mut session, &text("nope"), &[]),
        Err(Error("not an operation"))
    );
    assert_eq!(
        call(&mut session, &text(ops::EVALUATE), &[]),
        Err(Error("a row is cut short"))
    );
    assert_eq!(
        call(&mut session, &text(ops::SELECT), &[text("x")]),
        Err(Error("not a number"))
    );
    assert!(call(&mut session, &text(ops::LOAD), &[text("[{")]).is_err());
}
