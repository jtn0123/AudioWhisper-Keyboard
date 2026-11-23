//! Every icon against what an Android device made of it through the Kotlin `iconPath` this crate
//! replaced (29cb06dcd): `tests/fixtures/golden.tsv`, written by `scripts/icon-golden.sh`. Each
//! line holds the icon's point count and FNV-1a-64 of the text and of the flattened points' bits,
//! so the text is held to the digit and the points to the bit before any formatting.

use crate::approximate::approximate;
use crate::format::icon_path;
use crate::nodes::parse;
use crate::skia::build;
use crate::sources;

const GOLDEN: &str = include_str!("../tests/fixtures/golden.tsv");

fn fnv(text: &str) -> String {
    let hash = text.bytes().fold(0xCBF2_9CE4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
    });
    format!("{hash:016x}")
}

/// The flattened points of `name`'s paths as the dump wrote them: each float's bits in hex,
/// paths split by `|`.
fn points(source: &sources::Source) -> String {
    let paths = source.paths.iter().map(|svg| {
        let flat = approximate(&build(&parse(svg).unwrap_or_default()), 0.25);
        flat.iter()
            .map(|p| format!("{:08x}", p.to_bits()))
            .collect::<Vec<_>>()
            .join(" ")
    });
    paths.collect::<Vec<_>>().join("|")
}

#[test]
fn every_icon_is_the_path_android_made() {
    let lines: Vec<&str> = GOLDEN.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), sources::all().len(), "one line per icon");
    let (mut text, mut flat, mut count) = (Vec::new(), Vec::new(), Vec::new());
    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        let source = &sources::all()[sources::index(fields[0]).expect(fields[0])];
        let drawn = icon_path(source);
        let points = drawn.split(' ').filter(|t| *t == "M" || *t == "L").count();
        if points.to_string() != fields[1] {
            count.push(fields[0]);
        }
        if fnv(&drawn) != fields[2] {
            text.push(fields[0]);
        }
        if fnv(&self::points(source)) != fields[3] {
            flat.push(fields[0]);
        }
    }
    assert!(
        text.is_empty() && flat.is_empty() && count.is_empty(),
        "{} differ in points: {flat:?}; {} in count: {count:?}; {} in text: {text:?}",
        flat.len(),
        count.len(),
        text.len()
    );
}
