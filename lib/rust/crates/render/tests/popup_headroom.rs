//! A top-row key's popup reaches above the keyboard. The drawing buffer keeps that much room
//! above the scene (`popup_scene::headroom`) and the popup is drawn whole in it, as Compose drew
//! popups over the app. Before, the surface ended at the keyboard's top and cut the popup's upper
//! row in half. Nothing else draws in that room: it stays transparent over the app.
use keys::{Bounds, Rect};
use render::{
    panels::Panel,
    popup_scene::{configure, headroom},
    raster::Raster,
    scene::{Key, Popup, Scene},
    style::Style,
    trail::Circle,
};

const WIDTH: u16 = 400;
const ROW: f32 = 50.0;

/// Three rows of ten keys, as a phone keyboard's letters sit, under a strip `strip` tall.
fn keyboard(strip: f32) -> Scene {
    let keys = (0..30)
        .map(|i| {
            let (row, column) = ((i / 10) as f32, (i % 10) as f32);
            let touch = Rect {
                left: column * 40.0,
                right: (column + 1.0) * 40.0,
                top: row * ROW,
                bottom: (row + 1.0) * ROW,
            };
            let visible = Rect {
                left: touch.left + 2.0,
                right: touch.right - 2.0,
                top: touch.top + 2.0,
                bottom: touch.bottom - 2.0,
            };
            let style = |background| Style {
                size: 20.0,
                background,
                foreground: 0xff000000,
                shadow: 2.0,
                corners: [4.0; 4],
                ..Style::default()
            };
            Key {
                code: 97 + i,
                bounds: Bounds { touch, visible },
                label: char::from(b'a' + (i % 26) as u8).to_string(),
                hint: String::new(),
                icon: String::new(),
                styles: [style(0xfff0f0f0), style(0xffc0c0c0)],
                hints: std::array::from_fn(|_| Style::default()),
                enabled: true,
            }
        })
        .collect();
    Scene {
        width: WIDTH,
        height: (strip + 3.0 * ROW) as u16,
        strip,
        keyboard_height: 3.0 * ROW,
        desired_key: [36.0, 46.0],
        keys,
        background: 0xffffffff,
        ..Scene::default()
    }
}

/// `scene` with key `key`'s popup of `count` choices open, laid out as a long press lays it out.
fn open(scene: &Scene, key: usize, count: usize, prioritized: i32, landscape: bool) -> Scene {
    let choice = |background| Style {
        size: 18.0,
        background,
        foreground: 0xff202124,
        ..Style::default()
    };
    let mut open = Scene {
        popup_style: Style {
            background: 0xff445566,
            corners: [6.0; 4],
            ..Style::default()
        },
        popup_styles: (0..count)
            .map(|i| [choice(0xff102030 + i as u32 * 0x100), choice(0xff8899aa)])
            .collect(),
        pressed: vec![key],
        popup: Some(Popup {
            key,
            labels: (0..count).map(|i| i.to_string()).collect(),
            selected: 0,
        }),
        ..scene.clone()
    };
    configure(&mut open, prioritized, landscape).expect("popup grid");
    open
}

fn font() -> Vec<u8> {
    include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()
}

/// The RGBA pixel at (`x`, `y`) of the scene, which sits `headroom` rows down the buffer; None
/// outside the buffer.
fn pixel(raster: &Raster, scene: &Scene, x: f32, y: f32) -> Option<[u8; 4]> {
    let y = y + f32::from(scene.headroom);
    if x < 0.0 || y < 0.0 || x >= f32::from(scene.width) || y >= f32::from(scene.buffer_height()) {
        return None;
    }
    let at = (y as usize * usize::from(scene.width) + x as usize) * 4;
    raster.pixels()[at..at + 4].try_into().ok()
}

/// The ARGB colour `argb` as the raster's opaque RGBA bytes.
fn rgba(argb: u32) -> [u8; 4] {
    let [a, r, g, b] = argb.to_be_bytes();
    [r, g, b, a]
}

#[test]
fn every_popup_of_every_key_fits_above_the_keyboard() {
    for strip in [0.0, 30.0] {
        for landscape in [false, true] {
            let mut scene = keyboard(strip);
            scene.headroom = headroom(&scene, landscape);
            let room = scene.buffer_rect();
            let mut highest = 0.0f32;
            for key in 0..scene.keys.len() {
                for count in 1..=14 {
                    for prioritized in 0..=3.min(count as i32) {
                        let open = open(&scene, key, count, prioritized, landscape);
                        let bounds = open.popup_bounds().expect("popup");
                        highest = highest.min(bounds.top);
                        assert!(
                            bounds.top >= room.top,
                            "strip {strip}, landscape {landscape}, key {key}, {count} choices: \
                             popup top {} above the buffer's {}",
                            bounds.top,
                            room.top
                        );
                        for i in 0..count {
                            let choice = open.popup_choice_bounds(i).expect("choice");
                            assert!(choice.top >= room.top && choice.bottom <= bounds.bottom);
                        }
                    }
                }
            }
            // The room is what the highest popup needs, not more: a whole surface's worth of
            // transparent rows would cost every full repaint.
            assert!(highest < 0.0, "a top-row popup reaches above the keys");
            assert_eq!(scene.headroom, (-highest).ceil() as u16);
        }
    }
}

#[test]
fn landscape_popups_are_taller_and_a_strip_leaves_less_to_keep() {
    let portrait = headroom(&keyboard(0.0), false);
    let landscape = headroom(&keyboard(0.0), true);
    assert!(landscape > portrait, "{landscape} <= {portrait}");
    assert_eq!(headroom(&keyboard(30.0), false), portrait - 30);
    let mut sheet = keyboard(0.0);
    sheet.panel = Some(Panel {
        kind: 3,
        ..Panel::default()
    });
    assert_eq!(
        headroom(&sheet, false),
        0,
        "a full-screen panel shows no keys"
    );
    let mut no_size = keyboard(0.0);
    no_size.desired_key = [0.0, 0.0];
    assert_eq!(
        headroom(&no_size, false),
        0,
        "no popup before the keys have a size"
    );
}

#[test]
fn a_top_row_popup_is_drawn_whole_above_the_keys() {
    let mut closed = keyboard(0.0);
    closed.headroom = headroom(&closed, false);
    // Key "c" of the top row, with seven choices in two rows, as e's accents and 3 sit.
    let open = open(&closed, 2, 7, 1, false);
    let mut raster = Raster::new(WIDTH, open.buffer_height(), vec![font()]);
    raster.draw(&open, open.buffer_rect()).expect("raster");
    let bounds = open.popup_bounds().expect("popup");
    assert!(bounds.top < 0.0, "the upper row is above the keyboard");
    let popup = open.popup.as_ref().expect("popup");
    for (i, styles) in open.popup_styles.iter().enumerate() {
        let r = open.popup_choice_bounds(i).expect("choice");
        let style = &styles[usize::from(i == popup.selected)];
        // A corner of the choice's box, clear of its label.
        let shown = pixel(&raster, &open, r.left + 2.0, r.top + 2.0);
        assert_eq!(shown, Some(rgba(style.background)), "choice {i} at {r:?}");
    }
    // Beside the popup, the room above the keyboard stays clear over the app.
    assert_eq!(
        pixel(&raster, &open, bounds.right + 20.0, bounds.top + 4.0),
        Some([0; 4])
    );
    assert_eq!(pixel(&raster, &open, 1.0, -1.0), Some([0; 4]));
}

#[test]
fn the_keyboard_draws_the_same_pixels_lower_down() {
    let flush = keyboard(30.0);
    let mut kept = flush.clone();
    kept.headroom = headroom(&kept, true);
    assert!(kept.headroom > 0);
    let mut a = Raster::new(WIDTH, flush.buffer_height(), vec![font()]);
    let mut b = Raster::new(WIDTH, kept.buffer_height(), vec![font()]);
    a.draw(&flush, flush.buffer_rect()).expect("raster");
    b.draw(&kept, kept.buffer_rect()).expect("raster");
    let room = usize::from(kept.headroom) * usize::from(WIDTH) * 4;
    assert!(
        b.pixels()[..room].iter().all(|&v| v == 0),
        "only a popup draws up there"
    );
    assert!(
        b.pixels()[room..] == *a.pixels(),
        "the keyboard moved by whole rows, unchanged"
    );
}

#[test]
fn opening_and_closing_a_top_row_popup_repaints_the_room_above() {
    let mut closed = keyboard(0.0);
    closed.headroom = headroom(&closed, false);
    // A glide trail and key shadows at the keyboard's top edge stay on the keyboard.
    closed.trail = vec![Circle {
        x: 200.0,
        y: 2.0,
        radius: 12.0,
    }];
    closed.trail_color = 0xff0000ff;
    let frames = [
        closed.clone(),
        open(&closed, 0, 6, 0, false),
        closed.clone(),
        open(&closed, 9, 3, 3, false),
        closed.clone(),
    ];
    let (height, full) = (closed.buffer_height(), closed.buffer_rect());
    let mut raster = Raster::new(WIDTH, height, vec![font()]);
    let mut reference = Raster::new(WIDTH, height, vec![font()]);
    raster.draw(&frames[0], full).expect("raster");
    let room = usize::from(closed.headroom) * usize::from(WIDTH) * 4;
    assert!(raster.pixels()[..room].iter().all(|&v| v == 0));
    for pair in frames.windows(2) {
        let regions = pair[0].damage_regions(&pair[1]);
        assert!(
            !regions.contains(&full),
            "a popup repaints itself, not everything"
        );
        raster.draw_regions(&pair[1], &regions).expect("raster");
        reference.draw(&pair[1], full).expect("raster");
        assert!(
            raster.pixels() == reference.pixels(),
            "dirty differs from full"
        );
        let clear = raster.pixels()[..room].iter().all(|&v| v == 0);
        assert_eq!(
            clear,
            pair[1].popup.is_none(),
            "a closed popup leaves nothing behind"
        );
    }
}

#[test]
fn a_whole_repaint_and_the_buffer_damage_include_the_room_above() {
    let mut scene = keyboard(0.0);
    scene.headroom = 120;
    assert_eq!(scene.buffer_height(), 270);
    let full = Rect {
        left: 0.0,
        top: -120.0,
        right: 400.0,
        bottom: 150.0,
    };
    assert_eq!(Scene::default().damage_regions(&scene), vec![full]);
    let mut moved = scene.clone();
    moved.headroom = 100;
    assert_eq!(scene.damage_regions(&moved)[0].top, -100.0);
    // A popup's damage in scene coordinates, as the buffer's whole pixels.
    let popup = Rect {
        left: -3.5,
        top: -90.5,
        right: 60.2,
        bottom: 10.1,
    };
    let pixels = Rect {
        left: 0.0,
        top: 29.0,
        right: 61.0,
        bottom: 131.0,
    };
    assert_eq!(scene.buffer_pixels(popup), pixels);
    let outside = Rect {
        top: -300.0,
        bottom: -200.0,
        ..popup
    };
    let empty = scene.buffer_pixels(outside);
    assert!(empty.top == 0.0 && empty.bottom == 0.0, "{empty:?}");
}
