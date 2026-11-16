use super::*;
use crate::Content;

/// What the editor expects after an edit leaves `text` with the cursor at its end.
fn expected(value: &str) -> Content {
    let text = crate::text(value);
    let cursor = text.len() as i32;
    Content {
        text,
        offset: 0,
        selection: Range::cursor(cursor),
        composing: Range::NONE,
        word: Range::NONE,
    }
}

fn update(cursor: i32) -> Vec<Field> {
    [cursor, cursor, -1, -1, 1].map(number).to_vec()
}

fn late(session: &mut Session, cursor: i32) -> bool {
    session.checked_call("late", &update(cursor)[..4]) == Ok(vec![number(1)])
}

/// "I saw teh" corrected to "the", a space typed, then a backspace that undid it, all before the
/// app reported any of it: the cursor went 9, 10, 9.
fn undone_before_any_update() -> Session {
    let mut session = Session::new();
    let edits = ["I saw the", "I saw the ", "I saw teh"];
    session.queue.extend(edits.map(expected));
    session
}

#[test]
fn an_update_for_an_edit_since_followed_by_more_is_late() {
    let mut session = undone_before_any_update();
    assert!(late(&mut session, 9));
    session.call("update", &update(9));
    assert!(late(&mut session, 10));
    session.call("update", &update(10));
    // The undo's own update, though its cursor is where the correction's was, is the news.
    assert!(!late(&mut session, 9));
    session.call("update", &update(9));
    assert!(session.queue.is_empty());
}

#[test]
fn an_update_the_keyboard_did_not_cause_is_never_late() {
    let mut session = undone_before_any_update();
    assert!(!late(&mut session, 4));
    assert!(!late(&mut Session::new(), 9));
    session.mass = 1;
    assert!(!late(&mut session, 9));
}
