//! What the press-bubble tests share: a keyboard of letter keys, a styled bubble on one of them,
//! and drawing it. press_bubble.rs and press_bubble_frames.rs each use all of it.
use keys::{Bounds, Rect};
use render::{
    popup_scene::headroom,
    press_bubble::{Bubble, mark},
    raster::Raster,
    scene::{Key, Scene},
    style::Style,
};

pub const WIDTH: u16 = 400;
const ROW: f32 = 50.0;

/// Three rows of `columns` keys across a scene `width` wide under a strip `strip` tall, each
/// key's visible box `gap` in from its touch box, every key a letter that shows a bubble.
pub fn keyboard(width: u16, columns: usize, gap: f32, strip: f32) -> Scene {
    let step = f32::from(width) / columns as f32;
    let keys = (0..3 * columns)
        .map(|i| {
            let (row, column) = ((i / columns) as f32, (i % columns) as f32);
            let touch = Rect {
                left: column * step,
                right: (column + 1.0) * step,
                top: row * ROW,
                bottom: (row + 1.0) * ROW,
            };
            let visible = Rect {
                left: touch.left + gap,
                right: touch.right - gap,
                top: touch.top + gap,
                bottom: touch.bottom - gap,
            };
            Key {
                code: 97 + (i % 26) as i32,
                bounds: Bounds { touch, visible },
                label: char::from(b'a' + (i % 26) as u8).to_string(),
                hint: String::new(),
                icon: String::new(),
                styles: Default::default(),
                hints: Default::default(),
                enabled: true,
            }
        })
        .collect();
    let mut scene = Scene {
        width,
        height: (strip + 3.0 * ROW) as u16,
        strip,
        keyboard_height: 3.0 * ROW,
        desired_key: [step - 2.0 * gap, ROW - 2.0 * gap],
        keys,
        background: 0xffffffff,
        ..Scene::default()
    };
    mark(
        &mut scene,
        "characters",
        &vec!["character".into(); 3 * columns],
    );
    scene
}

/// `scene` laid out for `landscape`, with the room above the keys that takes.
pub fn oriented(mut scene: Scene, landscape: bool) -> Scene {
    scene.landscape = landscape;
    scene.headroom = headroom(&scene, landscape);
    scene
}

pub fn style() -> Style {
    Style {
        size: 20.0,
        background: 0xff445566,
        foreground: 0xffffffff,
        shadow: 2.0,
        corners: [6.0; 4],
        ..Style::default()
    }
}

/// `scene` with key `key`'s bubble open and styled, its dots shown for `choices`.
pub fn pressed(scene: &Scene, key: usize, choices: bool) -> Scene {
    let indicator = Style {
        size: 12.0,
        ..style()
    };
    Scene {
        pressed: vec![key],
        bubble: Some(Bubble {
            key,
            style: Some(style()),
            indicator: choices.then_some(indicator),
        }),
        ..scene.clone()
    }
}

fn font() -> Vec<u8> {
    include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()
}

pub fn draw(scene: &Scene) -> Raster {
    let mut raster = Raster::new(scene.width, scene.buffer_height(), vec![font()]);
    raster.draw(scene, scene.buffer_rect()).expect("raster");
    raster
}

/// The RGBA pixel at (`x`, `y`) of the scene, which sits `headroom` rows down the buffer.
pub fn pixel(raster: &Raster, scene: &Scene, x: f32, y: f32) -> [u8; 4] {
    let y = (y + f32::from(scene.headroom)) as usize;
    let at = (y * usize::from(scene.width) + x as usize) * 4;
    raster.pixels()[at..at + 4].try_into().expect("pixel")
}
