//! Backup/restore decisions. Kotlin owns SAF, codecs, ZIP/file I/O and Room.
#![forbid(unsafe_code)]

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
fn bit(s: &[u16]) -> bool {
    s == text("1")
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
    [
        string(preferences),
        "files/ime/keyboard".into(),
        "files/ime/theme".into(),
        format!("clipboard/{}", CLIPS[0]),
        format!("clipboard/{}", CLIPS[1]),
        format!("clipboard/{}", CLIPS[2]),
    ]
    .iter()
    .map(|s| text(s))
    .collect()
}

/// Restore preserves the historical warning precedence and accepts newer backups.
pub fn metadata(package: &[u16], version: i32, current: i32) -> [&'static str; 2] {
    let warning = if version != current {
        "different_version"
    } else if !package.starts_with(&text("dev.patrickgold.florisboard")) {
        "different_vendor"
    } else {
        ""
    };
    let error = if package.iter().all(|&u| chars::is_whitespace(u)) || version < 64 {
        "invalid_metadata"
    } else {
        ""
    };
    [warning, error]
}

/// Ordered effects. Missing optional entries are skipped; erase still clears selected
/// extension directories and ALL clipboard history even without incoming clipboard data.
pub fn restore(mask: i32, erase: bool, paths: &[Text], present: &[Text]) -> Vec<Text> {
    let mut out = vec![];
    let mut effect = |kind: &str, path: &[u16]| {
        out.extend([text(kind), path.to_vec()]);
    };
    for i in 0..3 {
        if mask & (1 << i) == 0 {
            continue;
        }
        if i == 0 {
            if bit(&present[i]) {
                effect(
                    if erase {
                        "preferences-erase"
                    } else {
                        "preferences-merge"
                    },
                    &paths[i],
                );
            }
        } else {
            if erase {
                effect("clear-directory", &paths[i]);
            }
            if bit(&present[i]) {
                effect("copy-directory", &paths[i]);
            }
        }
    }
    if erase {
        effect("clear-clipboard", &[]);
    }
    for i in 3..6 {
        if mask & (1 << i) != 0 && bit(&present[i]) {
            effect(["text", "image", "video"][i - 3], &paths[i]);
        }
    }
    out
}

/// Protocol only packs plain values; decisions above are usable without JNI.
pub fn call(op: i32, input: &[Text]) -> Result<Vec<Text>, &'static str> {
    Ok(match (op, input) {
        (0, [package, version, current]) => metadata(package, number(version)?, number(current)?)
            .iter()
            .map(|s| text(s))
            .collect(),
        (1, [preferences]) => layout(preferences),
        (2, [mask, erase, preferences, present @ ..]) if present.len() == 6 => {
            restore(number(mask)?, bit(erase), &layout(preferences), present)
        }
        (3, [package, version, timestamp]) => {
            let mut result = text("backup_");
            result.extend(package);
            result.push(95);
            result.extend(version);
            result.push(95);
            result.extend(timestamp);
            result.extend(text(".zip"));
            vec![result]
        }
        (4, [mask, preferences]) => layout(preferences)
            .into_iter()
            .enumerate()
            .filter(|(i, _)| number(mask).is_ok_and(|m| m & (1 << i) != 0))
            .flat_map(|(i, path)| [text(&i.to_string()), path])
            .collect(),
        (5, [kind, types @ ..]) => types
            .iter()
            .enumerate()
            .filter(|(_, t)| *t == kind)
            .map(|(i, _)| text(&i.to_string()))
            .collect(),
        (6, existing) => existing
            .iter()
            .enumerate()
            .filter(|(_, v)| !bit(v))
            .map(|(i, _)| text(&i.to_string()))
            .collect(),
        (7, [mask]) => {
            let mask = number(mask)?;
            let clips = mask & 56;
            vec![
                text(if mask != 0 { "1" } else { "0" }),
                text(if clips != 0 { "1" } else { "0" }),
                text(if clips == 0 {
                    "off"
                } else if clips == 56 {
                    "on"
                } else {
                    "mixed"
                }),
            ]
        }
        (8, []) => vec![
            text(METADATA),
            text(MISSING_METADATA),
            text("clipboard/clipboard_files"),
            text("backup.zip"),
        ],
        (9, [root, entries @ ..]) => {
            let (chunks, rest) = entries.as_chunks::<4>();
            if !rest.is_empty() {
                return Err("invalid backup entries");
            }
            let entries = chunks
                .iter()
                .map(|v| {
                    Ok(extensions::archive::Entry {
                        name: &v[0],
                        canonical_path: &v[1],
                        size: string(&v[2]).parse().map_err(|_| "invalid backup size")?,
                        directory: bit(&v[3]),
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
