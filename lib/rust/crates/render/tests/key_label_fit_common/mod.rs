//! What the key-label fit tests share: a row of three keys with a label in the middle one, the
//! fonts (Roboto, and Noto subsets for Hebrew and CJK), and where a frame put ink.
use keys::{Bounds, Rect};
use render::{
    label_fit::{Align, Overflow, TextRules},
    raster::Raster,
    scene::{Key, Scene},
    style::Style,
};

pub const RED: u32 = 0xffff0000;
pub const LONG: &str = "English (United States)";
const GAP: f32 = 10.0;
const TOP: f32 = 110.0;
pub const ROW: f32 = 100.0;

pub fn fonts() -> Vec<Vec<u8>> {
    vec![
        include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
        include_bytes!("../fixtures/noto-sans-hebrew-subset.ttf").to_vec(),
        include_bytes!("../fixtures/noto-sans-cjk-subset.otf").to_vec(),
    ]
}

/// Red text `size` px, under `rules`.
pub fn style(size: f32, rules: TextRules) -> Style {
    Style {
        size,
        foreground: RED,
        rules,
        ..Style::default()
    }
}

pub fn rules(max_lines: Option<i32>, overflow: Overflow) -> TextRules {
    TextRules {
        max_lines,
        overflow,
        align: Align::Start,
    }
}

/// Every pairing of `text-max-lines` and `text-overflow` a theme can set, and neither.
pub fn every_rule() -> Vec<TextRules> {
    let mut out = Vec::new();
    for max_lines in [None, Some(1), Some(2), Some(i32::MAX)] {
        for overflow in [
            Overflow::Unset,
            Overflow::Clip,
            Overflow::Ellipsis,
            Overflow::Visible,
        ] {
            out.push(rules(max_lines, overflow));
        }
    }
    out
}

/// A row of three keys `width` wide on white with room above it for a popup; the middle one
/// shows `label` in `style`, its neighbours nothing.
pub fn row(label: &str, style: &Style, width: f32) -> Scene {
    let key = |i: usize, label: &str, style: Style| {
        let left = GAP + i as f32 * (width + GAP);
        let visible = Rect {
            left,
            top: TOP,
            right: left + width,
            bottom: TOP + ROW,
        };
        let touch = Rect {
            left: visible.left - GAP / 2.0,
            right: visible.right + GAP / 2.0,
            top: visible.top - GAP / 2.0,
            bottom: visible.bottom + GAP / 2.0,
        };
        Key {
            code: 97 + i as i32,
            bounds: Bounds { touch, visible },
            label: label.into(),
            hint: String::new(),
            icon: String::new(),
            styles: [style.clone(), style.clone()],
            hints: [style.clone(), style],
            enabled: true,
        }
    };
    let plain = Style {
        foreground: RED,
        ..Style::default()
    };
    Scene {
        width: (3.0 * (width + GAP) + GAP) as u16,
        height: (TOP + ROW + GAP) as u16,
        keyboard_height: ROW + 2.0 * GAP,
        keys: vec![
            key(0, "", plain.clone()),
            key(1, label, style.clone()),
            key(2, "", plain),
        ],
        background: 0xffffffff,
        ..Scene::default()
    }
}

/// The box key `i` of `scene` draws in.
pub fn key_box(scene: &Scene, i: usize) -> Rect {
    scene.keys[i].bounds.visible
}

pub fn draw(scene: &Scene) -> Raster {
    let mut raster = Raster::new(scene.width, scene.buffer_height(), fonts());
    raster.draw(scene, scene.buffer_rect()).expect("raster");
    raster
}

/// The scene's pixels, each with where it is in the scene, which sits `headroom` rows down.
fn pixels<'a>(raster: &'a Raster, scene: &Scene) -> impl Iterator<Item = (f32, f32, [u8; 4])> + 'a {
    let (width, room) = (usize::from(scene.width), f32::from(scene.headroom));
    raster
        .pixels()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .map(move |(i, &p)| {
            let (x, y) = ((i % width) as f32, (i / width) as f32 - room);
            (x, y, p)
        })
}

/// Whether a pixel shows red text: on white, or on any grey.
pub fn red(p: [u8; 4]) -> bool {
    i32::from(p[0]) - i32::from(p[1]) > 8
}

/// How many red pixels fall inside `r`, and how many outside it.
pub fn red_in_and_out(raster: &Raster, scene: &Scene, r: Rect) -> (usize, usize) {
    let inside = |x: f32, y: f32| x >= r.left && x < r.right && y >= r.top && y < r.bottom;
    pixels(raster, scene)
        .filter(|&(_, _, p)| red(p))
        .fold(
            (0, 0),
            |(i, o), (x, y, _)| {
                if inside(x, y) { (i + 1, o) } else { (i, o + 1) }
            },
        )
}

/// The red pixels' bounds, if any are red.
pub fn red_bounds(raster: &Raster, scene: &Scene) -> Option<Rect> {
    pixels(raster, scene)
        .filter(|&(_, _, p)| red(p))
        .fold(None, |b: Option<Rect>, (x, y, _)| {
            let b = b.unwrap_or(Rect {
                left: x,
                top: y,
                right: x + 1.0,
                bottom: y + 1.0,
            });
            Some(Rect {
                left: b.left.min(x),
                top: b.top.min(y),
                right: b.right.max(x + 1.0),
                bottom: b.bottom.max(y + 1.0),
            })
        })
}
