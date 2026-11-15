//! Colour emoji must never make vello_cpu's 4096×4096 RGBA glyph-atlas page (64 MiB, filled
//! when made) resident again (stage4-followup4.md, A6). The only test in its binary, so no
//! other test allocates between the two readings of this process's resident size.
use keys::{Bounds, Rect};
use render::{
    raster::Raster,
    scene::{Key, Scene},
    style::Style,
};

fn resident_kib() -> u64 {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps runs");
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse()
        .expect("ps prints the resident size in KiB")
}

fn scene(labels: &[&str]) -> Scene {
    let style = Style {
        size: 63.0,
        background: 0xffffffff,
        foreground: 0xff000000,
        ..Style::default()
    };
    let keys = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let r = Rect {
                left: i as f32 * 150.0,
                right: (i + 1) as f32 * 150.0,
                top: 100.0,
                bottom: 300.0,
            };
            Key {
                code: 97 + i as i32,
                bounds: Bounds {
                    touch: r,
                    visible: r,
                },
                label: (*label).into(),
                hint: String::new(),
                icon: String::new(),
                styles: [style.clone(), style.clone()],
                hints: std::array::from_fn(|_| Style::default()),
                enabled: true,
            }
        })
        .collect();
    Scene {
        width: 1080,
        height: 400,
        keys,
        background: 0xffffffff,
        ..Scene::default()
    }
}

#[test]
fn colour_emoji_never_make_the_64_mib_atlas_page_resident() {
    let fonts = vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
        include_bytes!("fixtures/noto-color-emoji-subset.ttf").to_vec(),
        include_bytes!("fixtures/noto-color-emoji-flags-subset.ttf").to_vec(),
    ];
    let mut raster = Raster::new(1080, 400, fonts);
    raster.set_profiling(true);
    let plain = scene(&["a", "b", "c", "d", "e"]);
    let full = Scene::default().damage(&plain).unwrap();
    // Outline text first, twice, so the canvas and outline caches are already resident.
    raster.draw(&plain, full).unwrap();
    raster.draw(&plain, full).unwrap();
    let before = resident_kib();
    let emoji = scene(&["😀", "🚀", "👋", "👍🏽", "🇺🇸"]);
    for _ in 0..3 {
        raster.draw(&emoji, full).unwrap();
    }
    let grown = resident_kib().saturating_sub(before);
    let profile = raster.profile();
    assert!(
        profile.color_runs > 0,
        "the emoji were drawn from the colour fonts"
    );
    assert_eq!(profile.atlas_runs, 0);
    // The page alone is 65,536 KiB; the tiles for these glyphs are well under 1 MiB.
    assert!(
        grown < 16 * 1024,
        "drawing colour emoji grew the resident size by {grown} KiB"
    );
}
