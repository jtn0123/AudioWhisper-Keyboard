use wire::reading::effect::*;
use wire::reading::{from, message};

use super::*;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// A screen of `blocks` 100 high from the top, those in `list` in the list.
fn screen(blocks: &[&str], list: &[usize], visible: (i32, i32)) -> Screen {
    Screen {
        app: Some(utf16("app")),
        texts: blocks.iter().map(|it| utf16(it)).collect(),
        edges: (0..blocks.len() as i32)
            .map(|i| Edges {
                top: i * 100,
                bottom: i * 100 + 90,
            })
            .collect(),
        scrolls: (0..blocks.len()).map(|i| list.contains(&i)).collect(),
        visible: Edges {
            top: visible.0,
            bottom: visible.1,
        },
        nodes: 10,
        web: false,
        scroller: !list.is_empty(),
    }
}

/// A reading from `from` through a voice that loads and begins, up to its first walk.
fn begun(from: i32, selection: &str) -> Reading {
    let mut reading = Reading::new();
    assert_eq!(
        reading.start(from, utf16(selection)),
        Ok([SHOW_BUTTON, 0, 0])
    );
    assert_eq!(reading.done(), Ok([TAKE_VOICE, 0, 0]));
    assert_eq!(reading.answer(1), Ok([MAKE_VOICE, 1, 0]));
    assert_eq!(reading.answer(0), Ok([BEGIN, 0, 0]));
    let skip_own = i32::from(from == from::SELECTION);
    assert_eq!(reading.answer(1), Ok([WALK, skip_own, 0]));
    reading
}

fn dones(reading: &mut Reading, effects: &[Effect]) {
    for &expected in effects {
        assert_eq!(reading.done(), Ok(expected));
    }
}

#[test]
fn a_screen_is_read_from_its_first_block_in_sight_two_ahead() {
    let mut reading = begun(from::SCREEN, "");
    let found = reading.looked(Some(screen(&["a", "b", "c"], &[], (150, 400))));
    assert_eq!(found, Ok([KEEP, 0, 0]));
    dones(
        &mut reading,
        &[
            [FOLLOW, 0, 0],
            [AWAIT_BLOCKS, 1, 0],
            [QUEUE, 1, 0],
            [AWAIT_BLOCKS, 1, 0],
        ],
    );
    dones(
        &mut reading,
        &[
            [QUEUE, 2, 0],
            [AWAIT_BLOCKS, 1, 0],
            [AWAIT_BLOCKS, 0, 0],
            [UNFOLLOW, 0, 0],
        ],
    );
    dones(&mut reading, &[[DONE, 0, 0]]);
    assert!(reading.done().is_err());
}

#[test]
fn a_voice_that_failed_is_said_and_one_that_cannot_speak_ends_it() {
    let mut reading = Reading::new();
    reading.start(from::TOP, Vec::new()).unwrap();
    reading.done().unwrap();
    assert_eq!(reading.answer(2), Ok([TELL, message::VOICE_FAILED, 0]));
    assert_eq!(reading.done(), Ok([MAKE_VOICE, 0, 0]));
    assert_eq!(reading.answer(1), Ok([TELL, message::UNAVAILABLE, 1]));
    assert_eq!(reading.done(), Ok([DONE, 0, 0]));
    let mut refused = Reading::new();
    refused.start(from::SCREEN, Vec::new()).unwrap();
    refused.done().unwrap();
    refused.answer(3).unwrap();
    assert_eq!(refused.done(), Ok([MAKE_VOICE, 1, 0]));
    refused.answer(0).unwrap();
    assert_eq!(refused.answer(0), Ok([DONE, 0, 0]));
}

#[test]
fn a_screen_without_text_is_walked_eight_times_then_said() {
    let mut reading = begun(from::SCREEN, "");
    for walk in 1..=LOOK_TRIES {
        let empty = (walk % 2 == 0).then(|| screen(&[], &[], (0, 100)));
        assert_eq!(reading.looked(empty), Ok([WAIT, LOOK_RETRY_MS, 0]));
        let next = if walk < LOOK_TRIES {
            [WALK, 0, 0]
        } else {
            [TELL, message::NOTHING, 0]
        };
        assert_eq!(reading.done(), Ok(next));
    }
    assert_eq!(reading.done(), Ok([DONE, 0, 0]));
}

#[test]
fn a_web_page_all_in_sight_is_walked_again_while_it_grows() {
    let mut reading = begun(from::SCREEN, "");
    let mut page = screen(&["a"], &[], (0, 400));
    page.web = true;
    reading.looked(Some(page.clone())).unwrap();
    for grown in 1..=SETTLE_TRIES as i32 {
        assert_eq!(reading.done(), Ok([WAIT, LOOK_RETRY_MS, 0]));
        assert_eq!(reading.done(), Ok([WALK, 0, 0]));
        page.nodes += grown;
        assert_eq!(reading.looked(Some(page.clone())), Ok([KEEP, 0, 0]));
    }
    assert_eq!(reading.done(), Ok([FOLLOW, 0, 0]));
    let mut short = begun(from::SCREEN, "");
    let mut page = screen(&["a"], &[], (0, 400));
    page.web = true;
    short.looked(Some(page.clone())).unwrap();
    dones(&mut short, &[[WAIT, LOOK_RETRY_MS, 0], [WALK, 0, 0]]);
    assert_eq!(short.looked(Some(page)), Ok([FOLLOW, 0, 0]));
}

#[test]
fn the_list_scrolls_on_and_reading_carries_on_after_what_was_read() {
    let mut reading = begun(from::SCREEN, "");
    reading
        .looked(Some(screen(&["title", "one", "two"], &[1, 2], (0, 400))))
        .unwrap();
    dones(
        &mut reading,
        &[[FOLLOW, 0, 0], [AWAIT_BLOCKS, 1, 0], [QUEUE, 0, 0]],
    );
    dones(
        &mut reading,
        &[
            [AWAIT_BLOCKS, 1, 0],
            [QUEUE, 1, 0],
            [AWAIT_BLOCKS, 1, 0],
            [QUEUE, 2, 0],
        ],
    );
    dones(
        &mut reading,
        &[[AWAIT_BLOCKS, 1, 0], [SCROLL_FORWARD, 0, 0]],
    );
    assert_eq!(reading.answer(1), Ok([WAIT, SETTLE_MS, 0]));
    assert_eq!(reading.done(), Ok([WALK, 0, 0]));
    // Nothing new after the first look: looked for again, then the next look finds "three".
    reading
        .looked(Some(screen(&["one", "two"], &[0, 1], (0, 400))))
        .unwrap();
    dones(&mut reading, &[[WAIT, SETTLE_MS, 0], [WALK, 0, 0]]);
    reading
        .looked(Some(screen(&["x", "two", "three"], &[1, 2], (0, 400))))
        .unwrap();
    dones(
        &mut reading,
        &[[AWAIT_BLOCKS, 1, 0], [QUEUE, 2, 0], [AWAIT_BLOCKS, 1, 0]],
    );
    assert_eq!(reading.done(), Ok([SCROLL_FORWARD, 0, 0]));
    assert_eq!(reading.answer(0), Ok([AWAIT_BLOCKS, 0, 0]));
}

#[test]
fn another_app_after_a_scroll_ends_the_reading() {
    let mut reading = begun(from::SCREEN, "");
    reading
        .looked(Some(screen(&["one"], &[0], (0, 400))))
        .unwrap();
    dones(
        &mut reading,
        &[
            [FOLLOW, 0, 0],
            [AWAIT_BLOCKS, 1, 0],
            [QUEUE, 0, 0],
            [AWAIT_BLOCKS, 1, 0],
        ],
    );
    reading.done().unwrap();
    reading.answer(1).unwrap();
    reading.done().unwrap();
    let mut other = screen(&["two"], &[0], (0, 400));
    other.app = None;
    reading.looked(Some(other)).unwrap();
    assert_eq!(reading.done(), Ok([AWAIT_BLOCKS, 0, 0]));
}

#[test]
fn the_top_is_brought_into_sight_and_the_list_scrolled_back() {
    let mut reading = begun(from::TOP, "");
    reading
        .looked(Some(screen(&["title", "one"], &[1], (0, 400))))
        .unwrap();
    assert_eq!(reading.done(), Ok([REVEAL, 1, 0]));
    assert_eq!(reading.done(), Ok([SCROLL_BACK, 0, 0]));
    assert_eq!(reading.answer(1), Ok([WAIT, SCROLL_BACK_MS, 0]));
    assert_eq!(reading.done(), Ok([SCROLL_BACK, 0, 0]));
    assert_eq!(reading.answer(0), Ok([WAIT, SETTLE_MS, 0]));
    assert_eq!(reading.done(), Ok([WALK, 0, 0]));
    for _ in 0..LOOK_TRIES {
        reading.looked(None).unwrap();
        reading.done().unwrap();
    }
    // Nothing found: the page as it was, from the top.
    dones(&mut reading, &[[AWAIT_BLOCKS, 1, 0], [QUEUE, 0, 0]]);
}

#[test]
fn a_list_is_scrolled_back_sixty_times_at_most() {
    let mut reading = begun(from::TOP, "");
    reading
        .looked(Some(screen(&["one"], &[0], (0, 400))))
        .unwrap();
    assert_eq!(reading.done(), Ok([REVEAL, 0, 0]));
    // Main's number, not the constant, so a changed cap is a failed test.
    for _ in 0..60 {
        assert_eq!(reading.done(), Ok([SCROLL_BACK, 0, 0]));
        assert_eq!(reading.answer(1), Ok([WAIT, SCROLL_BACK_MS, 0]));
    }
    assert_eq!(reading.done(), Ok([WAIT, SETTLE_MS, 0]));
}

/// Only the last fifty blocks are kept to find the place after a scroll, however long the list.
#[test]
fn a_long_list_remembers_its_last_fifty_blocks() {
    let texts: Vec<String> = (0..120).map(|i| format!("block {i}")).collect();
    let blocks: Vec<&str> = texts.iter().map(String::as_str).collect();
    let list: Vec<usize> = (0..blocks.len()).collect();
    let mut reading = begun(from::SCREEN, "");
    reading
        .looked(Some(screen(&blocks, &list, (0, 400))))
        .unwrap();
    assert_eq!(reading.done(), Ok([FOLLOW, 0, 0]));
    for block in 0..blocks.len() as i32 {
        dones(&mut reading, &[[AWAIT_BLOCKS, 1, 0], [QUEUE, block, 0]]);
    }
    assert_eq!(reading.read.len(), 50);
    assert_eq!(reading.read.front(), Some(&utf16("block 70")));
}

#[test]
fn a_selection_is_found_or_spoken_by_itself() {
    let mut reading = begun(from::SELECTION, "two");
    reading
        .looked(Some(screen(&["one", "a two"], &[], (0, 400))))
        .unwrap();
    dones(
        &mut reading,
        &[[FOLLOW, 0, 0], [AWAIT_BLOCKS, 1, 0], [QUEUE, 1, 2]],
    );
    let mut missing = begun(from::SELECTION, "three");
    missing
        .looked(Some(screen(&["one"], &[], (0, 400))))
        .unwrap();
    let effects = [[SPEAK_SELECTION, 0, 0], [AWAIT_BLOCKS, 0, 0], [DONE, 0, 0]];
    dones(&mut missing, &effects);
}

#[test]
fn calls_out_of_turn_are_refused() {
    let mut reading = Reading::new();
    assert!(reading.done().is_err());
    assert!(reading.start(3, Vec::new()).is_err());
    reading.start(from::SCREEN, Vec::new()).unwrap();
    assert!(reading.answer(1).is_err());
    assert!(reading.looked(None).is_err());
    assert!(reading.start(from::SCREEN, Vec::new()).is_err());
}
