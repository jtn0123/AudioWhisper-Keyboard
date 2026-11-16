//! The "aim" operation: which kept words may teach where this user taps.

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
