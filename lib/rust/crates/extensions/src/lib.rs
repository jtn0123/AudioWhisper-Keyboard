//! Extension domain policy, independent of Android, file handles, ZIP readers and UI resources.
#![forbid(unsafe_code)]

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

/// Packed data at the JNI boundary. Input/output fields are documented by ExtensionPolicy.kt.
pub fn call(operation: i32, input: &[Text]) -> Result<Vec<Text>, Error> {
    match (operation, input) {
        (0, [rule, value]) => Ok(validation::validate(&utf8(rule), value)),
        (1, [value]) => Ok(maintainer::parse(value)
            .map(|r| r.iter().map(|n| text(&n.to_string())).collect())
            .unwrap_or_default()),
        (2, [name, email, url]) => Ok(vec![maintainer::format(name, email, url)]),
        (3, [value]) => {
            let positions = value
                .iter()
                .enumerate()
                .filter(|(_, u)| **u == 58)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            Ok(if let [at] = positions[..] {
                vec![vec![], value[..at].to_vec(), value[at + 1..].to_vec()]
            } else {
                vec![text(
                    "Extension component name must be of format <ext_id>:<comp_id>",
                )]
            })
        }
        (4, [root, entries @ ..]) => {
            let (chunks, rest) = entries.as_chunks::<4>();
            if !rest.is_empty() {
                return Err(malformed());
            }
            let entries = chunks
                .iter()
                .map(|v| {
                    Ok(archive::Entry {
                        name: &v[0],
                        canonical_path: &v[1],
                        size: utf8(&v[2]).parse().map_err(|_| malformed())?,
                        directory: v[3] == text("1"),
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Ok(archive::plan(root, &entries)
                .iter()
                .map(|action| text(action.name()))
                .collect())
        }
        (5, [raw, canonical]) => Ok(
            match match manifest::first_discriminator(raw) {
                Some(value) => manifest::Kind::of(&value),
                None => manifest::kind(canonical),
            } {
                Some(kind) => vec![
                    text(kind.serial()),
                    text(kind.group()),
                    text(kind.component_field()),
                ],
                None => vec![text("unknown"), vec![], vec![]],
            },
        ),
        (6, [serial]) => Ok(match manifest::Kind::of(serial) {
            Some(kind) => vec![vec![], text(kind.group())],
            None => vec![text("Unknown extension type")],
        }),
        (7, [requested, serial]) => Ok(vec![text(
            if manifest::Kind::of(serial).is_some_and(|kind| manifest::accepts(requested, kind)) {
                "1"
            } else {
                "0"
            },
        )]),
        (8, [supported, parsed, core]) => Ok(vec![text(if supported != &text("1") {
            "unsupported"
        } else if parsed != &text("1") {
            "corrupted"
        } else if core == &text("1") {
            "core"
        } else {
            ""
        })]),
        (9, names) => Ok(archive::names(names)
            .iter()
            .map(|&accepted| text(if accepted { "1" } else { "0" }))
            .collect()),
        (10, [id, label, stylesheet, authors @ ..]) => {
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
            let mut plan = vec![
                text(error),
                trim(id),
                trim(label),
                if present { stylesheet.clone() } else { vec![] },
                text(if present { "1" } else { "0" }),
            ];
            plan.extend(authors.iter().filter(|a| !trim(a).is_empty()).cloned());
            Ok(plan)
        }
        (11, [id, stylesheet]) => Ok(vec![if trim(stylesheet).is_empty() {
            let mut path = text("stylesheets/");
            path.extend(id);
            path.extend(text(".json"));
            path
        } else {
            stylesheet.clone()
        }]),
        (14, [serial]) => Ok(vec![text(if serial == &text("with-rules") {
            "with-rules"
        } else {
            "appender"
        })]),
        (15, [id, version, title, maintainers, license]) => {
            let valid = ["id", "version", "title", "maintainers", "license"]
                .into_iter()
                .zip([id, version, title, maintainers, license])
                .all(|(rule, value)| validation::validate(rule, value)[0] == text("valid"));
            Ok(vec![text(if valid { "1" } else { "0" })])
        }
        _ => Err(malformed()),
    }
}
