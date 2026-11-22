//! ProjectLicenseScreen drawn natively: the asset asked for once, the unwrapped text scrolling
//! both ways with each axis clamped, long-press word selection, handles dragged across lines and
//! past the edges, Select all and Copy, the toolbar's place, TalkBack's selection and traversal,
//! the error text and the saved scrolls.
use render::{
    settings_ui::{
        Action, App, catalogue,
        project_licence::{ASSET, ERROR_KEY, ROUTE, SCROLL_ID, TEXT_ID},
    },
    text::Text,
    touch::Motion,
};
use wire::settings_ui::text::{effect, outcome, selection};

const LICENCE: &str =
    include_str!("../../../../../app/src/main/assets/license/project_license.txt");
const FRAME: i64 = 16_666_667;
const LONG_PRESS: i64 = 400_000_000;

fn roboto() -> Text {
    Text::new([include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()])
}
/// The screen at 240 by 640 px, density 1, with the licence loaded and drawn.
fn screen() -> (App, Text) {
    let mut a = App {
        config: [240.0, 640.0, 1.0, 1.0],
        ..App::default()
    };
    a.strings.insert(
        ERROR_KEY.into(),
        "Error: Failed to load license text.\n-> Reason: {error_message}".into(),
    );
    a.navigate("settings/about");
    a.navigate(ROUTE);
    let mut text = roboto();
    a.scene(&mut text);
    a.set_asset(&[ASSET.into(), outcome::OK.into(), LICENCE.into()])
        .unwrap();
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
fn touch(a: &mut App, action: i32, t: i64, x: f32, y: f32) {
    a.motion(&Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, t],
        coords: vec![x, y],
    });
}
/// A finger point inside the word at `bytes`: its middle, a little above its line's bottom.
fn inside(a: &App, start: usize, end: usize) -> (f32, f32) {
    let (left, y) = a.project_point(start).unwrap();
    let (right, _) = a.project_point(end).unwrap();
    ((left + right) / 2.0, y - 3.0)
}
/// Holds a finger at (`x`, `y`) past the long press timeout, frames running, and returns the
/// time it got to.
fn hold(a: &mut App, text: &mut Text, x: f32, y: f32) -> i64 {
    let t = 10 * LONG_PRESS;
    touch(a, 0, t, x, y);
    a.scene(text);
    assert!(a.animating(), "the pending long press asks for frames");
    let mut now = t;
    while now < t + LONG_PRESS + FRAME {
        now += FRAME;
        a.tick(now);
        a.scene(text);
    }
    now
}
fn selected(a: &App) -> Option<&str> {
    let (anchor, focus) = a.project_licence.selection?;
    LICENCE.get(anchor.min(focus)..anchor.max(focus))
}
fn handle(a: &App, start: bool) -> Option<(f32, f32)> {
    a.controls
        .iter()
        .find(|c| c.action == Action::SelectionHandle(start))
        .map(|c| {
            (
                (c.bounds.left + c.bounds.right) / 2.0,
                (c.bounds.top + c.bounds.bottom) / 2.0,
            )
        })
}
fn selection_op(a: &mut App, p: &[&str]) -> Result<(), &'static str> {
    a.text_selection(&p.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

#[test]
fn the_route_is_native_and_its_error_string_is_asked_for() {
    assert!(catalogue::native_routes().contains(&ROUTE));
    assert!(catalogue::native(ROUTE));
    assert!(App::string_keys().contains(&ERROR_KEY));
}

#[test]
fn the_screen_asks_kotlin_for_the_asset_once_and_draws_only_lines_on_screen() {
    let mut a = App {
        config: [240.0, 640.0, 1.0, 1.0],
        ..App::default()
    };
    a.navigate(ROUTE);
    let mut text = roboto();
    a.scene(&mut text);
    a.scene(&mut text);
    let asks: Vec<_> = a.effects.iter().filter(|e| e[0] == effect::ASSET).collect();
    assert_eq!(asks, [&vec![effect::ASSET.to_owned(), ASSET.to_owned()]]);
    assert!(!a.controls.iter().any(|c| c.id == TEXT_ID));
    a.set_asset(&[ASSET.into(), outcome::OK.into(), LICENCE.into()])
        .unwrap();
    let panel = a.scene(&mut text).panel.unwrap();
    let node = a.controls.iter().find(|c| c.id == TEXT_ID).unwrap();
    assert_eq!(node.label, LICENCE);
    assert_eq!(node.role, "selectable");
    let drawn: Vec<_> = panel.items.iter().filter(|i| i.text.mono).collect();
    let lines = LICENCE.lines().count();
    assert!(
        !drawn.is_empty() && drawn.len() < lines / 3,
        "{}",
        drawn.len()
    );
    assert_eq!(drawn[0].label, LICENCE.lines().next().unwrap());
    assert!(
        drawn
            .iter()
            .all(|i| i.text.size == 10.0 && i.text.weight == 400)
    );
    // The text is taller and wider than the viewport, so both axes scroll.
    assert!(a.max_scroll > 1000.0, "{}", a.max_scroll);
    assert!(
        a.project_licence.max_x > 50.0,
        "{}",
        a.project_licence.max_x
    );
    assert_eq!(a.node_can_scroll(SCROLL_ID), [false, true]);
    assert_eq!(a.node_can_scroll(4), [false, true]);
}

#[test]
fn a_failed_read_shows_the_error_string_and_bad_reports_are_refused() {
    let (mut a, mut text) = screen();
    a.set_asset(&[ASSET.into(), outcome::FAILED.into(), "boom".into()])
        .unwrap();
    a.scene(&mut text);
    let node = a.controls.iter().find(|c| c.id == TEXT_ID).unwrap();
    assert_eq!(
        node.label,
        "Error: Failed to load license text.\n-> Reason: boom"
    );
    // An exception without a message reads as an empty reason, as `it.message ?: ""` gives.
    a.set_asset(&[ASSET.into(), outcome::FAILED.into()])
        .unwrap();
    assert!(
        a.project_licence
            .text
            .as_deref()
            .unwrap()
            .ends_with("Reason: ")
    );
    assert!(
        a.set_asset(&["other.txt".into(), outcome::OK.into(), "x".into()])
            .is_err()
    );
    assert!(
        a.set_asset(&[ASSET.into(), "maybe".into(), "x".into()])
            .is_err()
    );
    assert!(a.set_asset(&[ASSET.into()]).is_err());
}

#[test]
fn each_axis_scrolls_alone_and_clamps_at_its_ends() {
    let (mut a, mut text) = screen();
    // Past the slop sideways first: the horizontal scroller owns the drag, by the over-slop.
    touch(&mut a, 0, FRAME, 100.0, 300.0);
    touch(&mut a, 2, 2 * FRAME, 91.0, 305.0);
    assert_eq!(a.project_licence.scroll_x, 1.0);
    touch(&mut a, 2, 3 * FRAME, 91.0, 200.0);
    assert_eq!(a.screen_scroll(), 0.0);
    for k in 4..40 {
        touch(&mut a, 2, k * FRAME, 91.0 - (k as f32) * 40.0, 200.0);
    }
    touch(&mut a, 1, 40 * FRAME, -1500.0, 200.0);
    a.scene(&mut text);
    assert_eq!(a.project_licence.scroll_x, a.project_licence.max_x);
    assert_eq!(a.screen_scroll(), 0.0);
    // Then a vertical drag far past the end: only the vertical scroll moves, and stops there.
    touch(&mut a, 0, 50 * FRAME, 100.0, 600.0);
    for k in 1..80 {
        touch(&mut a, 2, (50 + k) * FRAME, 100.0, 600.0 - k as f32 * 60.0);
    }
    touch(&mut a, 1, 130 * FRAME, 100.0, 600.0 - 79.0 * 60.0);
    let mut now = 130 * FRAME;
    while a.animating() && now < 400 * FRAME {
        now += FRAME;
        a.tick(now);
        a.scene(&mut text);
    }
    assert_eq!(a.screen_scroll(), a.max_scroll);
    assert_eq!(a.project_licence.scroll_x, a.project_licence.max_x);
    // And a horizontal fling back stops at the start.
    touch(&mut a, 0, 500 * FRAME, 20.0, 300.0);
    touch(&mut a, 2, 501 * FRAME, 120.0, 300.0);
    touch(&mut a, 2, 502 * FRAME, 220.0, 300.0);
    touch(&mut a, 1, 502 * FRAME, 220.0, 300.0);
    let mut now = 502 * FRAME;
    while a.animating() && now < 800 * FRAME {
        now += FRAME;
        a.tick(now);
        a.scene(&mut text);
    }
    assert_eq!(a.project_licence.scroll_x, 0.0);
    assert_eq!(a.screen_scroll(), a.max_scroll);
}

#[test]
fn a_long_press_selects_the_word_under_the_finger_and_a_tap_clears_it() {
    let (mut a, mut text) = screen();
    let apache = LICENCE.find("Apache").unwrap();
    let (x, y) = inside(&a, apache, apache + 6);
    let now = hold(&mut a, &mut text, x, y);
    assert_eq!(selected(&a), Some("Apache"));
    assert!(a.effects.iter().any(|e| e[0] == effect::HAPTIC));
    // No toolbar while the finger is still down; it shows at the lift, with Select all.
    assert_eq!(a.text_toolbar(), None);
    touch(&mut a, 1, now, x, y);
    a.scene(&mut text);
    let bar = a.text_toolbar().unwrap();
    assert_eq!(bar[4], 1.0);
    assert!(
        bar[0] < x && bar[2] > x && bar[1] < y && bar[3] > y,
        "{bar:?}"
    );
    assert!(handle(&a, true).is_some() && handle(&a, false).is_some());
    // A quick tap anywhere on the text clears it.
    touch(&mut a, 0, now + 10 * FRAME, 50.0, 400.0);
    touch(&mut a, 1, now + 12 * FRAME, 50.0, 400.0);
    a.scene(&mut text);
    assert_eq!(selected(&a), None);
    assert_eq!(a.text_toolbar(), None);
    assert!(handle(&a, true).is_none());
}

#[test]
fn a_long_press_between_words_keeps_one_character() {
    let (mut a, mut text) = screen();
    // Line 1 starts with 33 spaces.
    let (x, y) = inside(&a, 10, 11);
    hold(&mut a, &mut text, x, y);
    assert_eq!(selected(&a).map(str::len), Some(1));
}

#[test]
fn dragging_after_a_long_press_extends_by_whole_words() {
    let (mut a, mut text) = screen();
    let start = LICENCE.find("Version").unwrap();
    let (x, y) = inside(&a, start, start + 7);
    let now = hold(&mut a, &mut text, x, y);
    let january = LICENCE.find("January").unwrap();
    let (jx, jy) = inside(&a, january + 1, january + 2);
    touch(&mut a, 2, now + FRAME, jx, jy);
    assert_eq!(selected(&a), Some("Version 2.0, January"));
    // Back before the pressed word: the selection runs from its end back to whole words.
    let apache = LICENCE.find("Apache").unwrap();
    let (ax, ay) = inside(&a, apache + 2, apache + 3);
    touch(&mut a, 2, now + 2 * FRAME, ax, ay);
    let (anchor, focus) = a.project_licence.selection.unwrap();
    assert_eq!((anchor, focus), (start + 7, apache));
}

#[test]
fn handles_drag_across_lines_by_characters_and_may_cross() {
    let (mut a, mut text) = screen();
    let word = LICENCE.find("Version").unwrap();
    let (x, y) = inside(&a, word, word + 7);
    let now = hold(&mut a, &mut text, x, y);
    touch(&mut a, 1, now, x, y);
    a.scene(&mut text);
    // The end handle down to the "TERMS" line, a character into it.
    let (hx, hy) = handle(&a, false).unwrap();
    let terms = LICENCE.find("TERMS").unwrap();
    let (tx, ty) = a.project_point(terms + 1).unwrap();
    let (ex, ey) = a.project_point(word + 7).unwrap();
    touch(&mut a, 0, now + 10 * FRAME, hx, hy);
    assert_eq!(a.text_toolbar(), None, "no toolbar while a handle drags");
    touch(&mut a, 2, now + 11 * FRAME, hx + tx - ex, hy + ty - ey);
    let (anchor, focus) = a.project_licence.selection.unwrap();
    assert_eq!((anchor, focus), (word, terms + 1));
    // Lit as Android lights it: the URL line and the blank line between run edge to edge.
    let rows: Vec<_> = a
        .project_boxes()
        .iter()
        .map(|r| (r.left, r.right))
        .collect();
    assert_eq!(
        rows[1..],
        [(rows[3].0, rows[0].1), (rows[3].0, rows[0].1), rows[3]]
    );
    // On up above the start: the ends cross, and the dragged end leads.
    let apache = LICENCE.find("Apache").unwrap();
    let (px, py) = a.project_point(apache).unwrap();
    touch(&mut a, 2, now + 12 * FRAME, hx + px - ex, hy + py - ey);
    assert_eq!(a.project_licence.selection, Some((word, apache)));
    touch(&mut a, 1, now + 13 * FRAME, hx + px - ex, hy + py - ey);
    a.scene(&mut text);
    assert!(a.text_toolbar().is_some());
}

#[test]
fn a_handle_held_at_an_edge_scrolls_the_text_and_follows_it() {
    let (mut a, mut text) = screen();
    let word = LICENCE.find("Apache").unwrap();
    let (x, y) = inside(&a, word, word + 6);
    let now = hold(&mut a, &mut text, x, y);
    touch(&mut a, 1, now, x, y);
    a.scene(&mut text);
    let (hx, hy) = handle(&a, false).unwrap();
    let t = now + 10 * FRAME;
    touch(&mut a, 0, t, hx, hy);
    // Held still deep in the bottom band: frames keep scrolling down and the end follows.
    touch(&mut a, 2, t + FRAME, hx, 636.0);
    a.scene(&mut text);
    assert!(a.animating());
    let (_, held) = a.project_licence.selection.unwrap();
    let mut now = t + FRAME;
    for _ in 0..60 {
        now += FRAME;
        a.tick(now);
        a.scene(&mut text);
    }
    let down = a.screen_scroll();
    assert!(down > 200.0, "{down}");
    // The end moved on with the text, a line for every line scrolled under the still finger.
    let (_, focus) = a.project_licence.selection.unwrap();
    let lines = LICENCE[held..focus].matches('\n').count();
    assert!(lines > 10, "lines {lines}");
    // Held at the right edge: the text scrolls sideways too, up to its end.
    touch(&mut a, 2, now + FRAME, 239.0, 300.0);
    for _ in 0..120 {
        now += FRAME;
        a.tick(now);
        a.scene(&mut text);
    }
    assert_eq!(a.project_licence.scroll_x, a.project_licence.max_x);
    // Away from the edges it stops, and no frames are asked for.
    touch(&mut a, 2, now + FRAME, 120.0, 300.0);
    let scroll = a.screen_scroll();
    a.tick(now + 2 * FRAME);
    a.scene(&mut text);
    assert_eq!(a.screen_scroll(), scroll);
    assert!(!a.animating());
    // Into the header above the viewport: it scrolls back up.
    touch(&mut a, 2, now + 3 * FRAME, 120.0, 40.0);
    a.tick(now + 4 * FRAME);
    a.tick(now + 5 * FRAME);
    assert!(a.screen_scroll() < scroll);
}

#[test]
fn select_all_and_copy_put_the_text_on_the_clipboard_and_clear() {
    let (mut a, mut text) = screen();
    selection_op(&mut a, &[selection::SELECT_ALL]).unwrap();
    a.scene(&mut text);
    assert_eq!(selected(&a), Some(LICENCE));
    // Everything is selected, so the toolbar offers no Select all.
    assert_eq!(a.text_toolbar().unwrap()[4], 0.0);
    selection_op(&mut a, &[selection::COPY]).unwrap();
    assert_eq!(
        a.effects,
        [vec![effect::CLIP.to_owned(), LICENCE.to_owned()]]
    );
    a.scene(&mut text);
    assert_eq!(a.project_licence.selection, None);
    assert_eq!(a.text_toolbar(), None);
    // Nothing selected copies nothing.
    a.effects.clear();
    selection_op(&mut a, &[selection::COPY]).unwrap();
    assert!(a.effects.is_empty());
    assert!(selection_op(&mut a, &["paste"]).is_err());
}

#[test]
fn a_word_copies_alone() {
    let (mut a, mut text) = screen();
    let word = LICENCE.find("Version").unwrap();
    let (x, y) = inside(&a, word, word + 7);
    let now = hold(&mut a, &mut text, x, y);
    touch(&mut a, 1, now, x, y);
    a.effects.clear();
    selection_op(&mut a, &[selection::COPY]).unwrap();
    assert_eq!(
        a.effects,
        [vec![effect::CLIP.to_owned(), "Version".to_owned()]]
    );
}

#[test]
fn handles_and_the_toolbar_hide_when_the_selection_scrolls_away() {
    let (mut a, mut text) = screen();
    let word = LICENCE.find("Apache").unwrap();
    selection_op(
        &mut a,
        &[selection::SET, &word.to_string(), &(word + 6).to_string()],
    )
    .unwrap();
    a.scene(&mut text);
    assert!(handle(&a, true).is_some() && a.text_toolbar().is_some());
    a.set_scroll(300.0);
    a.scene(&mut text);
    assert!(handle(&a, true).is_none() && handle(&a, false).is_none());
    assert_eq!(a.text_toolbar(), None);
    assert_eq!(selected(&a), Some("Apache"));
}

#[test]
fn talkback_sets_the_selection_and_traverses_by_each_granularity() {
    let (mut a, _) = screen();
    let apache = LICENCE.find("Apache").unwrap();
    let first_newline = LICENCE.find('\n').unwrap();
    assert_eq!(a.node_selection(TEXT_ID), (-1, -1));
    let node = a.text_node(TEXT_ID).unwrap();
    assert_eq!(node["granularities"], 31);
    assert!(a.text_node(SCROLL_ID).is_none());
    // Word by word from the start, then back.
    selection_op(&mut a, &[selection::TRAVERSE, "2", "1", "0"]).unwrap();
    assert_eq!(
        a.project_licence.traversed,
        Some([apache as i32, apache as i32 + 6, 2, 256])
    );
    selection_op(&mut a, &[selection::TRAVERSE, "2", "1", "0"]).unwrap();
    let license = apache as i32 + 7;
    assert_eq!(
        a.project_licence.traversed,
        Some([license, license + 7, 2, 256])
    );
    assert_eq!(
        a.node_selection(TEXT_ID),
        (license as i64 + 7, license as i64 + 7)
    );
    selection_op(&mut a, &[selection::TRAVERSE, "1", "0", "0"]).unwrap();
    assert_eq!(
        a.project_licence.traversed,
        Some([license + 6, license + 7, 1, 512])
    );
    // Extending from a cursor selects what it passes over.
    selection_op(
        &mut a,
        &[selection::SET, &apache.to_string(), &apache.to_string()],
    )
    .unwrap();
    selection_op(&mut a, &[selection::TRAVERSE, "2", "1", "1"]).unwrap();
    assert_eq!(
        a.node_selection(TEXT_ID),
        (apache as i64, apache as i64 + 6)
    );
    assert_eq!(selected(&a), Some("Apache"));
    // Lines include their newline; paragraphs pass over blank lines.
    selection_op(&mut a, &[selection::SET, "0", "0"]).unwrap();
    selection_op(&mut a, &[selection::TRAVERSE, "4", "1", "0"]).unwrap();
    assert_eq!(
        a.project_licence.traversed,
        Some([0, first_newline as i32 + 1, 4, 256])
    );
    let terms = LICENCE.find("   TERMS").unwrap();
    let blank = LICENCE.find("\n\n").unwrap();
    selection_op(
        &mut a,
        &[selection::SET, &blank.to_string(), &blank.to_string()],
    )
    .unwrap();
    selection_op(&mut a, &[selection::TRAVERSE, "8", "1", "0"]).unwrap();
    let end = terms + LICENCE[terms..].find('\n').unwrap();
    assert_eq!(
        a.project_licence.traversed,
        Some([terms as i32, end as i32, 8, 256])
    );
    // A page is the lines the viewport holds.
    selection_op(&mut a, &[selection::SET, "0", "0"]).unwrap();
    selection_op(&mut a, &[selection::TRAVERSE, "16", "1", "0"]).unwrap();
    let [_, to, ..] = a.project_licence.traversed.unwrap();
    let lines = LICENCE[..to as usize].matches('\n').count();
    assert!((30..70).contains(&lines), "{lines}");
    // At the end there is nothing more, so TalkBack moves on.
    let len = LICENCE.len().to_string();
    selection_op(&mut a, &[selection::SET, &len, &len]).unwrap();
    selection_op(&mut a, &[selection::TRAVERSE, "1", "1", "0"]).unwrap();
    assert_eq!(a.project_licence.traversed, None);
    // A negative selection clears; one past the end is refused.
    selection_op(&mut a, &[selection::SET, "-1", "-1"]).unwrap();
    assert_eq!(a.node_selection(TEXT_ID), (-1, -1));
    let past = (LICENCE.len() + 1).to_string();
    selection_op(&mut a, &[selection::SET, "0", &past]).unwrap();
    assert_eq!(a.project_licence.selection, None);
    assert!(selection_op(&mut a, &[selection::TRAVERSE, "3", "1", "0"]).is_err());
    assert!(selection_op(&mut a, &[selection::SET, "x", "1"]).is_err());
}

#[test]
fn talkback_pages_the_horizontal_scroller_by_its_width() {
    let (mut a, mut text) = screen();
    a.accessibility_scroll(SCROLL_ID, true);
    a.scene(&mut text);
    let max = a.project_licence.max_x;
    assert_eq!(a.project_licence.scroll_x, max.min(240.0));
    assert_eq!(a.screen_scroll(), 0.0);
    assert_eq!(a.node_can_scroll(SCROLL_ID), [true, max > 240.0]);
    a.accessibility_scroll(SCROLL_ID, false);
    assert_eq!(a.project_licence.scroll_x, 0.0);
}

#[test]
fn keyboard_focus_reaches_the_text() {
    let (mut a, _) = screen();
    a.focus_next(false);
    a.focus_next(false);
    assert_eq!(a.focus, TEXT_ID);
}

#[test]
fn a_saved_state_restores_both_scrolls_once_the_text_is_back() {
    let (mut a, mut text) = screen();
    a.set_scroll(500.0);
    a.accessibility_scroll(SCROLL_ID, true);
    a.scene(&mut text);
    let (vertical, horizontal) = (a.screen_scroll(), a.project_licence.scroll_x);
    assert!(vertical == 500.0 && horizontal > 0.0);
    let saved = a.project_saved();
    // Recreated: the stack and its scrolls come back before the asset does.
    let mut b = App {
        config: a.config,
        stack: a.stack.clone(),
        scrolls: a.scrolls.clone(),
        ..App::default()
    };
    b.project_restored(saved.parse().unwrap());
    b.scene(&mut text);
    assert!(b.effects.iter().any(|e| e[0] == effect::ASSET));
    b.set_asset(&[ASSET.into(), outcome::OK.into(), LICENCE.into()])
        .unwrap();
    b.scene(&mut text);
    assert_eq!(b.screen_scroll(), vertical);
    assert_eq!(b.project_licence.scroll_x, horizontal);
}

#[test]
fn reopening_the_screen_starts_at_the_left_with_nothing_selected() {
    let (mut a, mut text) = screen();
    a.accessibility_scroll(SCROLL_ID, true);
    selection_op(&mut a, &[selection::SELECT_ALL]).unwrap();
    a.back();
    a.navigate(ROUTE);
    a.scene(&mut text);
    assert_eq!(a.project_licence.scroll_x, 0.0);
    assert_eq!(a.project_licence.selection, None);
    // The text stays loaded: no second read.
    assert!(!a.effects.iter().any(|e| e[0] == effect::ASSET));
}
