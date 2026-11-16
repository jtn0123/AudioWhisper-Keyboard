//! AudioWhisper's roomy Smartbar: while words show, the main row is Gboard's (smartbar_gboard.rs
//! has its ends); here, what stays as it is, and how a word shrinks to fit before it ellipsises.
use render::{
    raster::Raster,
    scene::{CandidateStyle, Scene},
    smartbar::{self, Config, Layout, MIN_TEXT_SCALE, TEXT_FIT_STEPS, fit_text, text_scale},
    smartbar_scene::Smartbar,
    style::Style,
    text::Text,
};

/// The phone's defaults: shared layout, flipped toggles, speaker and sticky mic shown.
fn facts() -> [f32; 17] {
    [
        1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 19.0, 0.0, 1.0, 40.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]
}
fn decode(a: &[f32]) -> Config {
    Config::decode(a).unwrap()
}
fn row(c: &Config) -> Layout {
    smartbar::layout(c, 400.0, &[50.0; 3], 0.0)
}

#[test]
fn an_empty_row_keeps_todays_controls_and_auto_expansion() {
    let mut a = facts();
    for layout in [2.0, 3.0] {
        a[1] = layout;
        let idle = decode(&a);
        assert!(!idle.roomy());
        let empty = smartbar::layout(&idle, 400.0, &[], 0.0);
        let ids: Vec<_> = empty.actions.iter().map(|s| s.id).collect();
        assert_eq!(ids, [0, 1, 4], "mic, speaker and arrow; no pinned emoji");
    }
    // Shared and expanded with no words: the actions toolbar, between the same pinned controls.
    (a[1], a[3]) = (2.0, 1.0);
    let expanded = smartbar::layout(&decode(&a), 400.0, &[], 0.0);
    assert!(expanded.actions.len() > 3);
}

#[test]
fn other_layouts_inline_autofill_and_expanded_actions_are_unchanged() {
    // (layout, shared, inline)
    for (layout, shared, inline) in [
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (0.0, 0.0, 1.0),
        (1.0, 0.0, 1.0),
        (2.0, 1.0, 0.0),
        (2.0, 1.0, 1.0),
        (2.0, 0.0, 1.0),
        (3.0, 0.0, 1.0),
    ] {
        let mut a = facts();
        (a[1], a[3], a[12]) = (layout, shared, inline);
        let idle = decode(&a);
        a[14] = 1.0;
        let words = decode(&a);
        assert!(
            !words.roomy(),
            "layout={layout} shared={shared} inline={inline}"
        );
        assert_eq!(
            row(&idle),
            row(&words),
            "layout={layout} shared={shared} inline={inline}"
        );
    }
}

#[test]
fn dictation_and_reading_with_the_speaker_keep_todays_row() {
    for layout in [2.0, 3.0] {
        for flip in [0.0, 1.0] {
            let mut a = facts();
            (a[1], a[2], a[14]) = (layout, flip, 1.0);
            let today = |a: &[f32; 17]| {
                let mut idle = *a;
                idle[14] = 0.0;
                row(&decode(&idle))
            };
            for (at, speaker, roomy) in [
                (15, 1.0, false), // dictating: the mic shows its level and stop
                (15, 0.0, false),
                (16, 1.0, false), // reading: the speaker is the stop
                (16, 0.0, true),  // reading without a speaker: nothing to keep
                (13, 1.0, false), // dictation preview shows in the row
            ] {
                let mut b = a;
                (b[at], b[6]) = (1.0, speaker);
                let c = decode(&b);
                let case = format!("layout={layout} flip={flip} fact={at} speaker={speaker}");
                assert_eq!(c.roomy(), roomy, "{case}");
                if !roomy && at != 13 {
                    assert_eq!(row(&c), today(&b), "{case}");
                    assert!(
                        row(&c)
                            .actions
                            .iter()
                            .any(|s| s.id == if at == 15 { 0 } else { 1 })
                    );
                }
            }
        }
    }
}

#[test]
fn an_expanded_extended_row_keeps_its_actions_while_words_show_in_the_main_row() {
    for placement in [0.0, 1.0, 2.0] {
        for flip in [0.0, 1.0] {
            let mut a = facts();
            (a[1], a[2], a[4], a[5]) = (3.0, flip, 1.0, placement);
            let before = row(&decode(&a));
            a[14] = 1.0;
            let words = decode(&a);
            let after = row(&words);
            let top = before.center.top;
            let extended: Vec<_> = before
                .actions
                .iter()
                .filter(|s| s.bounds.top != top)
                .cloned()
                .collect();
            assert!(extended.len() > 4);
            let (main, second): (Vec<_>, Vec<_>) = after
                .actions
                .iter()
                .cloned()
                .partition(|s| s.bounds.top == top);
            assert_eq!(second, extended, "placement={placement} flip={flip}");
            let ids: Vec<_> = main.iter().map(|s| s.id).collect();
            assert_eq!(ids, if flip == 1.0 { [0, 4] } else { [4, 0] });
            assert_eq!(after.visible_dynamic, before.visible_dynamic);
            let center = if flip == 1.0 {
                (40.0, 370.0)
            } else {
                (30.0, 360.0)
            };
            assert_eq!(
                (after.center.left, after.center.top, after.center.right),
                (center.0, top, center.1)
            );
            assert_eq!(words.height(), 80.0);
        }
    }
}

#[test]
fn decode_takes_the_new_facts_only_as_a_full_seventeen() {
    let mut a = facts().to_vec();
    a.push(0.0);
    for len in 11..=18 {
        assert_eq!(
            Config::decode(&a[..len]).is_some(),
            [12, 14, 17].contains(&len),
            "{len}"
        );
    }
    let mut b = facts();
    (b[14], b[15], b[16]) = (1.0, 1.0, 1.0);
    let c = decode(&b);
    assert!(c.words && c.dictating && c.reading);
    let short = decode(&facts()[..14]);
    assert!(!short.words && !short.dictating && !short.reading && !short.roomy());
}

#[test]
fn words_shrink_in_whole_percents_down_to_three_quarters() {
    assert_eq!(MIN_TEXT_SCALE, 0.75);
    assert_eq!(text_scale(1.0, 100.0, 100.0), 1.0);
    assert_eq!(text_scale(1.0, 120.0, 100.0), 0.83);
    assert_eq!(text_scale(1.0, 400.0, 100.0), 0.75);
    assert_eq!(text_scale(0.8, 90.0, 100.0), 0.8);
    assert_eq!(
        text_scale(0.7, 200.0, 100.0),
        0.7,
        "a step never grows the text"
    );
    assert_eq!(text_scale(1.0, f32::NAN, 100.0), 1.0);
    let mut calls = 0;
    let linear = fit_text(100.0, |s| {
        calls += 1;
        120.0 * s
    });
    assert_eq!(
        (linear, calls),
        (0.83, 2),
        "one step lands, the next measure confirms"
    );
    // Hinted widths run a little superlinear: steps continue until the word fits.
    let hinted = fit_text(100.0, |s| 120.0 * s + 5.0);
    assert_eq!(hinted, 0.79);
    assert!(120.0 * hinted + 5.0 <= 100.0);
    assert_eq!(fit_text(100.0, |s| 200.0 * s), 0.75, "too long: the floor");
    calls = 0;
    fit_text(100.0, |s| {
        calls += 1;
        1000.0 * s.sqrt()
    });
    assert!(calls <= TEXT_FIT_STEPS);
}

const SIZE: f32 = 40.0;
fn roboto() -> Vec<u8> {
    include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()
}
/// Ink bounds (left, top, right, bottom) of one suggestion drawn alone across `width`.
fn ink(word: &str, size: f32, width: u16) -> [usize; 4] {
    let c = decode(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 60.0]);
    let mut bar = Smartbar::new(c, Vec::new());
    bar.intrinsic = vec![0.0]; // one classic slot: the whole row, whatever the word's width
    bar.relayout(f32::from(width));
    let text = Style {
        size,
        foreground: 0xff000000,
        ..Style::default()
    };
    let scene = Scene {
        width,
        height: 60,
        strip: 60.0,
        background: 0xffffffff,
        smartbar: Some(bar),
        candidates: vec![word.into()],
        candidate_styles: vec![std::array::from_fn(|_| CandidateStyle {
            container: Style::default(),
            text: text.clone(),
        })],
        ..Scene::default()
    };
    let mut raster = Raster::new(width, 60, vec![roboto()]);
    raster
        .draw(&scene, Scene::default().damage(&scene).unwrap())
        .unwrap();
    let mut b = [usize::MAX, usize::MAX, 0, 0];
    for (i, px) in raster.pixels().chunks(4).enumerate() {
        if px[0] < 128 {
            let (x, y) = (i % usize::from(width), i / usize::from(width));
            b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
        }
    }
    assert!(b[0] <= b[2], "{word} drew nothing");
    b
}

#[test]
fn a_long_word_shrinks_to_fit_its_slot_and_ellipsises_only_past_the_floor() {
    let mut text = Text::new([roboto()]);
    let word = "Kangarookeeper";
    let full = text.measure(word, SIZE, false, 400);
    let wide = ink(word, SIZE, (full * 2.0) as u16);
    let height = |b: [usize; 4]| (b[3] - b[1]) as f32;
    // Same glyphs at the same size: equal ink, give or take a pixel of subpixel placement.
    let same = |a: [usize; 4], b: [usize; 4]| {
        (a[2] - a[0]).abs_diff(b[2] - b[0]) <= 1 && height(a) == height(b)
    };
    // Needs about nine tenths: drawn whole at the fitted size, centred, no ellipsis.
    let slot = (full * 0.9).floor();
    let scale = fit_text(slot, |s| text.measure(word, SIZE * s, false, 400));
    assert!(scale > MIN_TEXT_SCALE && scale < 0.91, "{scale}");
    let shrunk = ink(word, SIZE, slot as u16);
    let expected = ink(word, SIZE * scale, (full * 2.0) as u16);
    assert!(
        same(shrunk, expected),
        "the whole word at {scale}: {shrunk:?} {expected:?}"
    );
    assert!(height(shrunk) < height(wide));
    let gaps = (shrunk[0] as f32, slot - 1.0 - shrunk[2] as f32);
    assert!((gaps.0 - gaps.1).abs() <= 3.0, "centred: {gaps:?}");
    // Needs half: the floor, then the ellipsis, at three quarters size.
    let slot = (full * 0.5).floor();
    assert_eq!(
        fit_text(slot, |s| text.measure(word, SIZE * s, false, 400)),
        MIN_TEXT_SCALE
    );
    let floor = SIZE * MIN_TEXT_SCALE;
    let ends: Vec<_> = word.char_indices().map(|(i, c)| i + c.len_utf8()).collect();
    let fit = smartbar::fit_prefix(ends.len(), slot, |i| {
        text.measure(&format!("{}…", &word[..ends[i]]), floor, false, 400)
    });
    let label = format!("{}…", &word[..ends[fit - 1]]);
    let unshrunk = smartbar::fit_prefix(ends.len(), slot, |i| {
        text.measure(&format!("{}…", &word[..ends[i]]), SIZE, false, 400)
    });
    assert!(fit > unshrunk, "shrinking first shows more of the word");
    let clipped = ink(word, SIZE, slot as u16);
    let expected = ink(&label, floor, (full * 2.0) as u16);
    assert!(
        same(clipped, expected),
        "{label} at the floor: {clipped:?} {expected:?}"
    );
    assert!(
        clipped[0] <= 3,
        "an ellipsised word draws from the slot's start"
    );
}
