use super::*;
use crate::session::Candidate;

/// A plain text field with suggestions on: `window` is the whole text, the cursor at `cursor`
/// and the word being typed at `composing`.
fn context(window: &str, cursor: usize, composing: (usize, usize)) -> Vec<Field> {
    let mut c = vec![number(0); 22];
    c[0] = number(1);
    c[2] = text("org.florisboard.nlp.providers.latin");
    c[3] = text("en");
    c[4] = number(1);
    c[6] = number(1);
    c[9] = number(1);
    c[10] = number(cursor);
    c[11] = number(cursor);
    c[12] = number(composing.0);
    c[13] = number(composing.1);
    c[14] = text(window);
    c[17] = number(1);
    c
}

fn session(window: &str, cursor: usize, composing: (usize, usize)) -> Session {
    let mut session = Session::default();
    session.configure(crate::Context(context(window, cursor, composing)));
    session
}

fn call(session: &mut Session, op: &str, fields: &[&str]) -> Vec<Field> {
    let fields: Vec<Field> = fields.iter().map(|f| text(f)).collect();
    session.checked_call(op, &fields).unwrap()
}

/// "learn" for `word` kept as typed at `start`, finished by a space.
fn keep(session: &mut Session, word: &str, start: usize) {
    let start = start.to_string();
    call(session, "learn", &["0", "-1", word, "", &start, "1", ""]);
}

fn count(session: &Session, name: &str) -> u64 {
    session.counters.get(name).unwrap()
}

fn word(id: &str, kind: i64) -> Candidate {
    Candidate {
        id: text(id),
        text: text(id),
        kind,
        auto: false,
    }
}

#[test]
fn a_word_kept_as_typed_counts_once() {
    let mut s = session("hello", 5, (0, 5));
    keep(&mut s, "hello", 0);
    keep(&mut s, "hello", 0);
    assert_eq!((count(&s, "words"), count(&s, "kept_as_typed")), (1, 1));
    // An apostrophe goes on with the word; nothing finished.
    call(&mut s, "learn", &["0", "39", "hello", "", "0", "1", ""]);
    // No word, or no composing region to say where it was.
    call(&mut s, "learn", &["0", "-1", "", "", "0", "1", ""]);
    call(&mut s, "learn", &["0", "-1", "hello", "", "0", "0", ""]);
    assert_eq!(count(&s, "words"), 1);
    // Its space deleted and typed again, or a letter fixed, it is still the same word.
    call(&mut s, "tally", &["deleted", "6", "6"]);
    call(&mut s, "tally", &["key", "0", "0"]);
    keep(&mut s, "hello", 0);
    call(&mut s, "tally", &["deleted", "5", "5"]);
    call(&mut s, "tally", &["key", "0", "0"]);
    keep(&mut s, "help", 0);
    assert_eq!((count(&s, "words"), count(&s, "kept_as_typed")), (1, 1));
    // A word somewhere else is a new one, and so is one in another field.
    keep(&mut s, "there", 6);
    keep(&mut s, "hello", 0);
    call(&mut s, "field", &[]);
    keep(&mut s, "hello", 0);
    assert_eq!(count(&s, "words"), 4);
}

#[test]
fn counts_with_learning_from_typing_off() {
    let mut c = context("hello", 5, (0, 5));
    c[6] = number(0);
    let mut s = Session::default();
    s.configure(crate::Context(c));
    keep(&mut s, "hello", 0);
    assert_eq!(count(&s, "words"), 1);
}

#[test]
fn nothing_counts_incognito_or_in_a_password() {
    for (at, value) in [(7, 1), (0, 0x81)] {
        let mut c = context("hello", 5, (0, 5));
        c[at] = number(value);
        let mut s = Session::default();
        s.configure(crate::Context(c));
        keep(&mut s, "hello", 0);
        s.automatic = Some(word("x", 0));
        call(&mut s, "pick", &["x"]);
        call(&mut s, "tally", &["undone", "0", "0"]);
        assert_eq!(s.counters, Counters::default(), "field {at}");
    }
}

#[test]
fn an_auto_correction_and_its_undo() {
    let mut s = session("teh", 3, (0, 3));
    s.automatic = Some(Candidate {
        text: text("the"),
        ..word("a", 0)
    });
    assert_eq!(call(&mut s, "pick", &["a"]), [number(1)]);
    assert_eq!(count(&s, "autocorrected"), 1);
    // The backspace after it: the selection is past the space; then the undo itself.
    call(&mut s, "tally", &["deleted", "4", "4"]);
    call(&mut s, "tally", &["undone", "0", "0"]);
    assert_eq!(count(&s, "autocorrect_undone"), 1);
    assert_eq!(count(&s, "typos_left"), 0);
    // The undo key straight after a correction takes it back; after a keystroke it does not.
    s.automatic = Some(Candidate {
        text: text("the"),
        ..word("b", 0)
    });
    call(&mut s, "pick", &["b"]);
    call(&mut s, "tally", &["undo", "0", "0"]);
    assert_eq!(count(&s, "autocorrect_undone"), 2);
    s.automatic = Some(Candidate {
        text: text("the"),
        ..word("c", 0)
    });
    call(&mut s, "pick", &["c"]);
    call(&mut s, "tally", &["key", "0", "0"]);
    call(&mut s, "tally", &["undo", "0", "0"]);
    assert_eq!(
        (count(&s, "autocorrected"), count(&s, "autocorrect_undone")),
        (3, 2)
    );
}

#[test]
fn an_auto_correction_that_changes_nothing_is_not_one() {
    let mut s = session("the", 3, (0, 3));
    s.automatic = Some(word("the", 0));
    call(&mut s, "pick", &["the"]);
    assert_eq!(count(&s, "autocorrected"), 0);
}

#[test]
fn suggestions_tapped_by_slot() {
    let row = || ["l", "m", "r", "f"].map(|id| word(id, 0)).to_vec();
    // Typing a word.
    let mut s = session("hel", 3, (0, 3));
    for id in ["l", "m", "r", "f", "m"] {
        s.candidates = row();
        call(&mut s, "pick", &[id]);
    }
    let typing = ["left", "middle", "right", "further"]
        .map(|slot| count(&s, &format!("picked_typing_{slot}")));
    assert_eq!(typing, [1, 2, 1, 1]);
    assert_eq!(count(&s, "predicted"), 0);
    // After a space: a prediction.
    let mut s = session("hello ", 6, (6, 6));
    s.candidates = row();
    call(&mut s, "pick", &["r"]);
    assert_eq!(count(&s, "picked_after_space_right"), 1);
    assert_eq!(count(&s, "predicted"), 1);
    // At the start of the field: a prediction, but not after a space.
    let mut s = session("", 0, (0, 0));
    s.candidates = row();
    call(&mut s, "pick", &["l"]);
    assert_eq!(count(&s, "picked_after_space_left"), 0);
    assert_eq!(count(&s, "predicted"), 1);
    // An emoji, or a candidate not shown, is not a word suggestion.
    s.candidates = vec![word("e", 1)];
    call(&mut s, "pick", &["e"]);
    call(&mut s, "pick", &["gone"]);
    assert_eq!(count(&s, "predicted"), 1);
}

#[test]
fn a_swipe_replaced_by_a_suggestion_or_a_backspace() {
    let mut s = session("hello", 5, (0, 5));
    call(&mut s, "learn", &["2", "-1", "hello", "", "0", "1", ""]);
    assert_eq!((count(&s, "words"), count(&s, "swiped")), (1, 1));
    s.candidates = vec![word("hallo", 0)];
    call(&mut s, "pick", &["hallo"]);
    assert_eq!(count(&s, "swipe_replaced"), 1);
    // Deleted straight after.
    call(&mut s, "learn", &["2", "-1", "hello", "", "0", "1", ""]);
    call(&mut s, "tally", &["deleted", "6", "6"]);
    assert_eq!(count(&s, "swipe_replaced"), 2);
    // Typing on keeps it; so does the next word, and a prediction after it.
    call(&mut s, "learn", &["2", "-1", "hello", "", "0", "1", ""]);
    call(&mut s, "tally", &["key", "0", "0"]);
    call(&mut s, "tally", &["deleted", "8", "8"]);
    call(&mut s, "learn", &["2", "-1", "hello", "", "0", "1", ""]);
    keep(&mut s, "there", 6);
    call(&mut s, "tally", &["deleted", "20", "20"]);
    assert_eq!(count(&s, "swipe_replaced"), 2);
    // The undo key straight after a swipe takes it back.
    call(&mut s, "learn", &["2", "-1", "hello", "", "0", "1", ""]);
    call(&mut s, "tally", &["undo", "0", "0"]);
    assert_eq!((count(&s, "swiped"), count(&s, "swipe_replaced")), (5, 3));
}

#[test]
fn a_word_put_back_or_taken_back_counts_once() {
    // An auto-correction undone puts the word typed back in its place: one word, counted once,
    // even if it is finished again there.
    let mut s = session("teh", 3, (0, 3));
    s.automatic = Some(Candidate {
        text: text("the"),
        ..word("a", 0)
    });
    call(&mut s, "pick", &["a"]);
    call(&mut s, "learn", &["1", "-1", "the", "", "0", "1", ""]);
    call(&mut s, "tally", &["undone", "0", "0"]);
    assert_eq!(count(&s, "words"), 1);
    keep(&mut s, "teh", 0);
    assert_eq!((count(&s, "words"), count(&s, "kept_as_typed")), (1, 0));
    // So does the undo key's.
    s.automatic = Some(Candidate {
        text: text("the"),
        ..word("b", 0)
    });
    call(&mut s, "pick", &["b"]);
    call(&mut s, "learn", &["1", "-1", "the", "", "0", "1", ""]);
    call(&mut s, "tally", &["undo", "0", "0"]);
    keep(&mut s, "teh", 0);
    assert_eq!(count(&s, "words"), 1);
    // A swipe replaced by a suggestion, deleted, or undone: what replaces it counts instead.
    call(&mut s, "learn", &["2", "-1", "hello", "", "4", "1", ""]);
    s.candidates = vec![word("hallo", 0)];
    call(&mut s, "pick", &["hallo"]);
    call(&mut s, "learn", &["1", "-1", "hallo", "", "4", "1", ""]);
    assert_eq!(count(&s, "words"), 2);
    call(&mut s, "learn", &["2", "-1", "hello", "", "10", "1", ""]);
    call(&mut s, "tally", &["deleted", "16", "16"]);
    call(&mut s, "learn", &["2", "-1", "hello", "", "10", "1", ""]);
    call(&mut s, "tally", &["undo", "0", "0"]);
    assert_eq!(count(&s, "words"), 2);
    keep(&mut s, "there", 10);
    assert_eq!(count(&s, "words"), 3);
}

#[test]
fn a_typo_left_is_a_word_deleted_into_soon_after() {
    let mut s = session("hello", 5, (0, 5));
    keep(&mut s, "hello", 0);
    // "hello |": the space goes first, then the o.
    call(&mut s, "tally", &["deleted", "6", "6"]);
    assert_eq!(count(&s, "typos_left"), 0);
    call(&mut s, "tally", &["deleted", "5", "5"]);
    assert_eq!(count(&s, "typos_left"), 1);
    call(&mut s, "tally", &["deleted", "4", "4"]);
    assert_eq!(count(&s, "typos_left"), 1, "once per word");
    // A selection over part of the word.
    keep(&mut s, "hello", 0);
    call(&mut s, "tally", &["key", "0", "0"]);
    call(&mut s, "tally", &["deleted", "3", "9"]);
    assert_eq!(count(&s, "typos_left"), 2);
    // Too many keystrokes later it is an edit, not a typo left.
    call(&mut s, "tally", &["key", "0", "0"]);
    keep(&mut s, "hello", 0);
    for _ in 0..TYPO_KEYSTROKES {
        call(&mut s, "tally", &["key", "0", "0"]);
    }
    call(&mut s, "tally", &["deleted", "5", "5"]);
    // Nor once the field changed.
    call(&mut s, "tally", &["key", "0", "0"]);
    keep(&mut s, "hello", 0);
    call(&mut s, "field", &[]);
    call(&mut s, "tally", &["deleted", "5", "5"]);
    assert_eq!(count(&s, "typos_left"), 2);
}

#[test]
fn a_suggestion_tapped_is_a_word_its_typo_can_be_left_in() {
    let mut s = session("helo", 4, (0, 4));
    call(&mut s, "learn", &["1", "-1", "hello", "", "0", "1", ""]);
    assert_eq!((count(&s, "words"), count(&s, "kept_as_typed")), (1, 0));
    call(&mut s, "tally", &["deleted", "5", "5"]);
    assert_eq!(count(&s, "typos_left"), 1);
}

#[test]
fn saved_text_is_counts_only() {
    let mut s = session("secretword", 10, (0, 10));
    keep(&mut s, "secretword", 0);
    s.candidates = vec![word("secretly", 0)];
    call(&mut s, "pick", &["secretly"]);
    let saved = String::from_utf16(&call(&mut s, "tallyText", &[])[0]).unwrap();
    assert!(!saved.contains("secret"));
    let mut lines = saved.lines();
    assert_eq!(lines.next(), Some(HEADER));
    let names: Vec<&str> = lines
        .map(|line| {
            let (name, value) = line.split_once(' ').unwrap();
            assert!(value.bytes().all(|b| b.is_ascii_digit()), "{line}");
            name
        })
        .collect();
    assert_eq!(names, NAMES);
    assert!(saved.contains("\nwords 1\n") && saved.contains("\npicked_typing_left 1\n"));
}

#[test]
fn loads_saves_and_resets() {
    let mut s = session("hello", 5, (0, 5));
    keep(&mut s, "hello", 0);
    assert_eq!(call(&mut s, "tallyChanged", &[]), [number(1)]);
    assert_eq!(call(&mut s, "tallyChanged", &[]), [number(0)]);
    let saved = s.counters.text();
    call(&mut s, "tallyReset", &[]);
    assert_eq!(count(&s, "words"), 0);
    call(&mut s, "tallyLoad", &[&saved]);
    assert_eq!(s.counters.text(), saved);
    assert_eq!(call(&mut s, "tallyChanged", &[]), [number(0)]);
    // Unknown lines and bad values are skipped; another version's file is ignored.
    call(
        &mut s,
        "tallyLoad",
        &[&format!("{HEADER}\nwords 7\nwords x\nother 3\nswiped")],
    );
    assert_eq!((count(&s, "words"), count(&s, "swiped")), (7, 0));
    call(&mut s, "tallyLoad", &["typing-counters 9\nwords 7"]);
    assert_eq!(count(&s, "words"), 0);
}

#[test]
fn rejects_malformed_calls() {
    let mut s = Session::default();
    assert!(s.checked_call("tally", &[text("key")]).is_err());
    assert!(s.checked_call("tallyText", &[text("x")]).is_err());
    let fields = [text("jump"), text("0"), text("0")];
    assert!(s.checked_call("tally", &fields).is_err());
}
