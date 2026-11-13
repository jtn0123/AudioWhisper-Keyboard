//! Domain errors are resource keys, so Compose retains identical translated UI messages.
use crate::{Text, text};

pub const META_ID_REGEX: &str = r"^[a-z][a-z0-9_]*(\.[a-z0-9][a-z0-9_]*)*$";
pub const COMPONENT_ID_REGEX: &str = r"^[a-z][a-z0-9_]*$";
pub const STYLESHEET_REGEX: &str = "^[^:*<>\"']*$";

fn blank(value: &[u16]) -> bool {
    value.iter().all(|&u| chars::is_whitespace(u))
}
fn lower(c: u16) -> bool {
    (97..=122).contains(&c)
}
fn digit(c: u16) -> bool {
    (48..=57).contains(&c)
}
fn id_part(value: &[u16], first: bool) -> bool {
    value
        .first()
        .is_some_and(|&c| lower(c) || (!first && digit(c)))
        && value.iter().all(|&c| lower(c) || digit(c) || c == 95)
}
fn package_id(value: &[u16]) -> bool {
    value
        .split(|&u| u == 46)
        .enumerate()
        .all(|(i, part)| id_part(part, i == 0))
}

/// Status, resource key, optional substitution name and regex. These are policy, not Android IDs.
pub fn validate(rule: &str, value: &[u16]) -> Vec<Text> {
    let error = |key: &str| {
        vec![
            text("invalid"),
            text(&format!("ext__validation__{key}")),
            vec![],
            vec![],
        ]
    };
    let valid = || vec![text("valid"), vec![], vec![], vec![]];
    let pattern_error = |key, arg, pattern| {
        vec![
            text("invalid"),
            text(&format!("ext__validation__{key}")),
            text(arg),
            text(pattern),
        ]
    };
    match rule {
        "id" => {
            if blank(value) {
                error("enter_package_name")
            } else if package_id(value) {
                valid()
            } else {
                pattern_error("error_package_name", "id_regex", META_ID_REGEX)
            }
        }
        "version" => {
            if blank(value) {
                error("enter_version")
            } else {
                valid()
            }
        }
        "title" => {
            if blank(value) {
                error("enter_title")
            } else {
                valid()
            }
        }
        "maintainers" => {
            if blank(value) {
                error("enter_maintainer")
            } else {
                valid()
            }
        }
        "license" => {
            if blank(value) {
                error("enter_license")
            } else {
                valid()
            }
        }
        "componentId" => {
            if blank(value) {
                error("enter_component_id")
            } else if id_part(value, true) {
                valid()
            } else {
                pattern_error(
                    "error_component_id",
                    "component_id_regex",
                    COMPONENT_ID_REGEX,
                )
            }
        }
        "componentLabel" => {
            if blank(value) {
                return error("enter_component_label");
            }
            // Android TextUtils.getTrimmedLength trims only UTF-16 units <= ASCII space.
            let start = value.iter().position(|&u| u > 32).unwrap_or(value.len());
            let end = value.iter().rposition(|&u| u > 32).map_or(start, |i| i + 1);
            if end - start > 30 {
                vec![
                    text("valid"),
                    text("ext__validation__hint_component_label_to_long"),
                    vec![],
                    vec![],
                ]
            } else {
                valid()
            }
        }
        "componentAuthors" => {
            if blank(value) {
                error("error_author")
            } else {
                valid()
            }
        }
        "stylesheet" => {
            if value.is_empty() {
                valid()
            } else if blank(value) {
                error("error_stylesheet_path_blank")
            } else if value.iter().any(|u| [58, 42, 60, 62, 34, 39].contains(u)) {
                pattern_error(
                    "error_stylesheet_path",
                    "stylesheet_path_regex",
                    STYLESHEET_REGEX,
                )
            } else {
                valid()
            }
        }
        _ => error("unknown_rule"),
    }
}
