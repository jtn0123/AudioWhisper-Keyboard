use super::{layout, round, weighted};
use wire::ime_window::{facts as f, plan as p};

/// A Pixel-like portrait window: 1080 × 2400 at 420 dpi, gesture navigation, the default
/// settings, a four-row characters keyboard and a one-row suggestions-and-actions Smartbar.
fn facts() -> Vec<f32> {
    let mut a = vec![0.0; f::SIZE];
    for (i, v) in [
        (f::DENSITY, 2.625),
        (f::SYSTEM_DENSITY, 2.625),
        (f::WIDTH, 1080.0),
        (f::HEIGHT_MODE, 2.0),
        (f::HEIGHT, 2337.0),
        (f::SAFE_BOTTOM, 63.0),
        (f::OFFSET_PORTRAIT, 0.0),
        (f::OFFSET_LANDSCAPE, 0.0),
        (f::ONE_HANDED_SCALE, 87.0),
        (f::HEIGHT_PORTRAIT, 100.0),
        (f::HEIGHT_LANDSCAPE, 100.0),
        (f::DISPLAY_WIDTH, 1080.0),
        (f::DISPLAY_HEIGHT, 2400.0),
        (f::NAVIGATION_BAR, 63.0),
        (f::STATUS_BAR, 136.0),
        (f::MIN_FRACTION, 0.4),
        (f::MAX_FRACTION, 0.46),
        (f::BASE_HEIGHT, 541.0),
        (f::ROWS, 4.0),
        (f::CHARACTER_ROWS, 4.0),
    ] {
        a[i] = v;
    }
    // Enabled, the shared layout, the default flags, density and row left for Rust.
    a.extend([1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 3.0, 0.0, 0.0, 0.0]);
    a.extend([0.0; 5]);
    a
}

fn plan(a: &[f32]) -> Vec<f32> {
    layout(a).expect("a plan")
}

fn with(changes: &[(usize, f32)]) -> Vec<f32> {
    let mut a = facts();
    for (i, v) in changes {
        a[*i] = *v;
    }
    a
}

#[test]
fn halves_round_up_as_compose_rounds_pixels() {
    assert_eq!((round(2.5), round(2.499_999_8), round(-2.5)), (3, 2, -2));
}

#[test]
fn a_row_hands_out_what_rounding_lost_from_the_start() {
    assert_eq!(weighted(1080, &[1.0]), vec![1080]);
    assert_eq!(weighted(1001, &[1.0 / 3.0; 3]), vec![333, 334, 334]);
    let sizes = weighted(1080, &[0.13, 0.87]);
    assert_eq!(sizes.iter().sum::<i32>(), 1080);
    assert_eq!(sizes, vec![140, 940]);
    assert_eq!(weighted(-5, &[1.0]), vec![0]);
}

#[test]
fn the_keyboard_fills_the_safe_width_at_the_window_bottom() {
    let out = plan(&with(&[(f::SAFE_LEFT, 30.0), (f::SAFE_RIGHT, 50.0)]));
    assert_eq!(out[p::KEYBOARD_LEFT], 30.0);
    assert_eq!(out[p::KEYBOARD_WIDTH], 1000.0);
    assert_eq!((out[p::START_WIDTH], out[p::END_WIDTH]), (0.0, 0.0));
    let rows = out[p::ROW_HEIGHT] * 4.0;
    assert!((out[p::KEYBOARD_PX] - rows).abs() < 0.01);
    let box_height = out[p::KEYBOARD_HEIGHT] + 63.0;
    assert_eq!(out[p::BOX_HEIGHT], box_height);
    assert_eq!(out[p::VIEW_HEIGHT], 2337.0);
    assert_eq!(out[p::BOX_TOP], 2337.0 - box_height);
}

#[test]
fn row_heights_follow_the_input_view_height_and_the_height_setting() {
    let out = plan(&facts());
    // ((0.4 × 1080 + 0.46 × 2201) / 2) × 0.21 = 151.67…, and the Smartbar 0.753 of it.
    assert_eq!(out[p::ROW_PIXELS], 151.0);
    assert_eq!(out[p::SMARTBAR_PIXELS], 114.0);
    let taller = plan(&with(&[(f::HEIGHT_PORTRAIT, 120.0)]));
    assert_eq!(taller[p::ROW_PIXELS], 182.0);
    assert!(taller[p::KEYBOARD_HEIGHT] > out[p::KEYBOARD_HEIGHT]);
}

#[test]
fn one_handed_mode_puts_a_panel_on_the_free_side_in_portrait_only() {
    let end = plan(&with(&[(f::ONE_HANDED, 1.0), (f::ONE_HANDED_END, 1.0)]));
    assert_eq!((end[p::START_LEFT], end[p::START_WIDTH]), (0.0, 140.0));
    assert_eq!(
        (end[p::KEYBOARD_LEFT], end[p::KEYBOARD_WIDTH]),
        (140.0, 940.0)
    );
    assert_eq!(end[p::END_WIDTH], 0.0);
    // The panel is as tall as the keyboard and Smartbar together.
    assert!(end[p::START_HEIGHT] >= end[p::KEYBOARD_HEIGHT]);
    let start = plan(&with(&[(f::ONE_HANDED, 1.0)]));
    assert_eq!(
        (start[p::KEYBOARD_LEFT], start[p::KEYBOARD_WIDTH]),
        (0.0, 940.0)
    );
    assert_eq!((start[p::END_LEFT], start[p::END_WIDTH]), (940.0, 140.0));
    let landscape = plan(&with(&[(f::ONE_HANDED, 1.0), (f::LANDSCAPE, 1.0)]));
    assert_eq!(landscape[p::KEYBOARD_WIDTH], 1080.0);
    assert_eq!(
        (landscape[p::START_WIDTH], landscape[p::END_WIDTH]),
        (0.0, 0.0)
    );
}

#[test]
fn a_panel_taller_than_the_keys_sets_the_rows_height() {
    // A GIF search's bar is one row; the settings' extended actions keep the panel two rows tall,
    // and the box follows the panel, as Compose's row did.
    let mut a = with(&[
        (f::ONE_HANDED, 1.0),
        (f::ONE_HANDED_END, 1.0),
        (f::SEARCH, 1.0),
    ]);
    a[f::SIZE + 1] = 3.0;
    a[f::SIZE + 4] = 1.0;
    let out = plan(&a);
    assert!(out[p::START_HEIGHT] > out[p::KEYBOARD_HEIGHT]);
    assert_eq!(out[p::BOX_HEIGHT], out[p::START_HEIGHT] + 63.0);
}

#[test]
fn rows_are_never_shorter_than_the_base_height_or_fewer_than_four() {
    // A base height above the screen's fractions wins: 2000 × 0.21.
    let based = plan(&with(&[(f::BASE_HEIGHT, 2000.0)]));
    assert!((based[p::ROW_HEIGHT] - 420.0).abs() < 0.01);
    let four = plan(&facts());
    let three = plan(&with(&[(f::ROWS, 3.0), (f::CHARACTER_ROWS, 3.0)]));
    assert_eq!(three[p::KEYBOARD_PX], four[p::KEYBOARD_PX]);
    let five = plan(&with(&[(f::ROWS, 5.0), (f::CHARACTER_ROWS, 5.0)]));
    assert!(five[p::KEYBOARD_PX] > four[p::KEYBOARD_PX]);
}

#[test]
fn the_bottom_offset_of_the_orientation_lifts_the_keyboard() {
    let out = plan(&facts());
    let lifted = plan(&with(&[
        (f::OFFSET_PORTRAIT, 10.0),
        (f::OFFSET_LANDSCAPE, 99.0),
    ]));
    assert_eq!(lifted[p::BOX_HEIGHT] - out[p::BOX_HEIGHT], 26.0);
}

#[test]
fn a_full_screen_extract_view_drops_the_spacer() {
    let out = plan(&with(&[
        (f::FULLSCREEN, 1.0),
        (f::EXTRACT_SHOWN, 1.0),
        (f::HEIGHT_MODE, 0.0),
    ]));
    assert_eq!(out[p::BOX_TOP], 0.0);
    assert_eq!(out[p::VIEW_HEIGHT], out[p::BOX_HEIGHT]);
    let bounded = plan(&with(&[
        (f::FULLSCREEN, 1.0),
        (f::HEIGHT_MODE, 1.0),
        (f::HEIGHT, 900.0),
    ]));
    assert_eq!(bounded[p::VIEW_HEIGHT], 900.0);
    assert_eq!(bounded[p::BOX_TOP], 900.0 - bounded[p::BOX_HEIGHT]);
}

#[test]
fn the_surface_rises_only_under_a_full_screen_extract_view() {
    let extract = plan(&with(&[(f::FULLSCREEN, 1.0), (f::EXTRACT_SHOWN, 1.0)]));
    assert_eq!(extract[p::RISE], 1.0);
    for changes in [&[(f::FULLSCREEN, 1.0)][..], &[(f::EXTRACT_SHOWN, 1.0)], &[]] {
        assert_eq!(plan(&with(changes))[p::RISE], 0.0);
    }
}

#[test]
fn each_panel_is_named_as_the_renderer_numbers_it() {
    let kind = |changes: &[(usize, f32)]| plan(&with(changes))[p::PANEL];
    assert_eq!(kind(&[]), 0.0);
    assert_eq!(kind(&[(f::UI_MODE, 1.0)]), 1.0);
    assert_eq!(kind(&[(f::UI_MODE, 1.0), (f::GIFS, 1.0)]), 5.0);
    assert_eq!(kind(&[(f::UI_MODE, 2.0)]), 4.0);
    assert_eq!(kind(&[(f::UI_MODE, 2.0), (f::OVERFLOW, 1.0)]), 2.0);
    assert_eq!(kind(&[(f::OVERFLOW, 1.0), (f::EDITOR, 1.0)]), 3.0);
}

#[test]
fn the_surface_reaches_up_the_screen_under_the_actions_editor() {
    let out = plan(&facts());
    assert_eq!(out[p::SURFACE_TOP], 0.0);
    assert_eq!(out[p::SURFACE_HEIGHT], out[p::KEYBOARD_HEIGHT]);
    let editor = plan(&with(&[(f::EDITOR, 1.0)]));
    assert_eq!(editor[p::SURFACE_HEIGHT], 2201.0);
    assert_eq!(editor[p::SURFACE_TOP], editor[p::KEYBOARD_HEIGHT] - 2201.0);
}

#[test]
fn the_smartbar_gets_rusts_row_and_a_search_gets_its_own_bar() {
    let out = plan(&facts());
    let config = &out[p::SIZE..];
    assert_eq!(config.len(), 17);
    assert_eq!(config[10], 2.625);
    assert!((config[11] - out[p::SMARTBAR_PIXELS]).abs() < 1.0);
    assert_eq!(config[11], out[p::SMARTBAR_ROW]);
    let search = plan(&with(&[(f::SEARCH, 1.0)]));
    let config = &search[p::SIZE..];
    assert_eq!(
        (config[0], config[1], config[6], config[9]),
        (1.0, 0.0, 0.0, 0.0)
    );
    assert_eq!(config[11], search[p::SMARTBAR_ROW]);
    assert_eq!(search[p::KEYBOARD_HEIGHT], out[p::KEYBOARD_HEIGHT]);
}

#[test]
fn inline_chips_take_the_strip_only_with_no_panel_or_search() {
    let mut a = facts();
    a[f::SIZE + 12] = 1.0;
    let out = plan(&a);
    assert!(out[p::INLINE_WIDTH] > 0.0);
    assert_eq!(out[p::INLINE_TOP], 0.0);
    assert!(out[p::INLINE_HEIGHT] > 0.0 && out[p::INLINE_HEIGHT] <= out[p::STRIP_PX] + 0.5);
    for (i, v) in [(f::UI_MODE, 1.0), (f::SEARCH, 1.0)] {
        let mut hidden = a.clone();
        hidden[i] = v;
        assert_eq!(plan(&hidden)[p::INLINE_WIDTH], 0.0);
    }
    assert_eq!(plan(&facts())[p::INLINE_WIDTH], 0.0);
}

#[test]
fn facts_that_are_cut_short_or_not_numbers_have_no_plan() {
    assert_eq!(layout(&facts()[..f::SIZE + 5]), None);
    assert_eq!(layout(&with(&[(f::WIDTH, f32::NAN)])), None);
    assert_eq!(layout(&with(&[(f::DENSITY, 0.0)])), None);
}
