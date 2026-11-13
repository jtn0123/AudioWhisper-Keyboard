//! Exact syntax of SnyggRule.kt, including findAll behaviour for standalone attributes.
use crate::{Error, Text, attributes::Attributes, compare, text};

pub const SELECTORS: [&str; 5] = ["", ":pressed", ":focus", ":hover", ":disabled"];
pub const DEFINES_PATTERN: &str = "^@defines$";
pub const FONT_PATTERN: &str = r"^@font `(?<fontName>[a-zA-Z0-9\s-]+)`$";
pub const ATTRIBUTE_PATTERN: &str = r"\[(?<attrKey>[a-zA-Z0-9-]+)=(?<attrRawValues>(?:`[^`]+`|(?:0|-?[1-9][0-9]*)[.]{2}(?:0|-?[1-9][0-9]*)|(?:0|-?[1-9][0-9]*))(?:,(?:`[^`]+`|(?:0|-?[1-9][0-9]*)[.]{2}(?:0|-?[1-9][0-9]*)|(?:0|-?[1-9][0-9]*)))*)]";

pub fn element_pattern() -> String {
    format!(
        "^(?<elementName>[a-zA-Z0-9-]+)(?<attributesRaw>(?:{ATTRIBUTE_PATTERN})+)?(?<selectorRaw>:pressed|:focus|:hover|:disabled)?$"
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rule {
    Defines,
    Font(Text),
    Element {
        name: Text,
        attributes: Attributes,
        selector: usize,
    },
}

impl Rule {
    pub fn format(&self) -> Text {
        match self {
            Self::Defines => text("@defines"),
            Self::Font(name) => [text("@font `"), name.clone(), text("`")].concat(),
            Self::Element {
                name,
                attributes,
                selector,
            } => [
                name.clone(),
                attributes.format(),
                text(SELECTORS[*selector]),
            ]
            .concat(),
        }
    }

    pub fn compare(&self, other: &Self) -> i32 {
        match (self, other) {
            (Self::Defines, Self::Defines) => 0,
            (Self::Font(a), Self::Font(b)) => compare(a, b),
            (Self::Defines, Self::Font(_)) => -2,
            (Self::Font(_), Self::Defines) => 2,
            (
                Self::Element {
                    name: a,
                    attributes: aa,
                    selector: sa,
                },
                Self::Element {
                    name: b,
                    attributes: ab,
                    selector: sb,
                },
            ) => {
                let name = compare(a, b);
                if name != 0 {
                    return name;
                }
                let selector = if *sa == 0 && *sb != 0 {
                    -1
                } else if *sb == 0 && *sa != 0 {
                    1
                } else {
                    *sa as i32 - *sb as i32
                };
                if selector != 0 {
                    selector
                } else {
                    aa.compare(ab)
                }
            }
            (Self::Element { .. }, _) => 1,
            (_, Self::Element { .. }) => -1,
        }
    }
}

pub fn name_unit(c: u16) -> bool {
    matches!(c, 0x41..=0x5A | 0x61..=0x7A | 0x30..=0x39 | 0x2D)
}

pub fn valid_name(s: &[u16]) -> bool {
    !s.is_empty() && s.iter().copied().all(name_unit)
}

pub fn selector(s: &[u16]) -> Option<usize> {
    if s.is_empty() {
        return Some(0);
    }
    // SnyggSelector.from drops any first unit, not necessarily a colon.
    SELECTORS
        .iter()
        .position(|v| v.encode_utf16().skip(1).eq(s[1..].iter().copied()))
        .filter(|&i| i != 0)
        .or_else(|| (s[1..] == text("none")).then_some(0))
}

pub fn parse(s: &[u16]) -> Result<Option<Rule>, Error> {
    if s == text("@defines") {
        return Ok(Some(Rule::Defines));
    }
    let prefix = text("@font `");
    if s.starts_with(&prefix) && s.last() == Some(&0x60) && s.len() > prefix.len() + 1 {
        let name = &s[prefix.len()..s.len() - 1];
        if name
            .iter()
            .all(|&c| name_unit(c) || chars::is_regex_space(c))
        {
            return Ok(Some(Rule::Font(name.to_vec())));
        }
    }
    let end = s.iter().position(|&c| !name_unit(c)).unwrap_or(s.len());
    if end == 0 {
        return Ok(None);
    }
    let mut at = end;
    let mut raw = Vec::new();
    while s.get(at) == Some(&0x5B) {
        let Some((next, key, values)) = attribute(s, at) else {
            return Ok(None);
        };
        raw.push((key, values));
        at = next;
    }
    let Some(sel) = SELECTORS
        .iter()
        .position(|v| v.encode_utf16().eq(s[at..].iter().copied()))
    else {
        return Ok(None);
    };
    Ok(Some(Rule::Element {
        name: s[..end].to_vec(),
        attributes: expand(raw)?,
        selector: sel,
    }))
}

#[derive(Debug)]
enum Value {
    String(Text),
    Int(Text),
    Range(Text, Text),
}
type RawAttributes = Vec<(Text, Vec<Value>)>;

fn integer(s: &[u16], at: &mut usize) -> Option<Text> {
    let start = *at;
    if s.get(*at) == Some(&0x30) {
        *at += 1;
        return Some(s[start..*at].to_vec());
    }
    if s.get(*at) == Some(&0x2D) {
        *at += 1;
    }
    if !matches!(s.get(*at), Some(0x31..=0x39)) {
        return None;
    }
    *at += 1;
    while matches!(s.get(*at), Some(0x30..=0x39)) {
        *at += 1;
    }
    Some(s[start..*at].to_vec())
}

fn attribute(s: &[u16], start: usize) -> Option<(usize, Text, Vec<Value>)> {
    let mut at = start + 1;
    let key_start = at;
    while s.get(at).is_some_and(|&c| name_unit(c)) {
        at += 1;
    }
    if at == key_start || s.get(at) != Some(&0x3D) {
        return None;
    }
    let key = s[key_start..at].to_vec();
    at += 1;
    let mut values = Vec::new();
    loop {
        if s.get(at) == Some(&0x60) {
            let start = at + 1;
            at = start;
            while s.get(at).is_some_and(|&c| c != 0x60) {
                at += 1;
            }
            if at == start || s.get(at) != Some(&0x60) {
                return None;
            }
            values.push(Value::String(s[start..at].to_vec()));
            at += 1;
        } else {
            let a = integer(s, &mut at)?;
            if s.get(at..at + 2) == Some(&[0x2E, 0x2E]) {
                at += 2;
                values.push(Value::Range(a, integer(s, &mut at)?));
            } else {
                values.push(Value::Int(a));
            }
        }
        match s.get(at) {
            Some(0x5D) => return Some((at + 1, key, values)),
            Some(0x2C) => at += 1,
            _ => return None,
        }
    }
}

fn expand(raw: RawAttributes) -> Result<Attributes, Error> {
    let mut attrs = Attributes::default();
    for (key, values) in raw {
        let mut expanded = attrs
            .0
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        for value in values {
            match value {
                Value::String(s) | Value::Int(s) => expanded.push(s),
                Value::Range(a, b) => {
                    let a = chars::text::to_int_or_null(&a).ok_or(Error::Number(a))?;
                    let b = chars::text::to_int_or_null(&b).ok_or(Error::Number(b))?;
                    for value in a..=b {
                        expanded.push(text(&value.to_string()));
                    }
                }
            }
        }
        expanded.sort();
        expanded.dedup();
        attrs.put(key, expanded);
    }
    Ok(attrs)
}

pub fn parse_attributes(s: &[u16]) -> Result<Attributes, Error> {
    let mut raw = Vec::new();
    let mut at = 0;
    while at < s.len() {
        if s[at] == 0x5B
            && let Some((next, key, values)) = attribute(s, at)
        {
            raw.push((key, values));
            at = next;
        } else {
            at += 1;
        }
    }
    expand(raw)
}
