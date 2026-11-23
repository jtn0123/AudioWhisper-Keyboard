use super::{BACKGROUND, asset, backdrop, luminance, window};
use crate::calls::rule;
use crate::inline::Palette;
use snygg::text;
use wire::ime_theme::{backdrop as out, window as row};

const SCHEMA: &str = r#""$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet""#;

fn sheet(window: &str) -> String {
    format!(r#"{{{SCHEMA},"window":{{{window}}}}}"#)
}

fn found(window_style: &str) -> super::Window {
    window(
        &sheet(window_style),
        &text("characters"),
        &text("unshifted"),
        &Palette::default(),
    )
}

#[test]
fn a_theme_without_a_window_background_draws_black_with_light_buttons() {
    let plain = found("");
    assert_eq!(plain.background, BACKGROUND);
    assert_eq!((plain.image, plain.scale), (None, 0));
    assert!(!plain.light_bars && !plain.contrast);
}

#[test]
fn a_light_background_darkens_the_navigation_buttons() {
    let light = found(r##""background":"#eeeeee""##);
    assert_eq!(light.background, 0xFFEE_EEEE);
    assert!(light.light_bars && !light.contrast);
    let dark = found(r##""background":"#202124""##);
    assert!(!dark.light_bars);
}

#[test]
fn an_image_is_found_in_the_themes_directory_with_its_scale() {
    let style = r##""background":"#eeeeee","background-image":"uri(`flex:/images/bg.png`)","content-scale":"fit""##;
    let imaged = found(style);
    assert_eq!(imaged.image.as_deref(), Some("images/bg.png"));
    assert_eq!(imaged.scale, 4);
    // Over an image Android keeps the bar readable, and the buttons stay light.
    assert!(imaged.contrast && !imaged.light_bars);
}

#[test]
fn an_asset_never_leaves_the_themes_directory() {
    let path = |uri: &str| asset(&text(uri));
    assert_eq!(path("flex:/a/./b/../c.png").as_deref(), Some("a/c.png"));
    assert_eq!(
        path("flex:///a/b%20c.png?x#y").as_deref(),
        Some("a/b c.png")
    );
    assert_eq!(path("flex:/../secret"), None);
    assert_eq!(path("flex://host/a.png"), None);
    assert_eq!(path("flex:/a/%zz"), None);
    assert_eq!(path("file:/a.png"), None);
    assert_eq!(path("flex:/"), None);
}

#[test]
fn luminance_is_srgbs() {
    assert_eq!(luminance([1.0, 1.0, 1.0, 1.0]), 1.0);
    assert_eq!(luminance([0.0, 0.0, 0.0, 1.0]), 0.0);
    // Mid grey: (0.5 + 0.055) / 1.055 to the 2.4th.
    assert!((luminance([0.5, 0.5, 0.5, 1.0]) - 0.214_041).abs() < 1e-5);
}

#[test]
fn an_image_is_scaled_and_centred_as_compose_drew_it() {
    let (image, canvas) = ([100.0, 50.0], [300.0, 300.0]);
    // Crop fills the larger way and overhangs; fit fills the smaller and leaves bands.
    assert_eq!(backdrop(0, image, canvas), [-150, 0, 450, 300]);
    assert_eq!(backdrop(4, image, canvas), [0, 75, 300, 225]);
    assert_eq!(backdrop(1, image, canvas), [0, 0, 300, 300]);
    assert_eq!(backdrop(6, image, canvas), [100, 125, 200, 175]);
    assert_eq!(backdrop(5, image, canvas), [100, 125, 200, 175]);
    // Odd halves round outward.
    assert_eq!(backdrop(6, [3.0, 3.0], [10.0, 10.0]), [3, 3, 7, 7]);
}

#[test]
fn the_window_rule_answers_a_backdrop_row() {
    let style = r##""background":"#eeeeee","background-image":"uri(`flex:/bg.png`)""##;
    let mut fields = vec![vec![]; row::SIZE];
    for (at, value) in [
        (row::STYLESHEET, sheet(style)),
        (row::MODE, "characters".into()),
        (row::SHIFT, "unshifted".into()),
        (row::IMAGE_WIDTH, "100".into()),
        (row::IMAGE_HEIGHT, "50".into()),
        (row::SURFACE_WIDTH, "300".into()),
        (row::SURFACE_HEIGHT, "300".into()),
    ] {
        fields[at] = text(&value);
    }
    let answer = rule(&text("window"), &fields).unwrap();
    assert_eq!(
        answer[out::BACKGROUND],
        text(&(0xFFEE_EEEEu32 as i32).to_string())
    );
    assert_eq!(answer[out::IMAGE], text("bg.png"));
    assert_eq!(
        answer[out::LEFT..=out::BOTTOM],
        ["-150", "0", "450", "300"].map(text)
    );
    fields[row::IMAGE_WIDTH] = text("0");
    let unknown = rule(&text("window"), &fields).unwrap();
    assert_eq!(
        unknown[out::LEFT..=out::BOTTOM],
        ["0", "0", "0", "0"].map(text)
    );
    assert!(rule(&text("window"), &fields[..2]).is_err());
}
