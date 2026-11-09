use super::*;

#[test]
fn editing_keys_reach_the_search() {
    for code in [-7, -8, 32, 12288, 10] {
        assert_eq!(route(code, false), Route::Search, "{code}");
    }
    assert_eq!(route('a' as i32, true), Route::Search);
    assert_eq!(route('7' as i32, true), Route::Search);
}

#[test]
fn keyboard_keys_work_as_usual() {
    for code in [-11, -13, -201, -212, -232, -301, -244, -9713] {
        assert_eq!(route(code, false), Route::Keyboard, "{code}");
    }
}

#[test]
fn anything_else_does_nothing() {
    // Tab and escape are characters, but not search text.
    assert_eq!(route(9, true), Route::Ignore);
    assert_eq!(route(27, true), Route::Ignore);
    // Cursor keys, undo and the like, and keys not known yet.
    for code in [-21, -22, -131, -132, -9714, 0, i32::MIN] {
        assert_eq!(route(code, false), Route::Ignore, "{code}");
    }
}

#[test]
fn a_swipe_deletes_what_the_app_would_have_selected() {
    let delete = |swipe, units| swipe_delete(swipe, units);
    assert_eq!(
        delete(Swipe::DeleteCharacters, -3),
        SwipeDelete {
            count: 2,
            words: false
        }
    );
    assert_eq!(
        delete(Swipe::DeleteCharacters, 2),
        SwipeDelete {
            count: 0,
            words: false
        }
    );
    assert_eq!(
        delete(Swipe::DeleteWords, -5),
        SwipeDelete {
            count: 1,
            words: true
        }
    );
    assert_eq!(
        delete(Swipe::DeleteWords, -1),
        SwipeDelete {
            count: 0,
            words: true
        }
    );
    assert_eq!(
        delete(Swipe::Select, -9),
        SwipeDelete {
            count: 0,
            words: false
        }
    );
    // Int's negation wraps at its end, as Kotlin's does.
    assert_eq!(delete(Swipe::DeleteCharacters, i32::MIN).count, i32::MAX);
    assert_eq!(delete(Swipe::DeleteWords, i32::MIN).count, 0);
}
