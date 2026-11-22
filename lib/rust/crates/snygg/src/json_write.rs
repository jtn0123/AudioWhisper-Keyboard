//! The JSON Compose's theme editor saved a stylesheet as: `SnyggStylesheet.toJson` with
//! `PrettyPrintConfig`, which is kotlinx's `prettyPrint` with a two-space indent. `"$schema"` comes
//! first, then each rule in the stylesheet's order as `Rule::format` writes it, each property as
//! its encoder serializes it. A font rule is an array of property objects. Strings are escaped as
//! kotlinx's `printQuoted` escapes them: only `"`, `\` and the C0 controls.
use crate::{
    Text, document,
    grammar::Rule,
    json::JsonError,
    properties,
    resolve::{Properties, PropertySets, Stylesheet},
    text,
    value::Value,
};

/// The indent kotlinx writes per level.
const INDENT: &str = "  ";

/// `sheet` as Compose saved it, or why it cannot be: a value with no serialized form, or a font
/// set without its `src`, the failures `toJson` throws.
pub fn to_pretty_json(schema: &[u16], sheet: &Stylesheet<Value>) -> Result<String, JsonError> {
    let mut out = String::from("{");
    entry(&mut out, 1, true, &text(document::SCHEMA_KEY));
    quoted(&mut out, schema);
    for (rule, sets) in &sheet.rules {
        entry(&mut out, 1, false, &rule.format());
        match sets {
            PropertySets::Single(props) => object(&mut out, 1, rule, props)?,
            PropertySets::Multiple(all) => array(&mut out, 1, rule, all)?,
        }
    }
    out.push_str("\n}");
    Ok(out)
}

fn array(out: &mut String, depth: usize, rule: &Rule, all: &[Properties]) -> Result<(), JsonError> {
    out.push('[');
    for (i, props) in all.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        newline(out, depth + 1);
        object(out, depth + 1, rule, props)?;
    }
    close(out, depth, !all.is_empty(), ']');
    Ok(())
}

fn object(
    out: &mut String,
    depth: usize,
    rule: &Rule,
    props: &Properties,
) -> Result<(), JsonError> {
    let names: Vec<Text> = props.iter().map(|(n, _)| n.clone()).collect();
    if let Some(key) = properties::missing(&names, &required(rule)) {
        let (rule, key) = (lossy(&rule.format()), lossy(key));
        return Err(JsonError(format!(
            "Rule '{rule}' is missing required property '{key}'"
        )));
    }
    out.push('{');
    for (i, (name, value)) in props.iter().enumerate() {
        let serialized = value
            .encoder()
            .serialize(value)
            .map_err(|e| JsonError(format!("{e:?}")))?;
        entry(out, depth + 1, i == 0, name);
        quoted(out, &serialized);
    }
    close(out, depth, !props.is_empty(), '}');
    Ok(())
}

/// The properties a rule's set must have, as `SnyggPropertyCodec.missing` checks them.
fn required(rule: &Rule) -> Vec<Text> {
    if matches!(rule, Rule::Font(_)) {
        vec![text("src")]
    } else {
        vec![]
    }
}

fn entry(out: &mut String, depth: usize, first: bool, key: &[u16]) {
    if !first {
        out.push(',');
    }
    newline(out, depth);
    quoted(out, key);
    out.push_str(": ");
}

fn close(out: &mut String, depth: usize, any: bool, end: char) {
    if any {
        newline(out, depth);
    }
    out.push(end);
}

fn newline(out: &mut String, depth: usize) {
    out.push('\n');
    out.push_str(&INDENT.repeat(depth));
}

fn lossy(s: &[u16]) -> String {
    String::from_utf16_lossy(s)
}

/// A JSON string as kotlinx prints one.
fn quoted(out: &mut String, s: &[u16]) {
    out.push('"');
    for c in char::decode_utf16(s.iter().copied()) {
        match c.unwrap_or(char::REPLACEMENT_CHARACTER) {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}
