//! The accent colour dialogs' maths (settings_ui::color): Compose's HSV and sRGB packing,
//! Jetpref's text formats and parser, luminance and the stored form. SettingsOtherThemeMatchTest
//! holds the same functions against the Kotlin libraries themselves.
use render::settings_ui::color::{self, Format, PRESETS, UNSPECIFIED};

#[test]
fn hsv_corners_and_primaries() {
    assert_eq!(color::hsv(0.0, 0.0, 0.0, 1.0), 0xff000000);
    assert_eq!(color::hsv(0.0, 0.0, 1.0, 1.0), 0xffffffff);
    assert_eq!(color::hsv(0.0, 1.0, 1.0, 1.0), 0xffff0000);
    assert_eq!(color::hsv(120.0, 1.0, 1.0, 1.0), 0xff00ff00);
    assert_eq!(color::hsv(240.0, 1.0, 1.0, 1.0), 0xff0000ff);
    // The hue bar's right end is 360, which wraps to red again.
    assert_eq!(color::hsv(360.0, 1.0, 1.0, 1.0), 0xffff0000);
    assert_eq!(color::hsv(60.0, 1.0, 0.5, 1.0), 0xff808000);
    assert_eq!(color::hsv(0.0, 1.0, 1.0, 0.5) >> 24, 0x80);
}

#[test]
fn to_hsv_of_primaries_and_greys() {
    assert_eq!(color::to_hsv(0xffff0000), [0.0, 1.0, 1.0, 1.0]);
    assert_eq!(color::to_hsv(0xff00ff00), [120.0, 1.0, 1.0, 1.0]);
    assert_eq!(color::to_hsv(0xff0000ff), [240.0, 1.0, 1.0, 1.0]);
    assert_eq!(color::to_hsv(0xffff00ff), [300.0, 1.0, 1.0, 1.0]);
    // A grey has no hue and no saturation; black has no saturation either.
    assert_eq!(color::to_hsv(0xff808080)[..2], [0.0, 0.0]);
    assert_eq!(color::to_hsv(0xff000000), [0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn hsv_round_trips_every_preset_and_stays_within_a_step_everywhere() {
    for p in PRESETS {
        let [h, s, v, a] = color::to_hsv(p);
        assert_eq!(color::hsv(h, s, v, a), p, "{p:08x}");
    }
    // Every 8-bit colour on a stride that reaches every channel value: Compose's float maths
    // may land one step off, never more.
    let mut c = 0u32;
    while c <= 0xffffff {
        let argb = 0xff000000 | c;
        let [h, s, v, a] = color::to_hsv(argb);
        assert!(
            (0.0..360.0).contains(&h) && (0.0..=1.0).contains(&s),
            "{argb:08x}"
        );
        let back = color::hsv(h, s, v, a);
        for shift in [0, 8, 16, 24] {
            let (x, y) = ((argb >> shift) & 0xff, (back >> shift) & 0xff);
            assert!(x.abs_diff(y) <= 1, "{argb:08x} -> {back:08x}");
        }
        c += 0x010101 * 3 + 0x10000;
    }
}

#[test]
fn formats_hex_rgb_and_hsv_without_alpha() {
    assert_eq!(color::format(Format::Hex, 0xff6275ac), "#6275ac");
    assert_eq!(color::format(Format::Hex, 0x80000000), "#000000");
    assert_eq!(color::format(Format::Rgb, 0xff0a00ff), "rgb(10, 0, 255)");
    assert_eq!(color::format(Format::Hsv, 0xffff0000), "hsv(0, 100, 100)");
    assert_eq!(color::format(Format::Hsv, 0xff808080), "hsv(0, 0, 50)");
}

#[test]
fn parses_hex_in_either_case_and_rejects_alpha_and_bad_lengths() {
    assert_eq!(color::parse("#6275ac"), Some((0xff6275ac, Format::Hex)));
    assert_eq!(color::parse("#6275AC"), Some((0xff6275ac, Format::Hex)));
    // Kotlin's trim takes any Unicode space from the ends, not only ASCII.
    assert_eq!(
        color::parse("\u{3000} #000000\t\n"),
        Some((0xff000000, Format::Hex))
    );
    assert_eq!(
        color::parse("\u{1f}#000000"),
        Some((0xff000000, Format::Hex))
    );
    for bad in [
        "#6275ac80",
        "#6275a",
        "#6275acc",
        "6275ac",
        "#62 75ac",
        "#gg0000",
        "",
        "#",
        "\u{85}#000000",
    ] {
        assert_eq!(color::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn parses_rgb_and_hsv_with_jetpref_number_rules() {
    assert_eq!(
        color::parse("rgb(255, 0, 10)"),
        Some((0xffff000a, Format::Rgb))
    );
    assert_eq!(color::parse("RGBA(1,2,3)"), Some((0xff010203, Format::Rgb)));
    assert_eq!(
        color::parse("rgb( 0 ,\t0 , 0 )"),
        Some((0xff000000, Format::Rgb))
    );
    assert_eq!(
        color::parse("hsv(120, 100, 100)"),
        Some((0xff00ff00, Format::Hsv))
    );
    assert_eq!(
        color::parse("HsVa(360,0,0)"),
        Some((0xff000000, Format::Hsv))
    );
    for bad in [
        "rgb(256, 0, 0)",
        "rgb(01, 0, 0)",
        "rgb(1, 2)",
        "rgb(1, 2, 3, 0.5)",
        "rgb(1, 2, 3, 1)",
        "rgb (1, 2, 3)",
        "rgb(-1, 2, 3)",
        "rgb(1,\u{a0}2, 3)",
        "hsv(361, 0, 0)",
        "hsv(0, 101, 0)",
        "hsv(0, 00, 0)",
        "hsl(0, 0, 0)",
        "rgb(1, 2, 3",
    ] {
        assert_eq!(color::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn luminance_picks_the_tick_colour() {
    assert_eq!(color::luminance(0xff000000), 0.0);
    assert!((color::luminance(0xffffffff) - 1.0).abs() < 1e-9);
    // Preset yellow takes a black tick, preset blue a white one.
    assert!(color::luminance(0xffffeb3b) >= 0.5);
    assert!(color::luminance(0xff2196f3) < 0.5);
}

#[test]
fn stored_form_round_trips_and_the_system_accent_stores_as_unspecified() {
    assert_eq!(color::decode(UNSPECIFIED), None);
    assert_eq!(color::decode("ff6275ac00000000"), Some(0xff6275ac));
    assert_eq!(color::decode("nonsense"), None);
    // A colour in another colour space (non-zero low bits) has no sRGB form here.
    assert_eq!(color::decode("3c003c003c000001"), None);
    assert_eq!(color::encode(0xff6275ac, 0xff4caf50), "ff6275ac00000000");
    assert_eq!(color::encode(0xff4caf50, 0xff4caf50), UNSPECIFIED);
    assert_eq!(
        color::decode(&color::encode(0xff112233, 0)),
        Some(0xff112233)
    );
}

#[test]
fn pictures_have_their_size_shape_and_ends() {
    let plane = color::plane(100, 100, 0xffff0000, 4.0);
    assert_eq!(plane.len(), 100 * 100);
    // Top right is nearly the hue, the bottom nearly black, the cut corner clear.
    assert_eq!(plane[0] as u32 >> 24, 0);
    let top_right = plane[10 * 100 + 98] as u32;
    assert!(
        (top_right >> 16) & 0xff > 0xe0 && (top_right & 0xff) < 0x20,
        "{top_right:08x}"
    );
    assert!((plane[98 * 100 + 50] as u32) & 0xffffff < 0x060606);
    let bar = color::hue_bar(120, 12);
    assert_eq!(bar[0] as u32 >> 24, 0, "the pill's corner is clear");
    assert!(
        bar[6 * 120] as u32 >> 24 > 0xf0,
        "its end's middle is drawn"
    );
    let middle = bar[6 * 120 + 60] as u32;
    assert!(
        middle >> 16 & 0xff == 0 && middle >> 8 & 0xff > 0xf0 && middle & 0xff == 0xff,
        "cyan in the middle"
    );
    let check = color::checkered(24, 6.0, 0x29000000);
    assert_eq!(check[12 * 24 + 1] as u32 >> 24, 0x29);
    assert_eq!(check[12 * 24 + 7] as u32 >> 24, 0);
}
