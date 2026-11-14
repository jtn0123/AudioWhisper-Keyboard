//! Stored primitive admission retains Jetpref's nullable parse behavior. Editing bounds are
//! separate: old files may contain any representable number, including floating-point NaN.
use crate::{Result, Text, catalogue, text};
pub fn long(raw: &[u16]) -> Option<i64> {
    let (&first, rest) = raw.split_first()?;
    let (negative, digits) = match first {
        45 => (true, rest),
        43 => (false, rest),
        _ => (false, raw),
    };
    if digits.is_empty() {
        return None;
    }
    let mut value = 0i128;
    for &unit in digits {
        value = value
            .checked_mul(10)?
            .checked_add(i128::from(chars::code_point::digit(u32::from(unit))?))?;
        if value > i128::from(i64::MAX) + 1 {
            return None;
        }
    }
    i64::try_from(if negative { -value } else { value }).ok()
}
pub fn decode(key: &[u16], raw: &[u16]) -> Result<Vec<Text>> {
    let pref = catalogue::get(key)?;
    let value = match pref.kind {
        "b" => {
            if raw == text("true") {
                Some(text("true"))
            } else if raw == text("false") {
                Some(text("false"))
            } else {
                None
            }
        }
        "i" => chars::text::to_int_or_null(raw).map(|v| text(&v.to_string())),
        "l" => long(raw).map(|v| text(&v.to_string())),
        "f" => {
            gif::java_float::to_float_or_null(raw).map(|v| text(&(v.to_bits() as i32).to_string()))
        }
        "s" => Some(raw.to_vec()),
        _ => return Err("unknown primitive type"),
    };
    Ok(value.into_iter().collect())
}
