//! The persisted scalar domain is intentionally wider than the slider-editing domain.
//! Jetpref historically accepts arbitrary parsed numbers. Only UI edits clamp to these bounds.
use crate::{Result, Text, text};
#[derive(Clone, Debug)]
pub struct Preference<'a> {
    pub key: &'a str,
    pub kind: &'a str,
    pub default: &'a str,
    pub min: &'a str,
    pub max: &'a str,
    pub step: &'a str,
    pub choices: &'a str,
}
pub fn all() -> Vec<Preference<'static>> {
    include_str!("catalogue.tsv")
        .lines()
        .map(|line| {
            let v: Vec<_> = line.split('\t').collect();
            Preference {
                key: v[0],
                kind: v[1],
                default: v[2],
                min: v[3],
                max: v[4],
                step: v[5],
                choices: v[6],
            }
        })
        .collect()
}
pub fn get(key: &[u16]) -> Result<Preference<'static>> {
    all()
        .into_iter()
        .find(|p| text(p.key) == key)
        .ok_or("unknown preference")
}
pub fn clamp(key: &[u16], raw: &[u16]) -> Result<Text> {
    let p = get(key)?;
    let value = gif::java_float::to_float_or_null(raw).ok_or("invalid float")?;
    let lo = p.min.parse::<f32>().map_err(|_| "no editing range")?;
    let hi = p.max.parse::<f32>().map_err(|_| "no editing range")?;
    // Kotlin coerceIn preserves NaN, including the original spelling of in-range inputs.
    Ok(if value < lo {
        text(p.min)
    } else if value > hi {
        text(p.max)
    } else {
        raw.to_vec()
    })
}
/// Enum.valueOf: exact, case-sensitive membership; callers retain nullable parse semantics.
pub fn enum_index(key: &[u16], raw: &[u16]) -> Result<i64> {
    Ok(get(key)?
        .choices
        .split('|')
        .position(|v| text(v) == raw)
        .map_or(-1, |v| v as i64))
}
