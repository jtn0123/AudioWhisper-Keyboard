//! A long word fits its slot Gboard's way: it squeezes (narrower glyphs, same height) down to
//! AOSP LatinIME's 0.70, then its font shrinks down to three quarters, then it ellipsises.
use render::{
    raster::Raster,
    scene::{CandidateStyle, Scene},
    smartbar::{self, Config},
    smartbar_fit::{
        MIN_TEXT_SCALE, MIN_TEXT_SQUEEZE, TEXT_FIT_STEPS, TextFit, fit_step, fit_text, text_scale,
    },
    smartbar_scene::Smartbar,
    style::Style,
    text::Text,
};

fn fit(squeeze: f32, scale: f32) -> TextFit {
    TextFit { squeeze, scale }
}

#[test]
fn a_word_squeezes_in_whole_percents_down_to_aosps_floor_before_it_shrinks() {
    assert_eq!(MIN_TEXT_SQUEEZE, 0.70);
    assert_eq!(MIN_TEXT_SCALE, 0.75);
    let natural = TextFit::NATURAL;
    assert_eq!(natural, fit(1.0, 1.0));
    assert_eq!(fit_step(natural, 100.0, 100.0), natural, "fits: untouched");
    assert_eq!(fit_step(natural, 125.0, 100.0), fit(0.8, 1.0));
    assert_eq!(
        fit_step(natural, 120.0, 100.0),
        fit(0.83, 1.0),
        "rounded down"
    );
    assert_eq!(
        fit_step(natural, 142.0, 100.0),
        fit(0.7, 1.0),
        "the floor, and it fits"
    );
    // Past the squeeze floor, and only there, the font shrinks, still squeezed to the floor.
    assert_eq!(fit_step(natural, 150.0, 100.0), fit(0.7, 0.95));
    assert_eq!(
        fit_step(natural, 400.0, 100.0),
        fit(0.7, 0.75),
        "both floors"
    );
    assert_eq!(fit_step(fit(0.7, 0.75), 300.0, 100.0), fit(0.7, 0.75));
    // A step never widens or grows a word, nor reacts to a broken measure.
    assert_eq!(fit_step(fit(0.8, 1.0), 110.0, 100.0), fit(0.8, 1.0));
    assert_eq!(fit_step(fit(0.75, 1.0), 200.0, 100.0), fit(0.7, 0.75));
    assert_eq!(fit_step(fit(0.7, 0.8), 140.0, 100.0), fit(0.7, 0.8));
    assert_eq!(fit_step(natural, f32::NAN, 100.0), natural);
    assert_eq!(fit_step(natural, 120.0, f32::INFINITY), natural);
    // The font's own step is unchanged: whole percents down to three quarters.
    assert_eq!(text_scale(1.0, 120.0, 100.0), 0.83);
    assert_eq!(text_scale(1.0, 400.0, 100.0), 0.75);
    assert_eq!(
        text_scale(0.7, 200.0, 100.0),
        0.7,
        "a step never grows the text"
    );
}

#[test]
fn squeeze_comes_before_shrink_and_both_before_the_ellipsis() {
    for width in 90..=400 {
        let measured = width as f32;
        let mut calls = 0;
        let got = fit_text(100.0, |s| {
            calls += 1;
            measured * s
        });
        let case = format!("{measured} wide in 100");
        if measured <= 100.0 {
            assert_eq!((got, calls), (TextFit::NATURAL, 1), "{case}");
        } else if measured <= 142.0 {
            // A squeeze is exact: one measure, the font untouched.
            assert_eq!((got.scale, calls), (1.0, 1), "{case}");
            assert!(
                got.squeeze < 1.0 && got.squeeze >= MIN_TEXT_SQUEEZE,
                "{case}"
            );
            assert!(measured * got.squeeze <= 100.0, "{case}");
            assert!(
                measured * (got.squeeze + 0.01) > 100.0,
                "the widest that fits: {case}"
            );
        } else if measured * MIN_TEXT_SQUEEZE * MIN_TEXT_SCALE <= 100.0 {
            assert_eq!(
                got.squeeze, MIN_TEXT_SQUEEZE,
                "squeezed to the floor first: {case}"
            );
            assert!(got.scale < 1.0 && got.scale >= MIN_TEXT_SCALE, "{case}");
            assert!(measured * got.scale * got.squeeze <= 100.0, "{case}");
            assert!(calls <= TEXT_FIT_STEPS, "{case}");
        } else {
            assert_eq!(
                got,
                fit(MIN_TEXT_SQUEEZE, MIN_TEXT_SCALE),
                "ellipsises: {case}"
            );
        }
    }
    // Hinted widths run a little superlinear: the font steps on until the word fits.
    let hinted = fit_text(100.0, |s| 150.0 * s + 5.0);
    assert_eq!(hinted, fit(0.7, 0.91));
    assert!((150.0 * hinted.scale + 5.0) * hinted.squeeze <= 100.0);
    let mut calls = 0;
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
    let c = Config::decode(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 60.0]);
    let mut bar = Smartbar::new(c.unwrap(), Vec::new());
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
    let damage = Scene::default().damage(&scene).unwrap();
    raster.draw(&scene, damage).unwrap();
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
fn wide(b: [usize; 4]) -> f32 {
    (b[2] - b[0]) as f32
}
fn tall(b: [usize; 4]) -> usize {
    b[3] - b[1]
}

#[test]
fn the_renderer_squeezes_then_shrinks_then_ellipsises_a_long_word() {
    let mut text = Text::new([roboto()]);
    let word = "Kangarookeeper";
    let full = text.measure(word, SIZE, false, 400);
    let room = (full * 2.0) as u16;
    let natural = ink(word, SIZE, room);
    let mut measure = |s: f32| text.measure(word, SIZE * s, false, 400);
    // Needs 0.85: squeezed whole, at full height, centred.
    let slot = (full * 0.85).floor();
    let got = fit_text(slot, &mut measure);
    assert!(
        got.scale == 1.0 && got.squeeze < 0.86 && got.squeeze > 0.8,
        "{got:?}"
    );
    let squeezed = ink(word, SIZE, slot as u16);
    // The height is kept, give or take a pixel of antialiasing: not a uniform shrink.
    assert!(
        tall(squeezed).abs_diff(tall(natural)) <= 1,
        "{squeezed:?} {natural:?}"
    );
    let shrunk = ink(word, SIZE * got.squeeze, room);
    assert!(tall(shrunk) + 3 < tall(squeezed), "{shrunk:?} {squeezed:?}");
    let expected = wide(natural) * got.squeeze;
    assert!(
        (wide(squeezed) - expected).abs() <= 2.0,
        "{squeezed:?} {natural:?}"
    );
    let gaps = (squeezed[0] as f32, slot - 1.0 - squeezed[2] as f32);
    assert!((gaps.0 - gaps.1).abs() <= 3.0, "centred: {gaps:?}");
    // Needs 0.6: squeezed to the floor, then a smaller font, whole.
    let slot = (full * 0.6).floor();
    let got = fit_text(slot, &mut measure);
    assert_eq!(got.squeeze, MIN_TEXT_SQUEEZE);
    assert!(got.scale < 0.9 && got.scale > MIN_TEXT_SCALE, "{got:?}");
    let both = ink(word, SIZE, slot as u16);
    let smaller = ink(word, SIZE * got.scale, room);
    assert!(tall(both) < tall(natural));
    assert!(
        tall(both).abs_diff(tall(smaller)) <= 1,
        "{both:?} {smaller:?}"
    );
    let expected = wide(smaller) * MIN_TEXT_SQUEEZE;
    assert!((wide(both) - expected).abs() <= 2.0, "{both:?} {smaller:?}");
    // Needs 0.4: both floors, then the ellipsis, drawn squeezed from the slot's start.
    let slot = (full * 0.4).floor();
    let floors = fit(MIN_TEXT_SQUEEZE, MIN_TEXT_SCALE);
    assert_eq!(fit_text(slot, &mut measure), floors);
    let floor = SIZE * MIN_TEXT_SCALE;
    let ends: Vec<_> = word.char_indices().map(|(i, c)| i + c.len_utf8()).collect();
    let mut prefix = |squeeze: f32| {
        smartbar::fit_prefix(ends.len(), slot, |i| {
            text.measure(&format!("{}…", &word[..ends[i]]), floor, false, 400) * squeeze
        })
    };
    let (shown, unsqueezed) = (prefix(MIN_TEXT_SQUEEZE), prefix(1.0));
    assert!(shown > unsqueezed, "squeezing first shows more of the word");
    let label = format!("{}…", &word[..ends[shown - 1]]);
    let clipped = ink(word, SIZE, slot as u16);
    let plain = ink(&label, floor, room);
    assert!(
        tall(clipped).abs_diff(tall(plain)) <= 1,
        "{clipped:?} {plain:?}"
    );
    let expected = wide(plain) * MIN_TEXT_SQUEEZE;
    assert!(
        (wide(clipped) - expected).abs() <= 2.0,
        "{label}: {clipped:?} {plain:?}"
    );
    assert!(
        clipped[0] <= 3,
        "an ellipsised word draws from the slot's start"
    );
}
