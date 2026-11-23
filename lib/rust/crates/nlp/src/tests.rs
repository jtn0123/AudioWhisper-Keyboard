//! Which kept words may teach where this user taps; and what each kind of field allows:
//! suggestions, auto-correction, learning and remembering a word kept as typed.

use crate::{Context, Field, Session, number, text};
use wire::nlp::{self as ops, context};
use wire::typing::row;

fn session(change: impl FnOnce(&mut Vec<Field>)) -> Session {
    let mut c = vec![number(0); context::SIZE];
    c[context::INPUT_TYPE] = number(1);
    c[context::PROVIDER] = text("org.florisboard.nlp.providers.latin");
    c[context::LANGUAGE] = text("en");
    c[context::SUGGESTIONS] = number(1);
    c[context::READY] = number(1);
    change(&mut c);
    let mut session = Session::default();
    session.configure(Context(c));
    session
}

/// Whether a word finished as typed may teach the aim: the setting, the trigger, the word and
/// whether the composing region is valid, in an event row as the typing plan reads them.
fn aim(session: &Session, [setting, trigger, word, composing_valid]: [&str; 4]) -> bool {
    let mut f: Vec<Field> = vec![vec![]; row::SIZE];
    f[row::AIM] = text(setting);
    f[row::TRIGGER] = text(trigger);
    f[row::WORD] = text(word);
    f[row::COMPOSING_VALID] = text(composing_valid);
    session.aim(&f)
}

#[test]
fn a_kept_word_teaches_the_aim_only_when_allowed() {
    let plain = session(|_| {});
    assert!(aim(&plain, ["1", "-1", "pop", "1"]));
    assert!(
        aim(&plain, ["1", "46", "pop", "1"]),
        "a full stop ends the word"
    );
    // The setting is off; the word goes on; nothing was typed; no composing region.
    assert!(!aim(&plain, ["0", "-1", "pop", "1"]));
    assert!(!aim(&plain, ["1", "39", "pop", "1"]));
    assert!(!aim(&plain, ["1", "-1", "", "1"]));
    assert!(!aim(&plain, ["1", "-1", "pop", "0"]));
    // It needs no "learn from typing": that is about words, this about taps.
    let learning_off = session(|c| c[context::LEARN] = number(0));
    assert!(aim(&learning_off, ["1", "-1", "pop", "1"]));
    // Incognito, a raw editor, a password, an e-mail address (even one asking for corrections),
    // or no suggestions.
    for change in [
        (context::INCOGNITO, 1),
        (context::RAW_EDITOR, 1),
        (context::INPUT_TYPE, 0x81),
        (context::INPUT_TYPE, 0x21),
        (context::INPUT_TYPE, 0x8021),
        (context::SUGGESTIONS, 0),
    ] {
        let s = session(|c| c[change.0] = number(change.1));
        assert!(!aim(&s, ["1", "-1", "pop", "1"]), "{change:?}");
    }
}

/// (on, words, emoji, auto-correct, learn) and whether a kept word is remembered.
fn allows(session: &mut Session) -> ([bool; 5], bool) {
    let policy = session
        .checked_call(ops::POLICY, &session.context.0.clone())
        .unwrap();
    let keep = session.checked_call(ops::KEEP, &[text("zorb")]).unwrap() == [number(1)];
    (std::array::from_fn(|i| policy[i] == number(1)), keep)
}

#[test]
fn each_kind_of_field_allows_what_it_should() {
    let prose = ([true, true, false, true, true], true);
    let not_prose = ([true, true, false, false, false], false);
    // Corrected because it asks, but nothing of it learned or remembered.
    let address_corrected = ([true, true, false, true, false], false);
    let secret = ([false, false, false, false, false], false);
    // (inputType, imeOptions, what it allows)
    for (input, options, expected) in [
        (0x1, 0, prose),
        (0x4001, 0, prose),
        // A search box: Gboard corrects in Settings' search, so this keyboard does too.
        (0x10_0001, 0x1000_0003, prose),
        (0x11, 0, not_prose),
        (0x21, 0, not_prose),
        // An address field that asks for auto-correction gets it, as on Gboard (Contacts'
        // e-mail field is 0x8021), and is still never learned from.
        (0x8011, 0, address_corrected),
        (0x8021, 0, address_corrected),
        (0x80d1, 0, address_corrected),
        (0xd1, 0, not_prose),
        (0x200b1, 0x4000_0006, not_prose),
        // A filter is never corrected, whatever it asks; nor is a password.
        (0x80b1, 0, not_prose),
        (0x8081, 0, secret),
        (0x81, 0, secret),
        (0x91, 0, secret),
        (0xe1, 0, secret),
        (0x12, 0, secret),
        (0x80001, 0, secret),
    ] {
        let mut s = session(|c| {
            c[context::INPUT_TYPE] = number(input);
            c[context::LEARN] = number(1);
            c[context::IME_OPTIONS] = number(options);
        });
        assert_eq!(allows(&mut s), expected, "{input:#x} {options:#x}");
    }
}

#[test]
fn a_kept_word_is_remembered_only_in_prose() {
    let mut learning_off = session(|c| c[context::LEARN] = number(0));
    assert!(allows(&mut learning_off).1, "keeping is not learning");
    for change in [
        (context::INCOGNITO, 1),
        (context::INPUT_TYPE, 0x11),
        (context::INPUT_TYPE, 0x21),
        (context::INPUT_TYPE, 0x8011),
        (context::INPUT_TYPE, 0x8021),
        (context::INPUT_TYPE, 0xb1),
        (context::INPUT_TYPE, 0x81),
        (context::SUGGESTIONS, 0),
    ] {
        let mut s = session(|c| c[change.0] = number(change.1));
        assert!(!allows(&mut s).1, "{change:?}");
    }
}

#[test]
fn an_unknown_operation_is_refused() {
    let mut s = Session::default();
    assert_eq!(s.checked_call("unknown", &[]), Err("unknown NLP operation"));
}
