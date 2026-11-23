// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).

//! Checks the crate against answers computed on a JVM by MaterialKolor 5.0.1's
//! own classes, written once by a harness kept outside this repository
//! (docs/phase3-compose-free-path.md). The in-repo check against MaterialKolor
//! is app/src/test's `PaletteMatchTest`, which every gate runs.
//!
//! `digests.txt` holds, for each style, darkness, contrast and spec (and
//! AMOLED for `Neutral`), an FNV-1a 64 hash of `roles()` over the first
//! 2,000 seeds of the sequence `seed_at` generates. `rows.txt` holds every
//! role for eight named seeds at Default contrast.

use std::thread;

use palette::{NAMES, Spec, Style, contrast_level, roles, scheme};

const DIGESTS: &str = include_str!("fixture/digests.txt");
const ROWS: &str = include_str!("fixture/rows.txt");
const SEEDS: u64 = 2000;

/// The seeds the harness hashes: special colours, then SplitMix64.
const SPECIALS: [u32; 28] = [
    0x000000, 0xFFFFFF, 0x808080, 0x7F7F7F, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0x00FFFF,
    0xFF00FF, 0x010101, 0xFEFEFE, 0x808081, 0x7F807F, 0x818080, 0x010000, 0x000001, 0xFFFFFE,
    0xFEFFFF, 0x020202, 0x101010, 0xEFEFEF, 0x404040, 0xC0C0C0, 0x6750A4, 0x3F51B5, 0x00796B,
    0xB3261E,
];

fn seed_at(i: u64) -> u32 {
    if let Some(&special) = SPECIALS.get(i as usize) {
        return special;
    }
    let mut z = (i - SPECIALS.len() as u64 + 1)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z & 0xFF_FFFF) as u32
}

fn spec_named(name: &str) -> Spec {
    Spec::from_name(name).unwrap_or_else(|| panic!("unknown spec {name}"))
}

fn style_named(name: &str) -> Style {
    Style::from_name(name).unwrap_or_else(|| panic!("unknown style {name}"))
}

fn data_lines(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.split_whitespace().collect())
}

/// One line of `digests.txt`, checked; `Err` describes a mismatch.
fn check_digest(fields: &[&str]) -> Result<(), String> {
    let style = style_named(fields[0]);
    let dark = fields[1] == "1";
    let contrast = contrast_level(fields[2]).expect("contrast name");
    let spec = spec_named(fields[3]);
    let amoled = fields[4] == "1";
    let want = u64::from_str_radix(fields[5], 16).expect("digest");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for i in 0..SEEDS {
        let seed = 0xFF00_0000 | seed_at(i);
        for word in roles(&scheme(seed, dark, amoled, style, contrast, spec)) {
            for byte in word.to_be_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
            }
        }
    }
    if hash == want {
        Ok(())
    } else {
        Err(format!("{fields:?}: got {hash:016x}"))
    }
}

#[test]
fn digests_match_the_jvm() {
    let lines: Vec<Vec<&str>> = data_lines(DIGESTS).collect();
    assert_eq!(
        lines.len(),
        9 * 2 * 4 * 2 + 2 * 4 * 2,
        "one line per combination"
    );
    let workers = thread::available_parallelism().map_or(4, |n| n.get());
    let failures: Vec<String> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|w| {
                let lines = &lines;
                scope.spawn(move || {
                    let mine = lines.iter().skip(w).step_by(workers);
                    mine.filter_map(|fields| check_digest(fields).err())
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("worker"))
            .collect()
    });
    assert!(
        failures.is_empty(),
        "{} digests differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn named_rows_match_the_jvm() {
    let mut checked = 0;
    for fields in data_lines(ROWS) {
        let seed = 0xFF00_0000 | u32::from_str_radix(fields[0], 16).expect("seed");
        let style = style_named(fields[1]);
        let dark = fields[2] == "1";
        let spec = spec_named(fields[3]);
        let got = roles(&scheme(seed, dark, false, style, 0.0, spec));
        for (i, want) in fields[4..].iter().enumerate() {
            let want = u32::from_str_radix(want, 16).expect("role");
            assert_eq!(got[i], want, "{} for {fields:?}", NAMES[i]);
        }
        checked += 1;
    }
    assert_eq!(checked, 8 * 9 * 2 * 2);
}

#[test]
fn amoled_forces_black_and_white_in_dark_only() {
    for style in Style::ALL {
        for spec in Spec::ALL {
            let plain = scheme(0xFF6750A4, true, false, style, 0.0, spec);
            let pure = scheme(0xFF6750A4, true, true, style, 0.0, spec);
            assert_eq!([pure.background, pure.surface], [0xFF00_0000; 2]);
            assert_eq!([pure.on_background, pure.on_surface], [0xFFFF_FFFF; 2]);
            assert_eq!(pure.primary, plain.primary);
            let light = scheme(0xFF6750A4, false, true, style, 0.0, spec);
            assert_eq!(light, scheme(0xFF6750A4, false, false, style, 0.0, spec));
        }
    }
}

#[test]
fn names_round_trip() {
    for style in Style::ALL {
        assert_eq!(Style::from_name(style.name()), Some(style));
    }
    for spec in Spec::ALL {
        assert_eq!(Spec::from_name(spec.name()), Some(spec));
    }
    assert_eq!(Style::from_name("tonalSpot"), None);
    assert_eq!(Spec::from_name("SPEC_2023"), None);
    let levels = ["Default", "Medium", "High", "Reduced"].map(contrast_level);
    assert_eq!(levels, [Some(0.0), Some(0.5), Some(1.0), Some(-1.0)]);
    assert_eq!(contrast_level("Low"), None);
}

#[test]
fn seed_alpha_is_ignored() {
    let opaque = scheme(
        0xFF12_3456,
        true,
        false,
        Style::Vibrant,
        0.5,
        Spec::Spec2025,
    );
    let clear = scheme(
        0x0012_3456,
        true,
        false,
        Style::Vibrant,
        0.5,
        Spec::Spec2025,
    );
    assert_eq!(opaque, clear);
}
