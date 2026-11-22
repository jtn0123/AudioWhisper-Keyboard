//! What the importer shows of each file, as FileInfoView showed it, and which files it skips:
//! ExtensionPolicy's reasons, then a type the importer on screen does not take.
use super::common::*;
use super::import_common::*;
use import::{File, Skip, parse, skip};
use wire::settings_ui::extensions::file;

const INK: u32 = 0xff1c_1b1f;
const BODY: u32 = 0xff1d_1d1d;
const ERROR: u32 = 0xffb3_261e;

/// The importer at `route` showing `files`, in colours a test can tell apart.
fn coloured(route: &str, files: &[Vec<String>]) -> (App, Text) {
    let (mut a, mut text) = showing(route, "w1", files);
    (a.colors[1], a.colors[15], a.colors[18]) = (INK, BODY, ERROR);
    a.scene(&mut text);
    (a, text)
}
/// The painted text `label`.
fn item(items: &[render::panels::Item], label: &str) -> render::panels::Item {
    let found = items.iter().find(|i| i.label == label).cloned();
    found.unwrap_or_else(|| panic!("nothing painted {label:?}"))
}
fn parsed() -> Vec<File> {
    parse(&files_report("w1", true, &five())).unwrap().2
}

#[test]
fn a_file_box_shows_what_file_info_view_showed() {
    let (mut a, mut text) = coloured(ANY, &five());
    let expected = [
        "ocean.flex",
        FLEX,
        "12 kB • my.ocean • 1.2.0",
        "Ocean themes",
        "Blue and green.",
        "By: Ann, Bo",
        "ocean_day",
        "ocean_night",
    ];
    // Its name over its type, then the rest top to bottom, all of it read as text.
    assert_eq!(box_labels(&a, 0), expected);
    let boxed: Vec<Control> = a
        .controls
        .iter()
        .filter(|c| expected.contains(&c.label.as_str()))
        .cloned()
        .collect();
    assert!(boxed.iter().all(|c| c.role == "text"));
    let tops: Vec<f32> = boxed.iter().map(|c| c.bounds.top).collect();
    assert!(tops.windows(2).all(|w| w[0] < w[1]), "{tops:?}");
    let items = painted(&mut a, &mut text);
    // The size, id and version at 56% of the text colour; the description in italics.
    assert_eq!(item(&items, expected[2]).text.foreground, 0x8f1c_1b1f);
    assert!(item(&items, "Blue and green.").text.italic);
    assert!(!item(&items, "Ocean themes").text.italic);
    assert_eq!(item(&items, "By: Ann, Bo").text.foreground, BODY);
    // 8 dp between the head, the title, who made it and the components.
    let gap = |k: usize| boxed[k + 1].bounds.top - boxed[k].bounds.bottom;
    for k in [2, 4, 5] {
        assert!((gap(k) - px(8.0)).abs() <= 1.0, "{k}: {}", gap(k));
    }
    assert!(gap(3).abs() <= 1.0);
    // A file whose type nobody named is application/unknown, as Compose had it.
    assert_eq!(
        box_labels(&a, 4)[..3],
        ["notes.txt", "application/unknown", "2 B"]
    );
}

#[test]
fn skip_reasons_follow_extension_policy() {
    let files = parsed();
    let reasons: Vec<Option<Skip>> = files.iter().map(|f| skip(ANY, f)).collect();
    let expected = [
        None,
        None,
        Some(Skip::Core),
        Some(Skip::Corrupted),
        Some(Skip::Unsupported),
    ];
    assert_eq!(reasons, expected);
    // Not typed as an archive comes first, whatever the file holds.
    let mut untyped = files[2].clone();
    untyped.flex = false;
    assert_eq!(skip(ANY, &untyped), Some(Skip::Unsupported));
    let (mut a, mut text) = coloured(ANY, &five());
    let shown = [(2, "Core."), (3, "Corrupted."), (4, "Unsupported.")];
    let items = painted(&mut a, &mut text);
    for (f, reason) in shown {
        let labels = box_labels(&a, f);
        assert_eq!(
            labels[labels.len() - 2..],
            ["File can not be imported. Reason:", reason]
        );
        assert!(item(&items, reason).text.italic);
        assert_eq!(item(&items, reason).text.foreground, ERROR);
    }
    let label = item(&items, "File can not be imported. Reason:");
    assert_eq!((label.text.foreground, label.text.italic), (ERROR, false));
    // The thin bar over each reason: 1 dp of the error colour at 56%, 10 dp above it.
    let bars = items
        .iter()
        .filter(|i| i.label.is_empty() && i.styles[0].background == 0x8fb3_261e);
    assert_eq!(bars.count(), 3);
    for f in [0, 1] {
        assert!(
            !box_labels(&a, f)
                .iter()
                .any(|l| l.starts_with("File can not"))
        );
    }
}

/// Improvement: ExtensionImportScreen listed an archive of another type as importable, skipped
/// it when Import was tapped and still said the import worked. It now says it will be skipped,
/// and Import leaves it out, or stays off when nothing else can be imported.
#[test]
fn a_file_of_another_type_is_shown_skipped_and_never_imported() {
    let files = parsed();
    let takes =
        |route: &str| -> Vec<bool> { files.iter().map(|f| skip(route, f).is_none()).collect() };
    assert_eq!(takes(ANY), [true, true, false, false, false]);
    assert_eq!(takes(THEMES), [true, false, false, false, false]);
    assert_eq!(takes(KEYBOARDS), [false, true, false, false, false]);
    assert_eq!(takes(PACKS), [false; 5]);
    assert_eq!(skip(THEMES, &files[1]), Some(Skip::OtherType));
    let (mut a, mut text) = coloured(THEMES, &five());
    let keys = box_labels(&a, 1);
    assert_eq!(
        keys[keys.len() - 2..],
        ["File can not be imported. Reason:", "Other type."]
    );
    assert!(tap(&mut a, &mut text, IMPORT));
    assert_eq!(a.effects[0][5], "0");
    let (a, _) = coloured(PACKS, &five());
    assert!(!control(&a, IMPORT).enabled);
    let (a, _) = coloured(KEYBOARDS, &five()[7..9]);
    assert!(control(&a, IMPORT).enabled);
}

/// Fix: an italic line was measured upright, which wraps elsewhere, so a description that fits
/// on one italic line left a blank line under it (seen on the emulator). A description of every
/// length is exactly as tall as the italic text painted in it.
#[test]
fn an_italic_description_is_as_tall_as_its_painted_text() {
    let said = "Default themes (both day and night) for the keyboard UI, in blue, green and grey";
    let words: Vec<&str> = said.split(' ').collect();
    let mut wrapped_apart = 0;
    for n in 1..=words.len() {
        let said = words[..n].join(" ");
        let mut files = five();
        files[2] = line(file::DESCRIPTION, "", ["", "", "", &said], false);
        let (mut a, mut text) = showing(ANY, "w1", &files);
        let items = painted(&mut a, &mut text);
        let painted = items.iter().find(|i| i.label == said).expect("painted");
        let (s, b) = (&painted.text, painted.bounds);
        let metrics = [s.line_height, s.letter_spacing];
        let mut italic = text.layout_typography(&said, s.size, false, 400, true, metrics);
        italic.break_all_lines(Some(b.right - b.left));
        assert_eq!(b.bottom - b.top, italic.height().ceil(), "{n} words");
        let control = a.controls.iter().find(|c| c.label == said).expect("read");
        assert_eq!(control.bounds, b);
        let upright = text.android(&said, s.size, 400, metrics, Some(b.right - b.left));
        wrapped_apart += usize::from(upright.height.ceil() != italic.height().ceil());
    }
    // The sweep crosses a place where the two wrap apart, or it proves nothing.
    assert!(wrapped_apart > 0);
}

#[test]
fn a_malformed_report_is_refused() {
    let good = files_report("w1", true, &five());
    assert!(parse(&good).is_ok());
    assert!(parse(&good[..good.len() - 1]).is_err());
    assert!(parse(&[]).is_err());
    // It starts with the workspace, and an archive's details follow a file and its archive.
    assert!(parse(&five()[0]).is_err());
    let orphan = files_report("w1", true, &[five()[1].clone()]);
    assert!(parse(&orphan).is_err());
    let loose = files_report("w1", true, &[five()[11].clone(), five()[2].clone()]);
    assert!(parse(&loose).is_err());
    let (mut a, _) = importer(&format!("{ANY}?files=w1"));
    assert!(a.set_import_files(&good[1..]).is_err());
}
