//! Android's hinted `TextMotion.Static` layout, replayed from a device fixture.
//!
//! `fixtures/android-roboto-light.json` is recorded by SettingsTypographyFixture (the `metrics`
//! device mode) with the tracked Roboto-Light: StaticLayout line ranges, widths and carets, and
//! TextRunShaper glyph ids and pen positions, at every size settings uses (14/16/22/24 sp at
//! font scale 1 and 2), with and without Material letter spacing, unwrapped and at two widths.
//! Static strings only.
use render::text::Text;
use serde_json::Value;

fn font() -> Text {
    Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ])
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/android-roboto-light.json")).unwrap()
}

fn num(v: &Value) -> f32 {
    v.as_f64().unwrap() as f32
}

#[derive(Default)]
struct Worst {
    lines: usize,
    glyphs: usize,
    carets: usize,
    x: f32,
    caret: f32,
    width: f32,
}

fn check_line(
    line: &render::android_text::Line,
    expected: &Value,
    lead: f32,
    at: &str,
    w: &mut Worst,
) {
    let range =
        expected["start"].as_u64().unwrap() as usize..expected["end"].as_u64().unwrap() as usize;
    assert_eq!(line.range, range, "line range for {at}");
    let drawn: Vec<_> = line.runs.iter().flat_map(|r| r.glyphs.iter()).collect();
    let ids: Vec<u32> = expected["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect();
    assert_eq!(
        drawn.iter().map(|g| g.0).collect::<Vec<_>>(),
        ids,
        "glyphs for {at}"
    );
    // TextRunShaper keeps the half spacing before the first cluster; StaticLayout drops it.
    for (g, x) in drawn.iter().zip(expected["x"].as_array().unwrap()) {
        w.x = w.x.max((g.1 - (num(x) - lead)).abs());
    }
    for caret in expected["carets"].as_array().unwrap() {
        let offset = caret[0].as_u64().unwrap() as usize;
        if let Some(rust) = line.carets.iter().find(|c| c.0 == offset) {
            w.caret = w.caret.max((rust.1 - num(&caret[1])).abs());
            w.carets += 1;
        }
    }
    w.width = w.width.max((line.width - num(&expected["max"])).abs());
    w.glyphs += drawn.len();
    w.lines += 1;
}

#[test]
fn android_text_matches_device_fixture() {
    let mut text = font();
    let fixture = fixture();
    let cases = fixture["cases"].as_array().unwrap();
    let mut w = Worst::default();
    for case in cases {
        let label = case["text"].as_str().unwrap();
        let size = num(&case["size"]);
        // Compose resolves sp letter spacing to px at the real size; Minikin scales it to the
        // truncated layout size.
        let spacing = num(&case["spacing_em"]) * size;
        let lead = spacing * size.trunc() / size / 2.0;
        let width = case["width"].as_u64().unwrap();
        let layout = text.android(
            label,
            size,
            400,
            [0.0, spacing],
            (width > 0).then_some(width as f32),
        );
        let lines = case["lines"].as_array().unwrap();
        let at = format!("{label:?} size {size} spacing {spacing} width {width}");
        assert_eq!(layout.lines.len(), lines.len(), "line count for {at}");
        for (line, expected) in layout.lines.iter().zip(lines) {
            check_line(line, expected, lead, &at, &mut w);
        }
    }
    println!(
        "{} cases, {} lines, {} glyphs, {} carets; worst glyph x {:.4} px, caret {:.4} px, line width {:.4} px",
        cases.len(),
        w.lines,
        w.glyphs,
        w.carets,
        w.x,
        w.caret,
        w.width
    );
    assert_eq!(cases.len(), 144);
    assert!(
        w.glyphs > 7000 && w.carets > 7000,
        "{} glyphs {} carets",
        w.glyphs,
        w.carets
    );
    // Float accumulation only: one hinted pixel, one kerning unit or one spacing half would
    // each be ≥ 0.017 px.
    assert!(w.x < 0.01, "worst glyph x delta {}", w.x);
    assert!(w.caret < 0.01, "worst caret delta {}", w.caret);
    assert!(w.width < 0.01, "worst line width delta {}", w.width);
}

#[test]
fn line_edges_drop_their_outer_half_letter_spacing_as_android_draws() {
    // The device drew "Hello" at 42 px with and without 0.5 em spacing: identical left ink and
    // a line 4 gaps (not 5) wider, so neither edge keeps its outer half gap.
    let fixture = fixture();
    let ink = &fixture["edge_ink"];
    assert_eq!(ink["0.0"][0], ink["0.5"][0]);
    assert_eq!(num(&ink["0.5"][2]) - num(&ink["0.0"][2]), 4.0 * 21.0);
    let mut text = font();
    let plain = text.android("Hello", 42.0, 400, [0.0, 0.0], None);
    let spaced = text.android("Hello", 42.0, 400, [0.0, 21.0], None);
    assert_eq!(plain.width, num(&ink["0.0"][2]));
    assert_eq!(spaced.width, num(&ink["0.5"][2]));
    assert_eq!(spaced.lines[0].runs[0].glyphs[0].1, 0.0);
}

#[test]
fn truncated_layout_size_and_hinted_advances() {
    assert_eq!(render::android_text::layout_size(36.75), 36.0);
    assert_eq!(render::android_text::layout_size(36.0), 36.0);
    // A 1100-unit glyph at 36.75 px (upem 2048) lays out at 36: round(19.336) = 19, and its
    // −18-unit kerning scales to 36 too.
    let base = 1100.0 * 36.75 / 2048.0;
    let kern = -18.0 * 36.75 / 2048.0;
    let advance = render::android_text::advance(base, None, base + kern, 36.75);
    assert!(
        (advance - (19.0 - 18.0 * 36.0 / 2048.0)).abs() < 1e-4,
        "{advance}"
    );
    // A hinted font's interpreter advance replaces the rounded linear one.
    assert_eq!(
        render::android_text::advance(base, Some(21.0), base, 36.75),
        21.0
    );
}

#[test]
fn unhinted_advances_round_through_26_6_like_freetype() {
    use render::android_text::{advance, pixel_round};
    // Roboto's `e` at weight 500 and 42 px: 1097 units of 2048 is 22.497 px, which FreeType holds
    // as 1440/64 = 22.5 and FT_PIX_ROUND takes to 23 (Android's StaticLayout puts the next glyph
    // 23 px on); rounding the linear advance would give 22.
    let e = 1097.0 * 42.0 / 2048.0;
    assert_eq!(pixel_round(e), 23.0);
    assert_eq!(advance(e, None, e, 42.0), 23.0);
    // Below the 26.6 half-pixel it still rounds down: 22.48 px is 1439/64.
    assert_eq!(pixel_round(22.48), 22.0);
    assert_eq!(pixel_round(22.0), 22.0);
}

#[test]
fn hard_breaks_hyphens_and_long_words_wrap_like_static_layout() {
    let mut text = font();
    // No break after hard hyphens without hyphenation (Minikin WordBreaker).
    let layout = text.android("auto-capitalization", 42.0, 400, [0.0, 0.0], Some(150.0));
    assert!(layout.lines.iter().all(|l| l.range.start != 5));
    // A trailing newline keeps an empty last line; widths exclude hanging spaces.
    let layout = text.android("One two \n", 42.0, 400, [0.0, 0.0], None);
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[1].range, 9..9);
    let one_two = text.android("One two", 42.0, 400, [0.0, 0.0], None);
    assert_eq!(layout.lines[0].width, one_two.width);
    assert_eq!(
        layout.height,
        layout.lines.iter().map(|l| l.height).sum::<f32>()
    );
}

#[test]
fn caret_selection_and_hit_testing_use_android_positions() {
    let mut text = font();
    let layout = text.android("Try Rust", 42.0, 400, [0.0, 0.0], None);
    let carets = &layout.lines[0].carets;
    let (line, x) = layout.caret(4);
    assert_eq!(line, 0);
    assert_eq!(x, carets.iter().find(|c| c.0 == 4).unwrap().1);
    assert_eq!(layout.index_at(x + 0.4, 1.0), 4);
    let rects = layout.selection(0, 3);
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0][0], 0.0);
    assert_eq!(rects[0][2], layout.caret(3).1);
    assert_eq!(layout.caret(8).1, layout.lines[0].width);
}

#[test]
fn empty_text_keeps_one_line_for_the_caret() {
    let mut text = font();
    let layout = text.android("", 42.0, 400, [0.0, 0.0], None);
    let parley = text.layout_typography("", 42.0, false, 400, false, [0.0, 0.0]);
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.height, parley.height());
    assert!(layout.height > 40.0);
    assert_eq!(layout.caret(0), (0, 0.0));
}

#[test]
fn line_boxes_are_compose_whole_pixel_lines() {
    use render::android_text::line_metrics;
    // Roboto's hhea metrics at the device's 2.625 density. The Compose trees in the pictures
    // show a 16 sp bodyLarge line (no line height) 49 px tall, 14/20 sp lines 53 px each (two
    // lines 106, six 318, never 52.5 apart) with the baseline 39 px down, and 22/28 sp 74.
    let roboto = |px: f32| line_metrics(px * 1900.0 / 2048.0, px * 500.0 / 2048.0, 0.0);
    let spaced = |px: f32, line: f32| line_metrics(px * 1900.0 / 2048.0, px * 500.0 / 2048.0, line);
    assert_eq!(roboto(42.0), (49.0, 39.0));
    assert_eq!(spaced(36.75, 52.5), (53.0, 39.0));
    assert_eq!(spaced(57.75, 73.5).0, 74.0);
    let mut text = font();
    let long = "The initial character selected after long press is dynamically decided";
    let layout = text.android(long, 36.75, 400, [52.5, 0.0], Some(260.0));
    assert!(layout.lines.len() >= 4, "{} lines", layout.lines.len());
    for (i, line) in layout.lines.iter().enumerate() {
        assert_eq!(line.top, 53.0 * i as f32);
        assert_eq!(line.height, 53.0);
        assert_eq!(line.baseline.fract(), 0.0);
    }
    assert_eq!(layout.height, 53.0 * layout.lines.len() as f32);
}
