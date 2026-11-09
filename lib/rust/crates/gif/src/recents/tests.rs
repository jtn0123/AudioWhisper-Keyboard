use super::From::{Before, Sent};
use super::*;
use crate::utf16;

fn ids(names: &[&str]) -> Vec<Vec<u16>> {
    names.iter().map(|it| utf16(it)).collect()
}

#[test]
fn a_gif_sent_goes_first_once() {
    assert_eq!(
        add(&ids(&["a", "b", "c"]), &utf16("b"), CAPACITY),
        [Sent(0), Before(0), Before(2)]
    );
    assert_eq!(add(&ids(&[]), &utf16("a"), CAPACITY), [Sent(0)]);
    // Every copy of it goes, as removeAll takes them all.
    assert_eq!(
        add(&ids(&["b", "a", "b"]), &utf16("b"), CAPACITY),
        [Sent(0), Before(1)]
    );
}

#[test]
fn the_oldest_go_beyond_the_capacity() {
    assert_eq!(add(&ids(&["a", "b"]), &utf16("c"), 2), [Sent(0), Before(0)]);
    assert_eq!(add(&ids(&["a"]), &utf16("c"), 0), []);
}

#[test]
fn loading_keeps_what_was_sent_meanwhile() {
    let saved = ids(&["x", "a", "y"]);
    // "b" was sent before "a", so "a" ends first; the saved "a" gives way to it.
    let sent = ids(&["a", "b"]);
    assert_eq!(
        load(&saved, &sent, CAPACITY),
        [Sent(0), Sent(1), Before(0), Before(2)]
    );
    assert_eq!(load(&saved, &ids(&[]), 2), [Before(0), Before(1)]);
    assert_eq!(load(&ids(&[]), &sent, 1), [Sent(0)]);
    // Saved duplicates stay, unless sent again.
    assert_eq!(
        load(&ids(&["x", "x"]), &ids(&[]), CAPACITY),
        [Before(0), Before(1)]
    );
}
