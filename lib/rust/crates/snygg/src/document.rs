//! Ordered JSON-object structure. The host codec supplies decoded schema text and keys;
//! property payloads stay opaque. A plan defers rule errors until the corresponding value
//! would be visited, preserving precedence over errors in earlier property/value decoders.
use crate::{
    Error, Text,
    grammar::{self, Rule},
    text,
};

pub const SCHEMA_KEY: &str = "$schema";
pub const SCHEMA_V2: &str = "https://schemas.florisboard.org/snygg/v2/stylesheet";

#[derive(Clone, Copy, Debug, Default)]
pub struct Configuration {
    pub ignore_missing_schema: bool,
    pub ignore_invalid_schema: bool,
    pub ignore_unsupported_schema: bool,
    pub ignore_invalid_rules: bool,
}

pub fn schema(schema: Option<&[u16]>, config: Configuration) -> Result<Text, Error> {
    let Some(schema) = schema else {
        return if config.ignore_missing_schema {
            Ok(text(SCHEMA_V2))
        } else {
            Err(Error::MissingSchema)
        };
    };
    // The dots are intentionally wildcards: Kotlin's original SCHEMA_PATTERN did not escape them.
    let mut at = 0;
    let prefix = consume(schema, &mut at, "https://schemas");
    let first = wildcard(schema, &mut at);
    let middle = consume(schema, &mut at, "florisboard");
    let second = wildcard(schema, &mut at);
    let tail = consume(schema, &mut at, "org/snygg/v");
    let digits = at;
    while matches!(schema.get(at), Some(0x30..=0x39)) {
        at += 1;
    }
    let had_digits = at > digits;
    let suffix = consume(schema, &mut at, "/stylesheet");
    let valid =
        prefix && first && middle && second && tail && had_digits && suffix && at == schema.len();
    let schema = if valid {
        schema.to_vec()
    } else if config.ignore_invalid_schema {
        text(SCHEMA_V2)
    } else {
        return Err(Error::InvalidSchema(schema.to_vec()));
    };
    if schema != text(SCHEMA_V2) && !config.ignore_unsupported_schema {
        return Err(Error::UnsupportedSchema(schema));
    }
    Ok(schema)
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub source: usize,
    pub slot: usize,
    pub rule: Result<Rule, Error>,
}

pub fn plan(keys: &[Text], config: Configuration) -> Vec<Entry> {
    let mut slots = Vec::new();
    let mut entries = Vec::new();
    for (source, key) in keys.iter().enumerate() {
        if *key == text(SCHEMA_KEY) {
            continue;
        }
        let rule = match grammar::parse(key) {
            Ok(Some(rule)) => Ok(rule),
            Ok(None) if config.ignore_invalid_rules => continue,
            Ok(None) => Err(Error::InvalidRule(key.clone())),
            Err(error) => Err(error),
        };
        let slot = if let Ok(rule) = &rule {
            if let Some(i) = slots.iter().position(|r| r == rule) {
                i
            } else {
                slots.push(rule.clone());
                slots.len() - 1
            }
        } else {
            0
        };
        let failed = rule.is_err();
        entries.push(Entry { source, slot, rule });
        if failed {
            break;
        }
    }
    entries
}

pub fn serialized_keys(rules: &[Rule]) -> Vec<Text> {
    std::iter::once(text(SCHEMA_KEY))
        .chain(rules.iter().map(Rule::format))
        .collect()
}

fn consume(input: &[u16], at: &mut usize, value: &str) -> bool {
    let value = text(value);
    if input.get(*at..).is_some_and(|s| s.starts_with(&value)) {
        *at += value.len();
        true
    } else {
        false
    }
}

fn wildcard(input: &[u16], at: &mut usize) -> bool {
    if *at >= input.len() {
        return false;
    }
    let (cp, length) = chars::text::code_point_at(input, *at);
    if matches!(cp, 0xA | 0xD | 0x85 | 0x2028 | 0x2029) {
        return false;
    }
    *at += length;
    true
}
