//! Jetpref's fixed ASCII HH:mm:ss.SSS format and ranges.
use crate::{Result, Text, integer, text};
pub fn decode(raw: &[u16]) -> Vec<Text> {
    if raw.len() != 12 || raw[2] != 58 || raw[5] != 58 || raw[8] != 46 {
        return vec![];
    }
    if raw
        .iter()
        .enumerate()
        .any(|(i, &u)| ![2, 5, 8].contains(&i) && !(48..=57).contains(&u))
    {
        return vec![];
    }
    let parts = [&raw[..2], &raw[3..5], &raw[6..8], &raw[9..]];
    let values: Vec<_> = parts.iter().map(|v| integer(v).unwrap_or(-1)).collect();
    if values
        .iter()
        .zip([24, 60, 60, 1000])
        .any(|(&v, hi)| v < 0 || v >= hi)
    {
        return vec![];
    }
    values.iter().map(|v| text(&v.to_string())).collect()
}
pub fn encode(values: &[Text]) -> Result<Text> {
    let [h, m, s, ms] = values else {
        return Err("invalid time fields");
    };
    let values = [integer(h)?, integer(m)?, integer(s)?, integer(ms)?];
    if values
        .iter()
        .zip([24, 60, 60, 1000])
        .any(|(&v, hi)| v < 0 || v >= hi)
    {
        return Err("invalid time");
    }
    Ok(text(&format!(
        "{:02}:{:02}:{:02}.{:03}",
        values[0], values[1], values[2], values[3]
    )))
}
