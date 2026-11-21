//! Jetpref's fixed ASCII HH:mm:ss.SSS format and ranges.
use crate::{Result, Text, integer, text};
use wire::settings::time::{HOUR, MILLISECOND, MINUTE, SECOND, SIZE};

/// The fields in the order the stored form writes them.
const FIELDS: [usize; SIZE] = [HOUR, MINUTE, SECOND, MILLISECOND];
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
    let mut row = vec![vec![]; SIZE];
    for (field, value) in FIELDS.into_iter().zip(values) {
        row[field] = text(&value.to_string());
    }
    row
}
pub fn encode(values: &[Text]) -> Result<Text> {
    if values.len() != SIZE {
        return Err("invalid time fields");
    }
    let values = [
        integer(&values[HOUR])?,
        integer(&values[MINUTE])?,
        integer(&values[SECOND])?,
        integer(&values[MILLISECOND])?,
    ];
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
