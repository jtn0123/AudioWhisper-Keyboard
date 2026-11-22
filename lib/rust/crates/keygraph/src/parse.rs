//! Reading layout and popup-mapping files as FlorisBoard's `DefaultJsonConfig` did: `$` names a
//! key's class and an unknown or missing one reads as a `text_key`, unknown fields are ignored,
//! a field left out takes its default, and a quoted number or an unquoted string is accepted.
//! Anything else that config rejects fails the whole file, which then counts as not loaded.

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::node::{Arena, Leaf, Mapping, Node, Popup};

/// Why a file didn't read; the caller treats every reason alike, as a missing file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub &'static str);

type Result<T> = std::result::Result<T, Error>;

/// `KeyType`'s names, in ordinal order.
const KEY_TYPES: [&str; 10] = [
    "CHARACTER",
    "ENTER_EDITING",
    "FUNCTION",
    "LOCK",
    "MODIFIER",
    "NAVIGATION",
    "SYSTEM_GUI",
    "NUMERIC",
    "PLACEHOLDER",
    "UNSPECIFIED",
];

/// `KeyVariation`'s names on the wire, by value.
const VARIATIONS: [&str; 5] = ["all", "email", "normal", "password", "uri"];

/// A layout file's rows, or `None` if it doesn't read; `arena` keeps nothing of a file that
/// doesn't.
pub fn layout(text: &str, arena: &mut Arena) -> Option<Vec<Vec<Node>>> {
    let mark = arena.mark();
    let rows = serde_json::from_str::<Value>(text)
        .map_err(|_| Error("not JSON"))
        .and_then(|value| rows(&value, arena));
    rows.inspect_err(|_| arena.rollback(mark)).ok()
}

/// A popup-mapping file, or `None` if it doesn't read.
pub fn mapping(text: &str, arena: &mut Arena) -> Option<Mapping> {
    let mark = arena.mark();
    let mapping = serde_json::from_str::<Value>(text)
        .map_err(|_| Error("not JSON"))
        .and_then(|value| read_mapping(&value, arena));
    mapping.inspect_err(|_| arena.rollback(mark)).ok()
}

fn rows(value: &Value, arena: &mut Arena) -> Result<Vec<Vec<Node>>> {
    array(value)?
        .iter()
        .map(|row| array(row)?.iter().map(|key| node(key, arena)).collect())
        .collect()
}

fn read_mapping(value: &Value, arena: &mut Arena) -> Result<Mapping> {
    let mut mapping = Mapping::default();
    for (name, labels) in object(value)? {
        let variation = VARIATIONS
            .iter()
            .position(|known| known == name)
            .ok_or(Error("unknown key variation"))?;
        let mut sets = HashMap::new();
        for (label, set) in object(labels)? {
            let popup = popup(set, arena)?;
            sets.insert(label.encode_utf16().collect(), popup);
        }
        mapping.variations[variation] = Some(sets);
    }
    Ok(mapping)
}

/// One key description, polymorphic on `$`.
pub fn node(value: &Value, arena: &mut Arena) -> Result<Node> {
    let fields = object(value)?;
    let class = match fields.get("$") {
        None | Some(Value::Null) => None,
        Some(Value::String(name)) => Some(name.as_str()),
        Some(Value::Number(_) | Value::Bool(_)) => Some(""),
        Some(_) => return Err(Error("a class name must be a primitive")),
    };
    Ok(match class {
        Some("auto_text_key") => {
            let leaf = leaf(fields, arena, true)?;
            arena.autos.push((leaf.code, leaf.label.clone()));
            Node::Auto(leaf, arena.autos.len() - 1)
        }
        Some("multi_text_key") => {
            // Its code is transient: always this, whatever the file says.
            let mut leaf = leaf(fields, arena, false)?;
            leaf.code = keys::codes::MULTIPLE_CODE_POINTS;
            let codes = match fields.get("codePoints") {
                None => Vec::new(),
                Some(value) => array(value)?.iter().map(int).collect::<Result<_>>()?,
            };
            arena.multis.push(codes);
            Node::Multi(leaf, arena.multis.len() - 1)
        }
        Some("case_selector") => Node::Select(0, required(fields, &["lower", "upper"], arena)?),
        Some("shift_state_selector") => {
            let names = [
                "unshifted",
                "shifted",
                "shiftedManual",
                "shiftedAutomatic",
                "capsLock",
                "default",
            ];
            Node::Select(1, optionals(fields, &names, arena)?)
        }
        Some("variation_selector") => {
            let names = ["default", "email", "uri", "normal", "password"];
            Node::Select(2, optionals(fields, &names, arena)?)
        }
        Some("layout_direction_selector") => {
            Node::Select(3, required(fields, &["ltr", "rtl"], arena)?)
        }
        Some("char_width_selector") => {
            // Nullable but without a default: each must be written, null or not.
            let mut children = Vec::new();
            for name in ["full", "half"] {
                if !fields.contains_key(name) {
                    return Err(Error("a required field is missing"));
                }
                children.push(optional(fields, name, arena)?);
            }
            Node::Select(4, children)
        }
        _ => Node::Text(leaf(fields, arena, true)?),
    })
}

/// Children that must be present and not null.
fn required(
    fields: &Map<String, Value>,
    names: &[&str],
    arena: &mut Arena,
) -> Result<Vec<Option<Node>>> {
    names
        .iter()
        .map(|name| match fields.get(*name) {
            None | Some(Value::Null) => Err(Error("a required key is missing")),
            Some(value) => node(value, arena).map(Some),
        })
        .collect()
}

/// Children that may be left out or null.
fn optionals(
    fields: &Map<String, Value>,
    names: &[&str],
    arena: &mut Arena,
) -> Result<Vec<Option<Node>>> {
    names
        .iter()
        .map(|name| optional(fields, name, arena))
        .collect()
}

fn optional(fields: &Map<String, Value>, name: &str, arena: &mut Arena) -> Result<Option<Node>> {
    match fields.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => node(value, arena).map(Some),
    }
}

/// The fields every key has; `code` is false for a class whose code isn't read.
fn leaf(fields: &Map<String, Value>, arena: &mut Arena, code: bool) -> Result<Leaf> {
    let mut leaf = Leaf::unspecified();
    if let Some(value) = fields.get("type") {
        let name = text(value)?.to_uppercase();
        let kind = KEY_TYPES.iter().position(|known| *known == name);
        leaf.kind = kind.ok_or(Error("unknown key type"))? as i32;
    }
    if let Some(value) = fields.get("code").filter(|_| code) {
        leaf.code = int(value)?;
    }
    if let Some(value) = fields.get("label") {
        leaf.label = text(value)?.encode_utf16().collect();
    }
    if let Some(value) = fields.get("groupId") {
        leaf.group = int(value)?;
    }
    leaf.popup = match fields.get("popup") {
        None | Some(Value::Null) => None,
        Some(value) => Some(popup(value, arena)?),
    };
    Ok(leaf)
}

/// A popup set, added to `arena`; its index.
fn popup(value: &Value, arena: &mut Arena) -> Result<usize> {
    let fields = object(value)?;
    let main = optional(fields, "main", arena)?;
    let relevant = match fields.get("relevant") {
        None => Vec::new(),
        Some(value) => array(value)?
            .iter()
            .map(|key| node(key, arena))
            .collect::<Result<_>>()?,
    };
    arena.popups.push(Popup { main, relevant });
    Ok(arena.popups.len() - 1)
}

fn object(value: &Value) -> Result<&Map<String, Value>> {
    value.as_object().ok_or(Error("expected an object"))
}

fn array(value: &Value) -> Result<&Vec<Value>> {
    value.as_array().ok_or(Error("expected an array"))
}

/// A string field; the lenient config also takes an unquoted literal as its text.
fn text(value: &Value) -> Result<String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Number(number) => Ok(number.to_string()),
        Value::Bool(flag) => Ok(flag.to_string()),
        _ => Err(Error("expected a string")),
    }
}

/// An `Int` field, quoted or not.
fn int(value: &Value) -> Result<i32> {
    let wide = match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse::<i64>().ok(),
        _ => None,
    };
    wide.and_then(|wide| i32::try_from(wide).ok())
        .ok_or(Error("expected an Int"))
}
