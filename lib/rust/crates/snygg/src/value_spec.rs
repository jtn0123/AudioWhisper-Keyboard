//! Built-in value specifications. Patterns are also exported verbatim to the public
//! Kotlin metadata API and JSON schema; parsing never invokes Java's regex engine.
use crate::{Text, text};
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

pub type Mapping = Vec<(Text, Text)>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    pub id: Option<String>,
    pub pattern: String,
    pub template: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    State(Text),
    Argument(Text),
    Number(Text),
}
impl Failure {
    pub fn state(s: &str) -> Self {
        Self::State(text(s))
    }
    pub fn argument(s: &str) -> Self {
        Self::Argument(text(s))
    }
}
fn pattern(s: String) -> String {
    format!(r"\s*{s}\s*").replace(r"\s*\s*", r"\s*")
}
impl Spec {
    pub fn atom(id: &str, body: &str, unit: &str) -> Self {
        Self {
            id: Some(id.into()),
            pattern: pattern(format!("(?<{id}>{body}){unit}")),
            template: format!("{{{id}}}{unit}"),
        }
    }
    pub fn float(id: &str, unit: &str) -> Self {
        Self::atom(id, r"(?:0|[1-9][0-9]*)(?:[.][0-9]*)?", unit)
    }
    pub fn percent(id: &str, integer: bool) -> Self {
        Self::atom(
            id,
            if integer {
                "100|[1-9]?[0-9]"
            } else {
                r"100(?:[.]0*)?|[1-9]?[0-9](?:[.][0-9]*)?"
            },
            "%",
        )
    }
    pub fn list(separator: &str, specs: &[Self]) -> Self {
        Self {
            id: None,
            pattern: pattern(
                specs
                    .iter()
                    .map(|s| s.pattern.as_str())
                    .collect::<Vec<_>>()
                    .join(separator),
            ),
            template: specs
                .iter()
                .map(|s| s.template.as_str())
                .collect::<Vec<_>>()
                .join(separator),
        }
    }
    pub fn function(name: &str, inner: Self) -> Self {
        Self {
            id: None,
            pattern: pattern(format!("{name}[(]{}[)]", inner.pattern)),
            template: format!("{name}({})", inner.template),
        }
    }
    pub fn nothing() -> Self {
        Self {
            id: None,
            pattern: pattern(String::new()),
            template: String::new(),
        }
    }
    pub fn parse(&self, input: &[u16]) -> Result<Mapping, Failure> {
        parse(&self.pattern, input).ok_or_else(|| {
            Failure::State(
                [
                    input.to_vec(),
                    text(" does not match pattern "),
                    text(&self.pattern),
                ]
                .concat(),
            )
        })
    }
    pub fn pack(&self, map: &Mapping) -> Result<Text, Failure> {
        let mut out = text(&self.template);
        let mut at = 0;
        while let Some(open) = out[at..]
            .iter()
            .position(|&u| u == u16::from(b'{'))
            .map(|i| i + at)
        {
            let Some(close) = out[open + 1..]
                .iter()
                .position(|&u| u == u16::from(b'}'))
                .map(|i| i + open + 1)
            else {
                break;
            };
            let open = out[open + 1..close]
                .iter()
                .rposition(|&u| u == u16::from(b'{'))
                .map_or(open, |i| open + 1 + i);
            let key = &out[open + 1..close];
            let value = map
                .iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .ok_or_else(|| {
                    Failure::State([key.to_vec(), text(" not provided in mapping")].concat())
                })?;
            out.splice(open..=close, value.iter().copied());
            at = open + 1;
            if at > out.len() {
                break;
            }
        }
        if parse(&self.pattern, &out).is_none() {
            return Err(Failure::state("cannot serialize"));
        }
        Ok(out)
    }
}
#[derive(Debug)]
struct Compiled {
    regex: Regex,
    names: Vec<String>,
}
static REGEX: LazyLock<Mutex<HashMap<String, Compiled>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static SPACE: LazyLock<String> = LazyLock::new(|| {
    let units = (0..=u16::MAX)
        .filter(|&u| chars::is_regex_space(u))
        .map(|u| format!(r"\x{{{u:x}}}"))
        .collect::<String>();
    format!("[{units}]")
});
/// Captures are sliced from the original UTF-16 using byte-to-unit boundaries. Lone
/// surrogates are represented by U+FFFD only for matching; their captured units survive.
fn parse(pattern: &str, input: &[u16]) -> Option<Mapping> {
    let mut cache = REGEX.lock().ok()?;
    if !cache.contains_key(pattern) {
        let native = format!(r"\A(?:{})\z", pattern.replace(r"\s", &SPACE));
        let regex = Regex::new(&native).ok()?;
        let names = regex.capture_names().flatten().map(str::to_owned).collect();
        cache.insert(pattern.into(), Compiled { regex, names });
    }
    let regex = cache.get(pattern)?;
    let mut string = String::new();
    let mut boundaries = vec![0];
    for (at, cp) in chars::text::code_points(input) {
        let ch = char::from_u32(cp).unwrap_or('\u{fffd}');
        string.push(ch);
        boundaries.resize(string.len(), at);
        boundaries.push(at + if cp > 0xffff { 2 } else { 1 });
    }
    let captures = regex.regex.captures(&string)?;
    Some(
        regex
            .names
            .iter()
            .filter_map(|name| {
                let value = captures.name(name)?;
                Some((
                    text(name),
                    input[boundaries[value.start()]..boundaries[value.end()]].to_vec(),
                ))
            })
            .collect(),
    )
}

pub fn get<'a>(map: &'a Mapping, key: &str) -> &'a [u16] {
    map.iter()
        .rev()
        .find(|(k, _)| *k == text(key))
        .map_or(&[], |(_, v)| v)
}
pub fn integer(value: &[u16]) -> Result<i32, Failure> {
    chars::text::to_int_or_null(value).ok_or_else(|| Failure::Number(value.to_vec()))
}
pub fn float(value: &[u16]) -> Result<f32, Failure> {
    String::from_utf16_lossy(value)
        .parse()
        .map_err(|_| Failure::Number(value.to_vec()))
}
/// Java Float.toString: shortest round-tripping decimal, with at least two significant
/// digits in scientific notation. The f64 formatting input represents the f32 exactly.
pub fn number(v: f32) -> Text {
    if v.is_nan() {
        return text("NaN");
    }
    if v.is_infinite() {
        return text(if v.is_sign_negative() {
            "-Infinity"
        } else {
            "Infinity"
        });
    }
    if v == 0.0 {
        return text(if v.is_sign_negative() { "-0" } else { "0" });
    }
    let scientific = !(0.001..10_000_000.0).contains(&v.abs());
    let mut best = String::new();
    for digits in if scientific { 2 } else { 1 }..=9 {
        best = format!("{:.*e}", digits - 1, f64::from(v));
        if best.parse::<f32>().ok().map(f32::to_bits) == Some(v.to_bits()) {
            break;
        }
    }
    let (mantissa, exponent) = best.split_once('e').unwrap_or((&best, "0"));
    let exp: i32 = exponent.parse().unwrap_or(0);
    if scientific {
        return text(&format!(
            "{}E{exp}",
            if mantissa.contains('.') {
                mantissa.to_owned()
            } else {
                format!("{mantissa}.0")
            }
        ));
    }
    let sign = if v.is_sign_negative() { "-" } else { "" };
    let digits = mantissa.trim_start_matches('-').replace('.', "");
    let point = exp + 1;
    let mut result = if point <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
    } else if point as usize >= digits.len() {
        format!(
            "{sign}{digits}{}",
            "0".repeat(point as usize - digits.len())
        )
    } else {
        format!(
            "{sign}{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    };
    if result.contains('.') {
        while result.ends_with('0') {
            result.pop();
        }
        if result.ends_with('.') {
            result.pop();
        }
    }
    text(&result)
}
