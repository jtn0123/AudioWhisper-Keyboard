//! The press bubble's kept images (`Raster::draw_frame`): each is found again only by all that
//! changes its pixels, a shifted label draws its own while shift keeps the others, and all go
//! when the layout or the theme changes. Every frame still ends as a whole draw paints it.
mod press_bubble_common;
use press_bubble_common::*;
use render::{profile::Profile, raster::Raster, scene::Scene, style::Style};

/// Draws each frame's damage onto the last, as the window does, checks each against a whole
/// draw, and returns the raster and each frame's profile.
fn frames(frames: &[Scene]) -> (Raster, Vec<Profile>) {
    let mut raster = draw(&frames[0]);
    raster.set_profiling(true);
    let mut profiles = Vec::new();
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
        profiles.push(raster.profile());
    }
    (raster, profiles)
}

fn draws(profiles: &[Profile]) -> Vec<u32> {
    profiles.iter().map(|p| p.bubble_draws).collect()
}

fn scene() -> Scene {
    oriented(keyboard(WIDTH, 10, 2.0, 30.0), false)
}

#[test]
fn a_shifted_label_draws_its_own_image_and_shift_keeps_the_others() {
    let small = scene();
    let mut capital = small.clone();
    for key in &mut capital.keys {
        key.label = key.label.to_uppercase();
    }
    let (raster, profiles) = frames(&[
        small.clone(),
        pressed(&small, 12, false),
        small.clone(),
        capital.clone(),
        pressed(&capital, 12, false),
        capital,
        small.clone(),
        pressed(&small, 12, false),
    ]);
    assert_eq!(draws(&profiles), [1, 0, 0, 1, 0, 0, 0]);
    assert_eq!(raster.kept_bubbles(), 2);
}

/// A theme changing under a held bubble paints everything once, puts nothing back and draws the
/// bubble anew.
#[test]
fn a_theme_change_drops_the_images_and_paints_everything_once() {
    let light = scene();
    let dark = Scene {
        background: 0xff202020,
        ..light.clone()
    };
    let held = pressed(&dark, 12, false);
    let (raster, profiles) = frames(&[
        light.clone(),
        pressed(&light, 12, false),
        held.clone(),
        dark,
    ]);
    assert_eq!(draws(&profiles), [1, 1, 0]);
    assert_eq!(raster.kept_bubbles(), 1);
    let mut whole = draw(&held);
    whole.set_profiling(true);
    whole.draw(&held, held.buffer_rect()).expect("raster");
    assert_eq!(profiles[1].labels, whole.profile().labels, "painted once");
    assert_eq!(profiles[1].restored_px, 0, "nothing put back");
}

/// Another layout (a key narrower, a key more) or a new size drops the kept images.
#[test]
fn a_new_layout_or_size_drops_the_images() {
    let first = scene();
    let mut narrower = first.clone();
    narrower.keys[29].bounds.visible.right -= 10.0;
    let mut more = narrower.clone();
    more.keys.push(more.keys[29].clone());
    more.bubbles.push(true);
    let resized = Scene {
        headroom: first.headroom,
        ..oriented(keyboard(WIDTH, 10, 3.0, 30.0), false)
    };
    let mut list = Vec::new();
    for layout in [&first, &narrower, &more, &resized] {
        list.extend([layout.clone(), pressed(layout, 12, false)]);
    }
    let (raster, profiles) = frames(&list);
    assert_eq!(draws(&profiles), [1, 0, 1, 0, 1, 0, 1]);
    assert_eq!(raster.kept_bubbles(), 1);
}

/// A raster that drew one bubble draws another that differs in any one thing its pixels depend
/// on as a fresh raster does.
#[test]
fn every_part_of_a_bubbles_look_finds_its_own_image() {
    let base = pressed(&scene(), 12, false);
    let restyled = Style {
        background: 0xff884422,
        ..style()
    };
    let mut variants = vec![("dots", pressed(&scene(), 12, true))];
    let mut v = base.clone();
    v.bubble.as_mut().expect("bubble").style = Some(restyled);
    variants.push(("style", v));
    let mut v = base.clone();
    v.keys[12].label = "Q".into();
    variants.push(("label", v));
    let mut v = base.clone();
    v.keys[12].bounds.visible.top += 6.0;
    variants.push(("key height", v));
    variants.push((
        "bounds",
        Scene {
            strip: base.strip + 0.25,
            ..base.clone()
        },
    ));
    let at = |s: &Scene| s.buffer_pixels(s.bubble_damage().expect("view"));
    for (part, variant) in variants {
        let same_place = variant.bubble_bounds(12) == base.bubble_bounds(12);
        assert_eq!(same_place, part != "bounds", "{part}");
        assert_eq!(at(&variant), at(&base), "{part}: the same pixels");
        let mut raster = draw(&base);
        raster
            .draw(&variant, variant.buffer_rect())
            .expect("raster");
        assert!(raster.pixels() == draw(&variant).pixels(), "{part}");
    }
}

/// The bubbles shown last are kept, 32 of them; one used again is kept longest.
#[test]
fn images_are_kept_for_the_last_thirty_two_bubbles() {
    let scene = oriented(keyboard(WIDTH, 11, 2.0, 30.0), false);
    let mut list = vec![scene.clone()];
    for key in (0..33).chain([1, 0, 1]) {
        list.extend([pressed(&scene, key, false), scene.clone()]);
    }
    let (raster, profiles) = frames(&list);
    let draws: Vec<u32> = draws(&profiles).into_iter().step_by(2).collect();
    assert_eq!(draws[..33], [1; 33]);
    assert_eq!(draws[33..], [0, 1, 0], "key 1 used again, key 0 drawn anew");
    assert_eq!(raster.kept_bubbles(), 32);
}

/// The image is laid over the keys, not copied onto them: its shadow shows what lies under it.
#[test]
fn the_shadow_shows_what_lies_under_it() {
    let light = pressed(&scene(), 12, false);
    let dark = Scene {
        background: 0xff000000,
        ..light.clone()
    };
    let (_, _, r) = light.bubble_view().expect("view");
    let (x, y) = ((r.left + r.right) / 2.0, r.bottom + 1.0);
    let (over_light, over_dark) = (
        pixel(&draw(&light), &light, x, y),
        pixel(&draw(&dark), &dark, x, y),
    );
    assert_ne!(over_light, [255; 4], "the shadow paints there");
    assert_ne!(over_light, over_dark);
}
