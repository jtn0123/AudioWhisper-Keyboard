//! Extension domain policy, independent of Android, file handles, ZIP readers and UI resources.
#![forbid(unsafe_code)]

use wire::extensions::{
    self as ops, archive_entry, component, group, manifest as wire_manifest, theme,
    validation as wire_validation,
};
use wire::flag;

pub mod archive;
pub mod maintainer;
pub mod manifest;
pub mod validation;

pub type Text = Vec<u16>;
pub fn text(value: &str) -> Text {
    value.encode_utf16().collect()
}
#[derive(Debug)]
pub struct Error(pub &'static str);
fn malformed() -> Error {
    Error("invalid extension policy transfer")
}
fn utf8(value: &[u16]) -> String {
    String::from_utf16_lossy(value)
}
fn trim(value: &[u16]) -> Text {
    let start = value
        .iter()
        .position(|&u| !chars::is_whitespace(u))
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|&u| !chars::is_whitespace(u))
        .map_or(start, |i| i + 1);
    value[start..end].to_vec()
}

/// Where component `id` keeps its stylesheet, `stylesheetPath()`: `stylesheet`, or
/// `stylesheets/<id>.json` when that is blank.
pub fn stylesheet_path(id: &[u16], stylesheet: &[u16]) -> Text {
    if !trim(stylesheet).is_empty() {
        return stylesheet.to_vec();
    }
    let mut path = text("stylesheets/");
    path.extend(id);
    path.extend(text(".json"));
    path
}

/// Packed data at the JNI boundary. The operations and their packets are declared in
/// `wire::extensions`.
pub fn call(operation: i32, input: &[Text]) -> Result<Vec<Text>, Error> {
    match (operation, input) {
        (ops::VALIDATE, [rule, value]) => Ok(validation::validate(&utf8(rule), value)),
        (ops::MAINTAINER_RANGES, [value]) => Ok(maintainer::parse(value)
            .map(|r| r.iter().map(|n| text(&n.to_string())).collect())
            .unwrap_or_default()),
        (ops::FORMAT_MAINTAINER, [name, email, url]) => {
            Ok(vec![maintainer::format(name, email, url)])
        }
        (ops::COMPONENT_NAME, [value]) => Ok(component_name(value)),
        (ops::ARCHIVE, [root, entries @ ..]) => {
            let (chunks, rest) = entries.as_chunks::<{ archive_entry::SIZE }>();
            if !rest.is_empty() {
                return Err(malformed());
            }
            let entries = chunks
                .iter()
                .map(|v| {
                    Ok(archive::Entry {
                        name: &v[archive_entry::NAME],
                        canonical_path: &v[archive_entry::CANONICAL_PATH],
                        size: utf8(&v[archive_entry::BYTES])
                            .parse()
                            .map_err(|_| malformed())?,
                        directory: flag::decode(&v[archive_entry::DIRECTORY]),
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Ok(archive::plan(root, &entries)
                .iter()
                .map(|action| text(action.name()))
                .collect())
        }
        (ops::MANIFEST, [raw, canonical]) => {
            let kind = match manifest::first_discriminator(raw) {
                Some(value) => manifest::Kind::of(&value),
                None => manifest::kind(canonical),
            };
            let mut row = vec![vec![]; wire_manifest::SIZE];
            row[wire_manifest::SERIAL] = text(kind.map_or("unknown", |kind| kind.serial()));
            if let Some(kind) = kind {
                row[wire_manifest::GROUP] = text(kind.group());
                row[wire_manifest::COMPONENT_FIELD] = text(kind.component_field());
            }
            Ok(row)
        }
        (ops::GROUP, [serial]) => Ok(match manifest::Kind::of(serial) {
            Some(kind) => {
                let mut row = vec![vec![]; group::SIZE];
                row[group::GROUP] = text(kind.group());
                row
            }
            None => {
                let mut row = vec![vec![]; group::SIZE];
                row[group::ERROR] = text("Unknown extension type");
                row
            }
        }),
        (ops::ARCHIVE_NAMES, names) => {
            Ok(archive::names(names).into_iter().map(flag::utf16).collect())
        }
        (ops::THEME_COMPONENT, [id, label, stylesheet, authors @ ..]) => {
            Ok(theme_component(id, label, stylesheet, authors))
        }
        (ops::STYLESHEET, [id, stylesheet]) => Ok(vec![stylesheet_path(id, stylesheet)]),
        (ops::COMPOSER_TYPE, [serial]) => Ok(vec![text(if serial == &text(ops::WITH_RULES) {
            ops::WITH_RULES
        } else {
            ops::APPENDER
        })]),
        (ops::METADATA_VALID, [id, version, title, maintainers, license]) => {
            let valid = [
                ops::ID,
                ops::VERSION,
                ops::TITLE,
                ops::MAINTAINERS,
                ops::LICENSE,
            ]
            .into_iter()
            .zip([id, version, title, maintainers, license])
            .all(|(rule, value)| {
                validation::validate(rule, value)[wire_validation::STATUS] == text(ops::VALID)
            });
            Ok(vec![flag::utf16(valid)])
        }
        _ => Err(malformed()),
    }
}

/// `ext:comp` as a [`component`] row.
fn component_name(value: &[u16]) -> Vec<Text> {
    let positions = value
        .iter()
        .enumerate()
        .filter(|(_, u)| **u == 58)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let mut row = vec![vec![]; component::SIZE];
    if let [at] = positions[..] {
        row[component::EXTENSION] = value[..at].to_vec();
        row[component::COMPONENT] = value[at + 1..].to_vec();
    } else {
        row[component::ERROR] =
            text("Extension component name must be of format <ext_id>:<comp_id>");
    }
    row
}

/// A theme component as a [`theme`] row.
fn theme_component(id: &[u16], label: &[u16], stylesheet: &Text, authors: &[Text]) -> Vec<Text> {
    let error = if trim(id).is_empty() {
        "Theme component ID cannot be blank"
    } else if trim(label).is_empty() {
        "Theme component label cannot be blank"
    } else if authors.is_empty() {
        "Theme component authors must contain at least one non-blank author field"
    } else {
        ""
    };
    let present = !trim(stylesheet).is_empty();
    let mut row = vec![vec![]; theme::AUTHORS];
    row[theme::ERROR] = text(error);
    row[theme::ID] = trim(id);
    row[theme::LABEL] = trim(label);
    if present {
        row[theme::STYLESHEET] = stylesheet.clone();
    }
    row[theme::STYLESHEET_PRESENT] = flag::utf16(present);
    row.extend(authors.iter().filter(|a| !trim(a).is_empty()).cloned());
    row
}
