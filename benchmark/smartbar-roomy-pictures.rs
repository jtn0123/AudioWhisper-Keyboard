//! Host pictures of the Smartbar main row on the phone's settings: 1440 px wide, density 3.75,
//! shared layout, flipped toggles, the sticky mic and the speaker, three suggestions.
//!
//! cargo run -p render --example smartbar_roomy_pictures -- <out.png> <theme.json> <font.ttf> [words|empty|dictating]
//!
//! The same file builds against the base commit, whose config takes 14 facts: the words,
//! dictating and reading facts then fall away and the row is drawn as it was.
use render::{
    raster::Raster,
    scene::Scene,
    smartbar::Config,
    smartbar_scene::{Action, Smartbar},
    text::Text,
    theme::{Request, Theme},
};

const WIDTH: u16 = 1440;
const DENSITY: f32 = 3.75;
/// 40 dp: the Smartbar's height at the default keyboard height, rounded.
const ROW: f32 = 150.0;
const WORDS: [&str; 3] = ["the", "wonderful", "unfortunately"];
// Material icons, 24-unit viewport: KeyboardVoice, VolumeUp, SentimentSatisfiedAlt, KeyboardArrowLeft.
const MIC: &str = "M12 15c1.66 0 2.99-1.34 2.99-3L15 6c0-1.66-1.34-3-3-3S9 4.34 9 6v6c0 1.66 1.34 3 3 3zm5.3-3c0 3-2.54 5.1-5.3 5.1S6.7 15 6.7 12H5c0 3.42 2.72 6.23 6 6.72V22h2v-3.28c3.28-.48 6-3.3 6-6.72h-1.7z";
const SPEAKER: &str = "M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z";
const EMOJI: &str = "M15.5 8A1.5 1.5 0 1 0 15.5 11A1.5 1.5 0 1 0 15.5 8zM8.5 8A1.5 1.5 0 1 0 8.5 11A1.5 1.5 0 1 0 8.5 8zM12 16c-1.48 0-2.75-.81-3.45-2H6.88c.8 2.05 2.79 3.5 5.12 3.5s4.32-1.45 5.12-3.5h-1.67c-.7 1.19-1.97 2-3.45 2zm-.01-14C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8z";
const ARROW: &str = "M15.41 16.59L10.83 12l4.58-4.59L14 6l-6 6 6 6 1.41-1.41z";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let state = args.get(3).map_or("words", String::as_str);
    let (words, dictating) = match state {
        "words" => (1.0, 0.0),
        "empty" => (0.0, 0.0),
        "dictating" => (1.0, 1.0),
        _ => panic!("state is words, empty or dictating"),
    };
    let facts = [
        1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, DENSITY, ROW, 0.0, 0.0, words, dictating,
        0.0,
    ];
    let config = Config::decode(&facts)
        .or_else(|| Config::decode(&facts[..14]))
        .expect("config");
    let theme = std::fs::read_to_string(&args[1]).expect("theme");
    let mut theme = Theme::new(&theme, Vec::new(), Vec::new(), DENSITY, 1.0, 1.0).expect("theme");
    let font = std::fs::read(&args[2]).expect("font");
    // Sticky mic, speaker, emoji, overflow and the shared toggle, as RendererSmartbar sends them.
    let icons = [MIC, SPEAKER, EMOJI, "", ARROW];
    let actions = icons
        .iter()
        .enumerate()
        .map(|(i, icon)| Action {
            code: if i == 0 { -233 } else { 0 },
            enabled: true,
            lit: if i == 0 && dictating != 0.0 { 1 } else { 0 },
            level: 0.4,
            icon: (*icon).into(),
            ..Action::default()
        })
        .collect();
    let candidates: Vec<String> = if words != 0.0 {
        WORDS.iter().map(|w| (*w).into()).collect()
    } else {
        Vec::new()
    };
    let mut scene = Scene {
        width: WIDTH,
        height: ROW as u16,
        strip: ROW,
        keyboard_height: 0.0,
        candidates,
        smartbar: Some(Smartbar::new(config, actions)),
        ..Scene::default()
    };
    theme.apply(&mut scene, "characters", "unshifted", &[], &[]);
    let base = Request {
        element: String::new(),
        mode: "characters".into(),
        shift: "unshifted".into(),
        code: 0,
        key_type: String::new(),
        selector: 0,
        auto_commit: false,
        popup_source: None,
    };
    let mut text = Text::new([font.clone()]);
    Smartbar::prepare(&mut scene, &mut theme, &base, &mut text, &[]);
    let mut raster = Raster::new(WIDTH, ROW as u16, vec![font]);
    raster
        .draw(&scene, Scene::default().damage(&scene).expect("damage"))
        .expect("draw");
    std::fs::write(&args[0], png(WIDTH.into(), ROW as u32, raster.pixels())).expect("write");
    let bar = scene.smartbar.as_ref().expect("smartbar");
    println!(
        "{state}: actions={} center={:.0}..{:.0} slots={:?}",
        bar.layout.actions.len(),
        bar.layout.center.left,
        bar.layout.center.right,
        bar.layout
            .candidates
            .iter()
            .map(|s| (s.bounds.left, s.bounds.right))
            .collect::<Vec<_>>()
    );
}

/// RGBA8 to PNG with stored (uncompressed) deflate blocks: no compression crate needed.
fn png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity(rgba.len() + height as usize);
    for row in rgba.chunks(width as usize * 4) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let blocks: Vec<_> = raw.chunks(65_535).collect();
    for (i, block) in blocks.iter().enumerate() {
        z.push(u8::from(i + 1 == blocks.len()));
        let n = block.len() as u16;
        z.extend(n.to_le_bytes());
        z.extend((!n).to_le_bytes());
        z.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &raw {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend((b << 16 | a).to_be_bytes());
    let mut header = width.to_be_bytes().to_vec();
    header.extend(height.to_be_bytes());
    header.extend([8, 6, 0, 0, 0]);
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    for (kind, data) in [(&b"IHDR"[..], header), (b"IDAT", z), (b"IEND", Vec::new())] {
        out.extend((data.len() as u32).to_be_bytes());
        let start = out.len();
        out.extend_from_slice(kind);
        out.extend(&data);
        out.extend(crc(&out[start..]).to_be_bytes());
    }
    out
}
fn crc(bytes: &[u8]) -> u32 {
    let mut c = !0u32;
    for byte in bytes {
        c ^= u32::from(*byte);
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xedb8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
    }
    !c
}
