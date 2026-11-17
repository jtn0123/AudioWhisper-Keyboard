//! Keyboard frame timing on the host: the emulator trial's 300 presses (12 keys, 25 each) and
//! eight `e` popups, replayed as scene changes through the CPU raster's damage regions. A
//! 1080×712 QWERTY surface with the Gboard-style suggestion row, as on the 1080×2400 AVD at
//! density 2.625, styled as the default `floris_day` theme draws it. Synthetic words only. Prints
//! numbers only.
//!
//! cargo run --release -p render --example keyboard_frame_timing -- <font.ttf>...
use keys::{Bounds, Rect};
use render::{
    raster::Raster,
    scene::{CandidateStyle, Key, Popup, Scene},
    smartbar::Config,
    smartbar_scene::{Action, Smartbar},
    style::Style,
    text::Text,
};
use std::time::Instant;

const DENSITY: f32 = 2.625;
const WIDTH: u16 = 1080;
const HEIGHT: u16 = 712;
const ROW: f32 = 105.0;
/// A four-pointed star in the 24-unit icon box, standing in for the real icons.
const ICON: &str = "M12 2 L14 10 L22 12 L14 14 L12 22 L10 14 L2 12 L10 10 Z";
/// The letters the trial presses, in its order.
const TRIAL: &str = "qwerasdfzxcv";
const WORDS: [&str; 6] = ["queue", "were", "erase", "sad", "fizz", "excavate"];

fn key_style(background: u32, size: f32) -> Style {
    Style {
        background,
        foreground: 0xff000000,
        size,
        shadow: 2.0 * DENSITY,
        corners: [8.0 * DENSITY; 4],
        margin: [3.0 * DENSITY, 5.0 * DENSITY, 3.0 * DENSITY, 5.0 * DENSITY],
        ..Style::default()
    }
}

fn key(code: i32, r: Rect, label: &str, hint: &str, icon: &str) -> Key {
    let size = if label.chars().count() > 1 {
        14.0
    } else {
        22.0
    } * DENSITY;
    Key {
        code,
        bounds: Bounds {
            touch: r,
            visible: r,
        },
        label: label.into(),
        hint: hint.into(),
        icon: icon.into(),
        styles: [key_style(0xffffffff, size), key_style(0xfff5f5f5, size)],
        hints: std::array::from_fn(|_| Style {
            foreground: 0xff5f5f5f,
            size: 12.0 * DENSITY,
            mono: true,
            ..Style::default()
        }),
        enabled: true,
    }
}

/// Rows of (label, hint, icon, width in key units) laid out 108 px a unit, 150 px a row.
fn keys() -> Vec<Key> {
    let rows: [&[(&str, &str, &str, f32)]; 4] = [
        &[
            ("q", "1", "", 1.0),
            ("w", "2", "", 1.0),
            ("e", "3", "", 1.0),
            ("r", "4", "", 1.0),
            ("t", "5", "", 1.0),
            ("y", "6", "", 1.0),
            ("u", "7", "", 1.0),
            ("i", "8", "", 1.0),
            ("o", "9", "", 1.0),
            ("p", "0", "", 1.0),
        ],
        &[
            ("", "", "", 0.5),
            ("a", "@", "", 1.0),
            ("s", "#", "", 1.0),
            ("d", "$", "", 1.0),
            ("f", "%", "", 1.0),
            ("g", "&", "", 1.0),
            ("h", "-", "", 1.0),
            ("j", "+", "", 1.0),
            ("k", "(", "", 1.0),
            ("l", ")", "", 1.0),
        ],
        &[
            ("", "", ICON, 1.5),
            ("z", "*", "", 1.0),
            ("x", "\"", "", 1.0),
            ("c", "'", "", 1.0),
            ("v", ":", "", 1.0),
            ("b", ";", "", 1.0),
            ("n", "!", "", 1.0),
            ("m", "?", "", 1.0),
            ("", "", ICON, 1.5),
        ],
        &[
            ("?123", "", "", 1.5),
            (",", "", "", 1.0),
            ("", "", ICON, 1.0),
            ("English (United States)", "", "", 4.0),
            (".", "", "", 1.0),
            ("", "", ICON, 1.5),
        ],
    ];
    let mut out = Vec::new();
    for (row, cells) in rows.iter().enumerate() {
        let mut left = 0.0;
        for &(label, hint, icon, units) in cells.iter() {
            let r = Rect {
                left,
                right: left + units * 108.0,
                top: row as f32 * 150.0,
                bottom: (row + 1) as f32 * 150.0,
            };
            left = r.right;
            if label.is_empty() && icon.is_empty() {
                continue;
            }
            let code = label.chars().next().map_or(-1, |c| c as i32);
            out.push(key(code, r, label, hint, icon));
        }
    }
    out
}

fn smartbar(text: &mut Text, words: &[String]) -> Smartbar {
    // Enabled, shared layout, flipped, speaker and sticky mic, words showing (Gboard's row).
    let facts = [
        1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 19.0, 0.0, DENSITY, ROW, 0.0, 0.0, 1.0, 0.0, 0.0,
    ];
    // Transparent action keys, and the shared-actions toggle (the fifth) a raised white disc.
    let action = Style {
        foreground: 0xff121212,
        size: 24.0 * DENSITY,
        corners: [8.0 * DENSITY; 4],
        ..Style::default()
    };
    let toggle = Style {
        background: 0xffffffff,
        foreground: 0xff000000,
        shape: snygg::value::ShapeKind::Circle,
        margin: [6.0 * DENSITY; 4],
        shadow: 2.0 * DENSITY,
        ..action.clone()
    };
    let actions = (0..5)
        .map(|code| {
            let style = if code == 4 { &toggle } else { &action };
            Action {
                code,
                enabled: true,
                icon: ICON.into(),
                styles: [style.clone(), style.clone()],
                ..Action::default()
            }
        })
        .collect();
    let mut bar = Smartbar::new(Config::decode(&facts).expect("smartbar facts"), actions);
    bar.intrinsic = words
        .iter()
        .map(|w| text.measure(w, 14.0 * DENSITY, false, 400) + 8.0 * DENSITY)
        .collect();
    bar.spacer = Style {
        foreground: 0x40000000,
        ..Style::default()
    };
    bar.relayout(WIDTH as f32);
    bar
}

fn scene(text: &mut Text, words: Vec<String>) -> Scene {
    let candidate = CandidateStyle {
        container: Style {
            margin: [4.0 * DENSITY; 4],
            padding: [0.0, 8.0 * DENSITY, 0.0, 8.0 * DENSITY],
            ..Style::default()
        },
        text: Style {
            foreground: 0xff121212,
            size: 14.0 * DENSITY,
            ..Style::default()
        },
    };
    Scene {
        keyboard_height: 600.0,
        width: WIDTH,
        height: HEIGHT,
        strip: ROW,
        background: 0xffe0e0e0,
        strip_style: Style {
            background: 0xffe0e0e0,
            ..Style::default()
        },
        smartbar: Some(smartbar(text, &words)),
        candidate_styles: vec![[candidate.clone(), candidate]; words.len()],
        popup_style: Style {
            background: 0xffeeeeee,
            size: 22.0 * DENSITY,
            corners: [8.0 * DENSITY; 4],
            shadow: 2.0 * DENSITY,
            ..Style::default()
        },
        keys: keys(),
        candidates: words,
        ..Scene::default()
    }
}

/// The trial's frames: each press, then its release with refilled words; then eight popups.
fn frames(text: &mut Text) -> Vec<(&'static str, Scene)> {
    let mut out = Vec::new();
    let base = scene(text, WORDS[..3].iter().map(|w| w.to_string()).collect());
    let index = |c: char| base.keys.iter().position(|k| k.label == c.to_string());
    let mut current = base.clone();
    let mut refill = 0;
    for letter in TRIAL.chars() {
        let k = index(letter).expect("trial key");
        for _ in 0..25 {
            let mut press = current.clone();
            press.pressed = vec![k];
            out.push(("press", press.clone()));
            refill += 1;
            let words: Vec<String> = (0..3)
                .map(|i| WORDS[(refill + i) % WORDS.len()].to_string())
                .collect();
            let mut release = scene(text, words);
            release.pressed.clear();
            out.push(("release", release.clone()));
            current = release;
        }
    }
    let e = index('e').expect("e key");
    for _ in 0..8 {
        let mut open = current.clone();
        open.pressed = vec![e];
        open.popup = Some(Popup {
            key: e,
            labels: ["3", "è", "é", "ê", "ë", "ē"].map(String::from).to_vec(),
            selected: 0,
        });
        out.push(("popup", open));
        out.push(("popup", current.clone()));
    }
    out
}

fn quantiles(mut v: Vec<f64>) -> String {
    v.sort_by(f64::total_cmp);
    let at = |q: f64| v[((v.len() - 1) as f64 * q) as usize];
    let over = v.iter().filter(|&&t| t > 2.0).count();
    format!(
        "n={} p50={:.3} p95={:.3} p99={:.3} max={:.3} over2ms={over}",
        v.len(),
        at(0.5),
        at(0.95),
        at(0.99),
        v[v.len() - 1]
    )
}

fn main() {
    let fonts: Vec<_> = std::env::args()
        .skip(1)
        .map(|p| std::fs::read(p).expect("font"))
        .collect();
    let mut text = Text::new(fonts.clone());
    let frames = frames(&mut text);
    let mut raster = Raster::new(WIDTH, HEIGHT, fonts);
    raster.set_profiling(true);
    let whole = Scene::default().damage(&frames[0].1).expect("full damage");
    let mut full = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        raster.draw(&frames[0].1, whole).expect("raster");
        full.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    println!("full redraw ms: {}", quantiles(full));
    // The first pass warms caches; the next two are measured, as warm emulator trials are.
    let mut kinds: Vec<(&str, Vec<f64>, f64, f64)> = Vec::new();
    for pass in 0..3 {
        let mut previous = frames[frames.len() - 1].1.clone();
        for (kind, scene) in &frames {
            let regions = previous.damage_regions(scene);
            let start = Instant::now();
            raster.draw_regions(scene, &regions).expect("raster");
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            previous = scene.clone();
            if pass == 0 {
                continue;
            }
            let p = raster.profile();
            for name in [*kind, "all"] {
                let at = match kinds.iter().position(|k| k.0 == name) {
                    Some(at) => at,
                    None => {
                        kinds.push((name, Vec::new(), 0.0, 0.0));
                        kinds.len() - 1
                    }
                };
                kinds[at].1.push(ms);
                kinds[at].2 += p.encode_ns as f64 / 1e6;
                kinds[at].3 += p.raster_ns as f64 / 1e6;
            }
        }
    }
    for (kind, times, encode, raster) in kinds {
        let n = times.len() as f64;
        println!(
            "{kind:8} ms: {} mean_encode={:.3} mean_raster={:.3}",
            quantiles(times),
            encode / n,
            raster / n
        );
    }
}
