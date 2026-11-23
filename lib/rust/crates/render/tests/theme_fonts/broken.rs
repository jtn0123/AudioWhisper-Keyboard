//! A file that is not a whole font, or not there, is no font: the theme draws exactly as one with
//! no `@font` files, and nothing panics.
use super::common::*;
use render::{
    text::{Font, Text},
    theme_fonts,
};
use std::path::PathBuf;

fn scratch(name: &str, bytes: &[u8]) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("theme-fonts-broken");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path.display().to_string()
}

/// Bytes no font parser accepts, from a fixed linear congruential sequence.
fn garbage(len: usize) -> Vec<u8> {
    let mut state = 0x2545_f491_u32;
    (0..len)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 24) as u8
        })
        .collect()
}

/// The broken files: truncated, garbage, garbage behind a TrueType header, empty, without a
/// character map, missing, a directory.
fn broken() -> Vec<(&'static str, String)> {
    let regular = std::fs::read(fixture(REGULAR.1)).unwrap();
    let mut headed = regular[..12].to_vec();
    headed.extend(garbage(2048));
    let missing = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("theme-fonts-missing.ttf");
    // Every table there, but the character map's directory entry renamed: no `cmap`.
    let mut unmapped = regular.clone();
    let tables = u16::from_be_bytes([regular[4], regular[5]]) as usize;
    let at = (0..tables)
        .map(|i| 12 + 16 * i)
        .find(|&r| &regular[r..r + 4] == b"cmap");
    unmapped[at.unwrap() + 3] = b'X';
    vec![
        (
            "truncated",
            scratch("truncated.ttf", &regular[..regular.len() / 2]),
        ),
        ("header only", scratch("header.ttf", &regular[..64])),
        (
            "short by 16 bytes",
            scratch("short.ttf", &regular[..regular.len() - 16]),
        ),
        ("garbage", scratch("garbage.ttf", &garbage(4096))),
        ("garbage after a header", scratch("headed.ttf", &headed)),
        ("empty", scratch("empty.ttf", &[])),
        ("no character map", scratch("unmapped.ttf", &unmapped)),
        ("missing", missing.display().to_string()),
        ("a directory", env!("CARGO_TARGET_TMPDIR").to_string()),
    ]
}

#[test]
fn a_broken_or_missing_file_draws_in_the_default_fonts_without_a_crash() {
    let (plain, _) = draw(&[]);
    for (case, path) in broken() {
        assert!(theme_fonts::read(&path).is_none(), "{case} read as a font");
        let files = vec![
            (REGULAR.0.to_string(), path.clone()),
            (BOLD.0.to_string(), path),
        ];
        let mut theme = theme(&files);
        assert!(theme.fonts().is_empty(), "{case} loaded");
        assert_eq!(theme.style(&window("key")).font, Font::Sans, "{case}");
        let (pixels, fonts) = draw(&files);
        assert_eq!(fonts, [Font::Sans; 2], "{case}");
        assert!(
            pixels == plain,
            "{case} drew differently from no font at all"
        );
    }
    eprintln!(
        "broken font cases drawn as the default chain: {}",
        broken().len()
    );
}

#[test]
fn an_unresolved_uri_is_no_face_and_a_bigger_than_allowed_file_is_not_read() {
    // Kotlin leaves out a URI its asset resolver refused, so the rule has no file at all.
    assert!(theme(&[]).fonts().is_empty());
    let unrelated = vec![(
        "flex:/other.ttf".to_string(),
        fixture(REGULAR.1).display().to_string(),
    )];
    assert!(theme(&unrelated).fonts().is_empty());
    // A whole font, padded past the limit, is still not read.
    let huge = scratch("huge.ttf", &std::fs::read(fixture(REGULAR.1)).unwrap());
    let file = std::fs::OpenOptions::new().write(true).open(&huge).unwrap();
    file.set_len(theme_fonts::MAX_FILE + 1).unwrap();
    assert!(theme_fonts::read(&huge).is_none());
    std::fs::remove_file(&huge).unwrap();
}

#[test]
fn one_broken_face_leaves_the_rest_of_the_font() {
    let truncated = broken().remove(0).1;
    let files = vec![files(&[REGULAR]).remove(0), (BOLD.0.to_string(), truncated)];
    let mut theme = theme(&files);
    let font = theme.style(&window("key")).font;
    assert!(matches!(font, Font::Theme(_)));
    let mut text = Text::new([roboto()]);
    // Bold asks for the face that did not load and gets the nearest one that did.
    let bold = faces(&mut text, "Hello", font, 700, false);
    assert_eq!(bold, [("Hello".into(), "Fixture Script Regular".into())]);
}

#[test]
fn the_whole_check_accepts_every_bundled_font() {
    for name in [
        REGULAR.1,
        BOLD.1,
        "noto-sans-hebrew-subset.ttf",
        "noto-sans-cjk-subset.otf",
        "noto-color-emoji-subset.ttf",
        "noto-color-emoji-flags-subset.ttf",
    ] {
        assert!(
            theme_fonts::whole(&std::fs::read(fixture(name)).unwrap()),
            "{name}"
        );
    }
    assert!(theme_fonts::whole(&roboto()));
}
