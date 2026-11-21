//! Backup/restore decisions. Kotlin owns SAF, codecs, ZIP/file I/O and Room.
#![forbid(unsafe_code)]

use wire::backup::{
    self as ops, effect, layout, manifest, metadata_status as status, names, restore_request,
    selection,
};
use wire::extensions::archive_entry as entry;
use wire::flag;

pub type Text = Vec<u16>;
fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}
fn string(s: &[u16]) -> String {
    String::from_utf16_lossy(s)
}
fn number(s: &[u16]) -> Result<i32, &'static str> {
    chars::text::to_int_or_null(s).ok_or("invalid backup policy integer")
}

pub const METADATA: &str = "backup_metadata.json";
pub const MISSING_METADATA: &str =
    "Invalid archive: either backup_metadata.json is missing or file is not a ZIP archive.";
const CLIPS: [&str; 3] = [
    "clipboard_text_items.json",
    "clipboard_images.json",
    "clipboard_video.json",
];

/// Group order is preferences, keyboard extensions, themes, text, images, video.
pub fn layout(preferences: &[u16]) -> Vec<Text> {
    let mut paths = vec![vec![]; layout::SIZE];
    paths[layout::PREFERENCES] = preferences.to_vec();
    paths[layout::KEYBOARDS] = text("files/ime/keyboard");
    paths[layout::THEMES] = text("files/ime/theme");
    for (group, file) in CLIPBOARD.into_iter().zip(CLIPS) {
        paths[group] = text(&format!("clipboard/{file}"));
    }
    paths
}

/// The clipboard's groups, in the order of [`CLIPS`].
const CLIPBOARD: [usize; 3] = [
    layout::CLIPBOARD_TEXT,
    layout::CLIPBOARD_IMAGES,
    layout::CLIPBOARD_VIDEO,
];

/// Whether `mask` selects `group`.
fn selects(mask: i32, group: usize) -> bool {
    mask & (1 << group) != 0
}

/// Restore preserves the historical warning precedence and accepts newer backups.
pub fn metadata(package: &[u16], version: i32, current: i32) -> [&'static str; 2] {
    let warning = if version != current {
        ops::DIFFERENT_VERSION
    } else if !package.starts_with(&text("dev.patrickgold.florisboard")) {
        ops::DIFFERENT_VENDOR
    } else {
        ""
    };
    let error = if package.iter().all(|&u| chars::is_whitespace(u)) || version < 64 {
        ops::INVALID_METADATA
    } else {
        ""
    };
    [warning, error]
}

/// Ordered effects. Missing optional entries are skipped; erase still clears selected
/// extension directories and ALL clipboard history even without incoming clipboard data.
pub fn restore(mask: i32, erase: bool, paths: &[Text], present: &[Text]) -> Vec<Text> {
    let mut out = vec![];
    let mut step = |kind: &str, path: &[u16]| {
        let mut row = vec![vec![]; effect::SIZE];
        row[effect::EFFECT] = text(kind);
        row[effect::PATH] = path.to_vec();
        out.extend(row);
    };
    let present = |group: usize| flag::decode(&present[group]);
    let preferences = layout::PREFERENCES;
    if selects(mask, preferences) && present(preferences) {
        let kind = if erase {
            ops::PREFERENCES_ERASE
        } else {
            ops::PREFERENCES_MERGE
        };
        step(kind, &paths[preferences]);
    }
    for group in [layout::KEYBOARDS, layout::THEMES] {
        if !selects(mask, group) {
            continue;
        }
        if erase {
            step(ops::CLEAR_DIRECTORY, &paths[group]);
        }
        if present(group) {
            step(ops::COPY_DIRECTORY, &paths[group]);
        }
    }
    if erase {
        step(ops::CLEAR_CLIPBOARD, &[]);
    }
    let kinds = [ops::RESTORE_TEXT, ops::RESTORE_IMAGES, ops::RESTORE_VIDEO];
    for (group, kind) in CLIPBOARD.into_iter().zip(kinds) {
        if selects(mask, group) && present(group) {
            step(kind, &paths[group]);
        }
    }
    out
}

/// Protocol only packs plain values; decisions above are usable without JNI.
pub fn call(op: i32, input: &[Text]) -> Result<Vec<Text>, &'static str> {
    Ok(match (op, input) {
        (ops::METADATA, [package, version, current]) => {
            let [warning, error] = metadata(package, number(version)?, number(current)?);
            let mut row = vec![vec![]; status::SIZE];
            row[status::WARNING] = text(warning);
            row[status::ERROR] = text(error);
            row
        }
        (ops::LAYOUT, [preferences]) => layout(preferences),
        (ops::RESTORE, input) if input.len() == restore_request::SIZE + layout::SIZE => {
            use restore_request::{ERASE, MASK, PREFERENCES, SIZE};
            restore(
                number(&input[MASK])?,
                flag::decode(&input[ERASE]),
                &layout(&input[PREFERENCES]),
                &input[SIZE..],
            )
        }
        (ops::FILE_NAME, [package, version, timestamp]) => {
            let mut result = text("backup_");
            result.extend(package);
            result.push(95);
            result.extend(version);
            result.push(95);
            result.extend(timestamp);
            result.extend(text(".zip"));
            vec![result]
        }
        (ops::MANIFEST, [mask, preferences]) => layout(preferences)
            .into_iter()
            .enumerate()
            .filter(|(group, _)| number(mask).is_ok_and(|mask| selects(mask, *group)))
            .flat_map(|(group, path)| {
                let mut row = vec![vec![]; manifest::SIZE];
                row[manifest::GROUP] = text(&group.to_string());
                row[manifest::PATH] = path;
                row
            })
            .collect(),
        (ops::ITEMS, [kind, types @ ..]) => types
            .iter()
            .enumerate()
            .filter(|(_, t)| *t == kind)
            .map(|(i, _)| text(&i.to_string()))
            .collect(),
        (ops::INCOMING, existing) => existing
            .iter()
            .enumerate()
            .filter(|(_, v)| !flag::decode(v))
            .map(|(i, _)| text(&i.to_string()))
            .collect(),
        (ops::SELECTION, [mask]) => {
            let mask = number(mask)?;
            let all = CLIPBOARD.iter().fold(0, |all, group| all | (1 << group));
            let clips = mask & all;
            let mut row = vec![vec![]; selection::SIZE];
            row[selection::ANY] = flag::utf16(mask != 0);
            row[selection::CLIPBOARD] = flag::utf16(clips != 0);
            row[selection::CLIPBOARD_STATE] = text(if clips == 0 {
                ops::OFF
            } else if clips == all {
                ops::ON
            } else {
                ops::MIXED
            });
            row
        }
        (ops::NAMES, []) => {
            let mut row = vec![vec![]; names::SIZE];
            row[names::METADATA] = text(METADATA);
            row[names::MISSING_METADATA] = text(MISSING_METADATA);
            row[names::MEDIA_DIRECTORY] = text("clipboard/clipboard_files");
            row[names::INPUT_ARCHIVE] = text("backup.zip");
            row
        }
        (ops::ARCHIVE, [root, entries @ ..]) => {
            let (chunks, rest) = entries.as_chunks::<{ entry::SIZE }>();
            if !rest.is_empty() {
                return Err("invalid backup entries");
            }
            let entries = chunks
                .iter()
                .map(|v| {
                    Ok(extensions::archive::Entry {
                        name: &v[entry::NAME],
                        canonical_path: &v[entry::CANONICAL_PATH],
                        size: string(&v[entry::BYTES])
                            .parse()
                            .map_err(|_| "invalid backup size")?,
                        directory: flag::decode(&v[entry::DIRECTORY]),
                    })
                })
                .collect::<Result<Vec<_>, &'static str>>()?;
            extensions::archive::plan(root, &entries)
                .iter()
                .map(|a| text(a.name()))
                .collect()
        }
        _ => return Err("invalid backup policy transfer"),
    })
}

#[cfg(test)]
mod tests;
