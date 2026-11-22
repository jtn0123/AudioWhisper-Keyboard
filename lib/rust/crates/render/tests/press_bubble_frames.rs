//! How `Raster::draw_frame` draws the frames that open, hold and close a press bubble: it paints
//! what changed, lays the bubble's image over the keys, keeps the pixels under it fresh while it
//! is held and puts them back when it goes. Each frame ends as a whole draw paints it.
mod press_bubble_common;
use press_bubble_common::*;
use render::{scene::Scene, style::Style};

/// The keyboard with grey keys that go dark when pressed.
fn toned() -> Scene {
    let mut scene = keyboard(WIDTH, 10, 0.0, 0.0);
    let tone = |background| Style {
        background,
        ..Style::default()
    };
    for key in &mut scene.keys {
        key.styles = [tone(0xff808080), tone(0xff202020)];
    }
    scene
}

/// Draws each frame's damage onto the last, as the window does, against a whole draw.
fn frames_match(frames: &[Scene]) {
    let mut raster = draw(&frames[0]);
    for (i, pair) in frames.windows(2).enumerate() {
        let regions = pair[0].damage_regions(&pair[1]);
        raster
            .draw_frame(&pair[0], &pair[1], &regions)
            .expect("raster");
        assert!(
            raster.pixels() == draw(&pair[1]).pixels(),
            "frame {}",
            i + 1
        );
    }
}

/// The pixels kept under a bubble are put back when it goes, and what changed under it since,
/// here a second finger on the key beside it, is painted again.
#[test]
fn what_changes_under_a_held_bubble_shows_when_it_goes() {
    let scene = toned();
    let held = pressed(&scene, 12, false);
    let both = Scene {
        pressed: vec![12, 11],
        ..held.clone()
    };
    let left = Scene {
        pressed: vec![11],
        bubble: None,
        ..held.clone()
    };
    frames_match(&[scene, held, both, left]);
}

/// A region given twice is painted once, not left out as lying inside the other.
#[test]
fn a_region_given_twice_is_still_painted() {
    let scene = toned();
    let held = Scene {
        pressed: vec![12, 29],
        ..pressed(&scene, 12, false)
    };
    let regions = scene.damage_regions(&held);
    let far = regions[1];
    let mut raster = draw(&scene);
    let twice = [regions[0], far, far, regions[2]];
    raster.draw_frame(&scene, &held, &twice).expect("raster");
    assert!(raster.pixels() == draw(&held).pixels());
}

/// The strip painted again under a held top-row bubble: the bubble is laid over the fresh pixels
/// there and nothing is put back or drawn anew. When it goes, the new strip shows under it.
#[test]
fn a_strip_painted_under_a_held_bubble_shows_through_it_and_after_it() {
    let scene = Scene {
        candidates: vec!["one".into(), "two".into()],
        ..oriented(keyboard(WIDTH, 10, 2.0, 30.0), false)
    };
    let held = pressed(&scene, 2, false);
    let words = Scene {
        candidates: vec!["three".into(), "four".into()],
        ..held.clone()
    };
    let gone = Scene {
        pressed: vec![],
        bubble: None,
        ..words.clone()
    };
    let mut raster = draw(&scene);
    raster.set_profiling(true);
    for (i, pair) in [&scene, &held, &words, &gone].windows(2).enumerate() {
        let regions = pair[0].damage_regions(pair[1]);
        raster
            .draw_frame(pair[0], pair[1], &regions)
            .expect("raster");
        assert!(raster.pixels() == draw(pair[1]).pixels(), "frame {i}");
        if i == 1 {
            let p = raster.profile();
            let bubble = held.buffer_pixels(held.bubble_damage().expect("view"));
            let under =
                (bubble.bottom.min(30.0 + f32::from(held.headroom)) - bubble.top) * bubble.width();
            assert_eq!(regions.len(), 1, "the strip alone");
            assert!(p.bubble_px > 0 && p.bubble_px as f32 <= under);
            assert_eq!((p.restored_px, p.bubble_draws), (0, 0));
        }
    }
}

/// A bubble a whole draw drew, with no pixels kept under it, goes as cleanly.
#[test]
fn a_bubble_drawn_whole_goes_with_its_frame() {
    let scene = toned();
    frames_match(&[pressed(&scene, 12, false), scene]);
}

/// A bubble's damage holds every pixel it paints, and a deep shadow fades out inside it: along the
/// damage's own edges, where the buffer does not cut it, the bubble shows by a level at most.
#[test]
fn a_bubble_paints_nothing_outside_its_damage() {
    let scene = oriented(keyboard(WIDTH, 10, 2.0, 30.0), false);
    for key in [0, 4, 9, 14] {
        let mut shown = pressed(&scene, key, false);
        let deep = Style {
            shadow: 6.0,
            ..style()
        };
        shown.bubble.as_mut().expect("bubble").style = Some(deep);
        let hidden = Scene {
            bubble: None,
            ..shown.clone()
        };
        let (a, b) = (draw(&shown), draw(&hidden));
        let damage = shown.bubble_damage().expect("view");
        let room = i32::from(shown.headroom);
        for y in -room..i32::from(shown.height) {
            for x in 0..i32::from(shown.width) {
                let (x, y) = (x as f32, y as f32);
                let inside = x + 1.0 > damage.left && x < damage.right;
                let inside = inside && y + 1.0 > damage.top && y < damage.bottom;
                let (with, without) = (pixel(&a, &shown, x, y), pixel(&b, &hidden, x, y));
                assert!(inside || with == without, "key {key}: ({x}, {y}) outside");
                let ring = x == damage.left.floor() || x + 1.0 == damage.right.ceil();
                let ring = ring || y == damage.top.floor() || y + 1.0 == damage.bottom.ceil();
                let ring = inside && ring;
                let faint = with.iter().zip(without).all(|(&p, q)| p.abs_diff(q) <= 1);
                assert!(!ring || faint, "key {key}: ({x}, {y}) on the edge");
            }
        }
    }
}
