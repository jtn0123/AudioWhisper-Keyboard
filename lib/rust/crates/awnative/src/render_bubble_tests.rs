//! The native session's part in the press bubble: it styles the bubble a touch opens, restyles a
//! held one when the keys or their long-press choices change, and lays it out for the
//! orientation the headroom was asked for.
use crate::{
    boundary::args::RenderNativeSemanticConfig,
    render::{self, session},
    render_accessibility::semantic_config,
    render_input::{gestures, touch},
};

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn all(texts: &[&str]) -> Vec<Vec<u16>> {
    texts.iter().map(|t| utf16(t)).collect()
}

/// A renderer with q above a, keys 50 by 60 under a strip 20 tall, the desired key 50 by 60,
/// shifted as `shift` says, and the bubble setting on.
fn renderer() -> i64 {
    let native = render::create(&[]).unwrap();
    let json = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "key-popup-box":{"background":"#ff0000","foreground":"#000000"},
        "key-popup-box[shiftstate=`caps_lock`]":{"foreground":"#00ff00"}}"##;
    render::theme(native, &utf16(json), &[], &[], &[1.0, 1.0, 1.0]).unwrap();
    update(native, "unshifted");
    let values = [1., 32., 0., 0., 0., 0., 200., 32., 1900., 0., 0., 1.];
    gestures(native, &values, &all(&["NO_ACTION"; 9])).unwrap();
    native
}

fn update(native: i64, shift: &str) {
    let geometry = [
        0., 0., 50., 60., 0., 0., 50., 60., 1., 113., //
        0., 60., 50., 120., 0., 60., 50., 120., 1., 97.,
    ];
    let meta = all(&["characters", shift, "character", "character"]);
    let labels = all(&["q", "", "", "a", "", ""]);
    render::update(
        native,
        &[100., 140., 20., 50., 60.],
        &meta,
        &[],
        &geometry,
        &labels,
        &[],
    )
    .unwrap();
}

/// A finger goes down on a, or (`up`) lifts from it.
fn press(native: i64, up: bool) {
    touch(
        native,
        &[i32::from(up), 0, 7],
        &[1, 2],
        &[25., 110.],
        0,
        false,
    )
    .unwrap();
}

/// The bubble's foreground, whether it shows dots, and its height.
fn bubble(native: i64) -> Option<(u32, bool, f32)> {
    session(native, |s| {
        Ok(s.scene
            .bubble_view()
            .map(|(b, style, r)| (style.foreground, b.indicator.is_some(), r.height())))
    })
    .unwrap()
}

fn choices(native: i64, popups: &[i32]) {
    semantic_config(RenderNativeSemanticConfig {
        handle: native,
        names: &[],
        values: &[],
        text: &[],
        popups,
        removable: &[],
        checked: &[],
        editor: &[0, 0],
    })
    .unwrap();
}

#[test]
fn a_touch_styles_the_bubble_it_opens_and_a_release_takes_it_away() {
    let native = renderer();
    press(native, false);
    assert_eq!(bubble(native), Some((0xff000000, false, 150.0)));
    press(native, true);
    assert_eq!(bubble(native), None);
    render::close(native).unwrap();
}

#[test]
fn a_held_bubble_follows_the_keys_shift_and_choices() {
    let native = renderer();
    press(native, false);
    update(native, "caps_lock");
    assert_eq!(
        bubble(native).map(|b| b.0),
        Some(0xff00ff00),
        "restyled for caps lock"
    );
    choices(native, &[0, 1]);
    assert_eq!(
        bubble(native).map(|b| b.1),
        Some(true),
        "a's choices show its dots"
    );
    render::close(native).unwrap();
}

#[test]
fn the_headroom_orientation_lays_the_bubble_out() {
    let native = renderer();
    render::headroom(native, true).unwrap();
    press(native, false);
    // Three desired keys tall in landscape, not two and a half.
    assert_eq!(bubble(native).map(|b| b.2), Some(180.0));
    render::close(native).unwrap();
}
