//! Static UI-label probe: compares `Text::android` glyph ids and pen positions with a device
//! receipt from SettingsTypographyDevice (`compose-metrics.json`). Prints numbers only.
//!
//! cargo run -p render --example settings_font_probe -- <receipt.json> <Roboto-Regular.ttf>...
use render::text::Text;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&args[0]).expect("receipt")).expect("json");
    let fonts: Vec<_> = args[1..]
        .iter()
        .map(|p| std::fs::read(p).expect("font"))
        .collect();
    let mut text = Text::new(fonts);
    let label = "Try Rust drawing for text keys, suggestions";
    let mut worst = 0.0f64;
    for run in receipt["runs"].as_array().expect("runs") {
        let Some(expected) = run["glyph_x"].as_array() else {
            continue;
        };
        if run["config"] != "compose-static" {
            continue;
        }
        let size = run["size"].as_f64().unwrap() as f32;
        // Callers pass spacing in px at the real size (em × size), as Compose resolves it.
        let spacing = run["spacing_em"].as_f64().unwrap() as f32 * size;
        let layout = text.android(label, size, 400, [0.0, spacing], None);
        let glyphs: Vec<_> = layout.lines[0]
            .runs
            .iter()
            .flat_map(|r| r.glyphs.iter())
            .collect();
        let ids: Vec<u64> = run["glyph_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        assert_eq!(
            ids,
            glyphs.iter().map(|g| u64::from(g.0)).collect::<Vec<_>>()
        );
        // TextRunShaper keeps the half spacing before the first cluster; StaticLayout drops it.
        let lead = f64::from(spacing * size.trunc() / size / 2.0);
        let delta = expected
            .iter()
            .zip(&glyphs)
            .map(|(e, g)| (e.as_f64().unwrap() - lead - f64::from(g.1)).abs())
            .fold(0.0, f64::max);
        worst = worst.max(delta);
        println!(
            "size={size} spacing_px={spacing:.5} glyphs={} android_advance={} rust_width={:.5} max_glyph_x_delta={delta:.6}",
            glyphs.len(),
            run["advance"],
            layout.width,
        );
    }
    println!("worst_glyph_x_delta={worst:.6}");
}
