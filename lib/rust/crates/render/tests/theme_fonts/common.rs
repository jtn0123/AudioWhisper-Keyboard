//! The fixture theme, its scene and the reports the theme-font tests compare.
use keys::{Bounds, Rect};
use parley::PositionedLayoutItem;
use render::{
    raster::Raster,
    scene::{Key, Scene},
    text::{Font, Text},
    theme::{Request, Theme},
};
use skrifa::{FontRef, MetadataProvider, string::StringId};
use std::path::PathBuf;

pub const STRIP: f32 = 60.0;
pub const SIZE: (u16, u16) = (320, 160);

/// A face of the fixture theme: its `src` URI and the bundled file it resolves to.
pub type File = (&'static str, &'static str);
pub const REGULAR: File = (
    "flex:/fonts/regular.ttf",
    "fixture-script-regular-subset.ttf",
);
pub const BOLD: File = ("flex:/fonts/bold.ttf", "fixture-script-bold-subset.ttf");

/// An `@font` rule with a regular and a bold face, which the root, and so every element, uses;
/// one with the bold file declared as the italic face, for the action tiles; one declaring each
/// file the other's weight, for the one-handed buttons; and a button that names a font the theme
/// does not have.
pub const STYLESHEET: &str = r##"{
  "$schema": "https://schemas.florisboard.org/snygg/v2/stylesheet",
  "@font `Fixture`": [
    {"src": "uri(`flex:/fonts/regular.ttf`)"},
    {"src": "uri(`flex:/fonts/bold.ttf`)", "font-weight": "bold"}
  ],
  "@font `Swapped`": [
    {"src": "uri(`flex:/fonts/regular.ttf`)", "font-weight": "bold"},
    {"src": "uri(`flex:/fonts/bold.ttf`)", "font-weight": "normal"}
  ],
  "@font `Slanted`": [
    {"src": "uri(`flex:/fonts/regular.ttf`)"},
    {"src": "uri(`flex:/fonts/bold.ttf`)", "font-style": "italic"}
  ],
  "root": {"font-family": "`Fixture`"},
  "window": {"background": "#ffffff", "foreground": "#000000"},
  "key": {"background": "#e0e0e0", "foreground": "#000000", "font-size": "24sp"},
  "smartbar-candidate-word-text": {"font-size": "20sp"},
  "clipboard-header-button": {"font-family": "`Missing`"},
  "smartbar-action-tile-text": {"font-family": "`Slanted`"},
  "one-handed-panel-button": {"font-family": "`Swapped`"}
}"##;

pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn roboto() -> Vec<u8> {
    include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()
}

/// What Kotlin passes for `faces`: each URI with the absolute path of its bundled file.
pub fn files(faces: &[File]) -> Vec<(String, String)> {
    faces
        .iter()
        .map(|(uri, name)| (uri.to_string(), fixture(name).display().to_string()))
        .collect()
}

pub fn theme(files: &[(String, String)]) -> Theme {
    let mut theme = Theme::new(STYLESHEET, vec![], vec![], 2.0, 1.0, 1.0).unwrap();
    theme.load_fonts(files);
    theme
}

/// The fixture scene, a key labelled "Hello" under one candidate, drawn in the fixture theme
/// loaded with `files`: its pixels, and the key label's and the candidate's fonts.
pub fn draw(files: &[(String, String)]) -> (Vec<u8>, [Font; 2]) {
    let mut theme = theme(files);
    let r = Rect {
        left: 10.0,
        top: 10.0,
        right: SIZE.0 as f32 - 10.0,
        bottom: SIZE.1 as f32 - STRIP - 10.0,
    };
    let mut scene = Scene {
        width: SIZE.0,
        height: SIZE.1,
        strip: STRIP,
        keys: vec![Key {
            code: 104,
            bounds: Bounds {
                touch: r,
                visible: r,
            },
            label: "Hello".into(),
            hint: String::new(),
            icon: String::new(),
            enabled: true,
            styles: Default::default(),
            hints: Default::default(),
        }],
        candidates: vec!["Smart".into()],
        ..Scene::default()
    };
    theme.apply(
        &mut scene,
        "characters",
        "unshifted",
        &["character".into()],
        &[0],
    );
    let fonts = [
        scene.keys[0].styles[0].font,
        scene.candidate_styles[0][0].text.font,
    ];
    let mut raster = Raster::new(SIZE.0, SIZE.1, vec![roboto()]);
    let full = Scene::default().damage(&scene).unwrap();
    raster.draw(&scene, full).unwrap();
    (raster.pixels().to_vec(), fonts)
}

/// The pixels that differ between `a` and `b` among those `inside` picks by (x, y).
pub fn differing(a: &[u8], b: &[u8], inside: impl Fn(usize, usize) -> bool) -> usize {
    assert_eq!(a.len(), b.len());
    let width = SIZE.0 as usize;
    a.chunks(4)
        .zip(b.chunks(4))
        .enumerate()
        .filter(|(i, (p, q))| inside(i % width, i / width) && p != q)
        .count()
}

/// Each shaped run of `sample` and the full name of the face it was shaped in.
pub fn faces(
    text: &mut Text,
    sample: &str,
    font: Font,
    weight: i32,
    italic: bool,
) -> Vec<(String, String)> {
    let layout = text.layout_font(sample, 40.0, font, weight, italic);
    let mut runs = vec![];
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(run) = item {
                let run = run.run();
                let data = run.font();
                let face = FontRef::from_index(data.data.data(), data.index).unwrap();
                let name = face
                    .localized_strings(StringId::FULL_NAME)
                    .english_or_first();
                runs.push((
                    sample[run.text_range()].to_string(),
                    name.unwrap().to_string(),
                ));
            }
        }
    }
    runs
}

/// Every glyph of `sample` as laid out: its face's file length, id and position bits.
pub fn glyphs(text: &mut Text, sample: &str, font: Font) -> Vec<(usize, u32, u32, u32)> {
    let layout = text.layout_weight(sample, 40.0, font, 400);
    let mut out = vec![];
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(run) = item {
                let len = run.run().font().data.data().len();
                out.extend(
                    run.positioned_glyphs()
                        .map(|g| (len, g.id, g.x.to_bits(), g.y.to_bits())),
                );
            }
        }
    }
    out
}

/// The request for `element` in the window, unshifted characters, at rest.
pub fn window(element: &str) -> Request {
    Request {
        element: element.into(),
        mode: "characters".into(),
        shift: "unshifted".into(),
        code: 0,
        key_type: String::new(),
        selector: 0,
        auto_commit: false,
        popup_source: None,
    }
}
