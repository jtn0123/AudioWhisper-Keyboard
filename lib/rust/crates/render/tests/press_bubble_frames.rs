//! How `Raster::draw_frame` draws the frames that open and close a press bubble: it paints
//! everything the bubble touches and nothing past it, puts back the pixels kept under it, and
//! paints the bubble alone only where its box hides everything under it. Each frame ends as a
//! whole draw paints it.
mod press_bubble_common;
use press_bubble_common::*;
use render::{scene::Scene, style::Style};

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
    let mut scene = keyboard(WIDTH, 10, 0.0, 0.0);
    let tone = |background| Style {
        background,
        ..Style::default()
    };
    for key in &mut scene.keys {
        key.styles = [tone(0xff808080), tone(0xff202020)];
    }
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
    let scene = keyboard(WIDTH, 10, 0.0, 0.0);
    let held = pressed(&scene, 12, false);
    let r = held.bubble_damage().expect("view");
    let mut raster = draw(&scene);
    raster.draw_frame(&scene, &held, &[r, r]).expect("raster");
    assert!(raster.pixels() == draw(&held).pixels());
}

/// A bubble's damage holds every pixel it paints, the faintest edge of a deep shadow included.
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
        let (width, room) = (usize::from(shown.width), f32::from(shown.headroom));
        let pixels = a
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .zip(b.pixels().as_chunks::<4>().0);
        for (i, (with, without)) in pixels.enumerate() {
            let (x, y) = ((i % width) as f32, (i / width) as f32 - room);
            let inside = x + 1.0 > damage.left && x < damage.right;
            let inside = inside && y + 1.0 > damage.top && y < damage.bottom;
            assert!(
                inside || with == without,
                "key {key}: ({x}, {y}) outside {damage:?}"
            );
        }
    }
}

/// What `Raster::draw_frame` paints with the bubble alone: whole tiles of the buffer, inside the
/// box's rounded corners, where nothing under an opaque box shows. A see-through box has none.
#[test]
fn a_bubbles_solid_part_is_whole_tiles_that_hide_everything_under_it() {
    for landscape in [false, true] {
        let scene = oriented(keyboard(WIDTH, 10, 2.0, 30.0), landscape);
        for key in 0..scene.keys.len() {
            let shown = pressed(&scene, key, false);
            let (_, _, r) = shown.bubble_view().expect("view");
            let solid = shown.bubble_cover().expect("an opaque box");
            let room = f32::from(shown.headroom);
            let edges = [
                solid.left,
                solid.top + room,
                solid.right,
                solid.bottom + room,
            ];
            assert!(edges.iter().all(|v| v % 4.0 == 0.0), "key {key}: {solid:?}");
            let corner = style().corners[0];
            assert!(
                solid.left > r.left && solid.right < r.right,
                "key {key}: {solid:?}"
            );
            assert!(
                solid.top > r.top + corner && solid.bottom < r.bottom - corner,
                "key {key}"
            );
            let other = Scene {
                background: 0xff000000,
                ..shown.clone()
            };
            let (a, b) = (draw(&shown), draw(&other));
            for y in solid.top as i32..solid.bottom as i32 {
                for x in solid.left as i32..solid.right as i32 {
                    let (x, y) = (x as f32, y as f32);
                    assert_eq!(
                        pixel(&a, &shown, x, y),
                        pixel(&b, &other, x, y),
                        "key {key}"
                    );
                }
            }
        }
    }
    let mut glass = pressed(&keyboard(WIDTH, 10, 2.0, 30.0), 0, false);
    let see_through = Style {
        background: 0x80445566,
        ..style()
    };
    glass.bubble.as_mut().expect("bubble").style = Some(see_through);
    assert_eq!(
        glass.bubble_cover(),
        None,
        "a see-through box hides nothing"
    );
}
