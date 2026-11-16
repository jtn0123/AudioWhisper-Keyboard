//! The "aim" operation: which kept words may teach where this user taps; and what each kind of
//! field allows: suggestions, auto-correction, learning and remembering a word kept as typed.

use crate::{Context, Field, Session, number, text};

fn session(change: impl FnOnce(&mut Vec<Field>)) -> Session {
    let mut c = vec![number(0); 22];
    c[0] = number(1);
    c[2] = text("org.florisboard.nlp.providers.latin");
    c[3] = text("en");
    c[4] = number(1);
    c[17] = number(1);
    change(&mut c);
    let mut session = Session::default();
    session.configure(Context(c));
    session
}

fn aim(session: &mut Session, fields: [&str; 4]) -> bool {
    let fields: Vec<Field> = fields.iter().map(|f| text(f)).collect();
    session.checked_call("aim", &fields).unwrap() == [number(1)]
}

#[test]
fn a_kept_word_teaches_the_aim_only_when_allowed() {
    let mut plain = session(|_| {});
    assert!(aim(&mut plain, ["1", "-1", "pop", "1"]));
    assert!(
        aim(&mut plain, ["1", "46", "pop", "1"]),
        "a full stop ends the word"
    );
    // The setting is off; the word goes on; nothing was typed; no composing region.
    assert!(!aim(&mut plain, ["0", "-1", "pop", "1"]));
    assert!(!aim(&mut plain, ["1", "39", "pop", "1"]));
    assert!(!aim(&mut plain, ["1", "-1", "", "1"]));
    assert!(!aim(&mut plain, ["1", "-1", "pop", "0"]));
    // It needs no "learn from typing": that is about words, this about taps.
    let mut learning_off = session(|c| c[6] = number(0));
    assert!(aim(&mut learning_off, ["1", "-1", "pop", "1"]));
    // Incognito, a raw editor, a password, an e-mail address, or no suggestions.
    for change in [(7, 1), (1, 1), (0, 0x81), (0, 0x21), (4, 0)] {
        let mut s = session(|c| c[change.0] = number(change.1));
        assert!(!aim(&mut s, ["1", "-1", "pop", "1"]), "{change:?}");
    }
    let mut s = Session::default();
    assert!(s.checked_call("aim", &[text("1")]).is_err());
}

/// (on, words, emoji, auto-correct, learn) and whether a kept word is remembered.
fn allows(session: &mut Session) -> ([bool; 5], bool) {
    let policy = session
        .checked_call("policy", &session.context.0.clone())
        .unwrap();
    let keep = session.checked_call("keep", &[text("zorb")]).unwrap() == [number(1)];
    (std::array::from_fn(|i| policy[i] == number(1)), keep)
}

#[test]
fn each_kind_of_field_allows_what_it_should() {
    let prose = ([true, true, false, true, true], true);
    let not_prose = ([true, true, false, false, false], false);
    let secret = ([false, false, false, false, false], false);
    // (inputType, imeOptions, what it allows)
    for (input, options, expected) in [
        (0x1, 0, prose),
        (0x4001, 0, prose),
        // A search box: Gboard corrects in Settings' search, so this keyboard does too.
        (0x10_0001, 0x1000_0003, prose),
        (0x11, 0, not_prose),
        (0x21, 0, not_prose),
        // An e-mail field that asks for auto-correction gets none either.
        (0x8021, 0, not_prose),
        (0xd1, 0, not_prose),
        (0x200b1, 0x4000_0006, not_prose),
        (0x81, 0, secret),
        (0x91, 0, secret),
        (0xe1, 0, secret),
        (0x12, 0, secret),
        (0x80001, 0, secret),
    ] {
        let mut s = session(|c| {
            c[0] = number(input);
            c[6] = number(1);
            c[21] = number(options);
        });
        assert_eq!(allows(&mut s), expected, "{input:#x} {options:#x}");
    }
}

#[test]
fn a_kept_word_is_remembered_only_where_it_could_be_corrected() {
    let mut learning_off = session(|c| c[6] = number(0));
    assert!(allows(&mut learning_off).1, "keeping is not learning");
    for change in [(7, 1), (0, 0x11), (0, 0x21), (0, 0xb1), (0, 0x81), (4, 0)] {
        let mut s = session(|c| c[change.0] = number(change.1));
        assert!(!allows(&mut s).1, "{change:?}");
    }
}
